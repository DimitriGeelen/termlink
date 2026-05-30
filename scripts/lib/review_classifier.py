"""
T-1885 v0.1 — AC classifier module.

Extracted from scripts/T-1884-S1-classify.py (the spike).
Same 9-class taxonomy, same keyword sets, same prefix-trust default.

Public API:
    extract_ac_entries(task_text) -> list of dict
        Returns each unchecked ### Human AC as
        {prefix, body, rest, full_entry}.

    classify_ac(prefix, body, rest) -> (class, confident, reason)

    extract_steps_commands(ac_entry) -> list of (step_num, cmd_str)

    extract_expected(ac_entry) -> str | None

    parse_frontmatter(text) -> dict
"""
import re

try:
    import yaml
except ImportError:
    yaml = None


# 9 classes — matches T-1884-S1 round 2 (87.5% on 72-AC corpus)
CLASSES = [
    "REVIEWER-AGENT-MISFILE",
    "REVIEW-RENDER",        # NOTE: aliased to CLI-WATCH in v0.1; render = --watch CLI
    "REVIEW-CLI",
    "RUBBER-STAMP-MECHANICAL",
    "RUBBER-STAMP-RELEASE",
    "OBSERVE-INFRA",
    "OPERATOR-ACTION",
    "TIME-GATED",
    "OTHER",
]

# v0.1 routes REVIEW-RENDER → CLI-WATCH validator (the actual surface — see T-1884 S3 finding)
CLI_WATCH_CLASS = "REVIEW-RENDER"


RENDER_KEYWORDS = [
    "watch view", "live view", "live single-peer", "live overview",
    "live dashboard", "no flicker", "steady", "dashboard", "watch is",
    "live thread", "live fleet", "by-project table", "scannable for fleet",
    "no jitter", "no row jitter", "row jumping",
]
CLI_KEYWORDS = [
    "output", "reads naturally", "operator-readable", "operator-scannable",
    "error messages", "error wording", "wording", "naming", "naturally",
    "truncation footer", "empty-with-filter", "empty-with-thread",
    "natural", "scans well", "render", "rendering", "self-doc", "verb's ux",
    "verb reads", "feels right", "operator-fluent",
]
RELEASE_KEYWORDS = [
    "github release", "release published", "release pipeline produced",
    "release tag", "release artifacts", "macos + linux",
    "v0.10.0", "v0.11.0", "v0.11.1",
]
MECHANICAL_KEYWORDS = [
    "cron entry installed", "/etc/cron.d", "mcp listing", "mcp lists",
    "discoverable", "installed in", "file exists", "tap repo exists",
    "binary deployed", "shows the", "upstream landed", "upstream landed on",
    "shipped",
]
OBSERVE_INFRA_KEYWORDS = [
    "var/log", "below 50", "stops rebooting", "running flag-off",
    "≥7 days", ">= 7 days", "ct 200", "ct 101", "production hubs",
    "hub.capabilities", "on next .122 deploy", "on next deploy",
    "post-bake", "post-deploy of t-1418", "freshness signal",
    "verify production", "warning is visible in hub stderr",
    "fleet check", "fleet doctor green", "re-pin",
    "running on new binary", "hub restarted",
]
OPERATOR_ACTION_KEYWORDS = [
    "rotate", "revoke", "re-enable onedev", "decide whether",
    "review response artifact and approve",
    "applying the warning",
]
TIME_GATED_KEYWORDS = [
    "on next .122 deploy", "on next deploy", "after t-1166 bake",
    "after prod cut", "post-bake", "running flag-off for ≥7 days",
    "running for ≥7 days",
]
SELF_REVIEW_KEYWORDS = [
    "review exploration findings",
    "approve go/no-go decision",
]


def parse_frontmatter(text):
    m = re.match(r"^---\n(.*?)\n---", text, re.DOTALL)
    if not m:
        return {}
    raw = m.group(1)
    if yaml:
        try:
            return yaml.safe_load(raw) or {}
        except Exception:
            return {}
    out = {}
    for line in raw.splitlines():
        m2 = re.match(r"^([a-zA-Z_]+):\s*(.*)$", line)
        if m2:
            out[m2.group(1)] = m2.group(2).strip().strip('"')
    return out


def extract_human_section(text):
    m = re.search(
        r"###\s+Human\s*\n(.*?)(?=\n##\s+|\Z)", text, re.DOTALL
    )
    return m.group(1) if m else None


def extract_ac_entries(task_text, include_checked=False):
    """Return list of {prefix, body, rest, full_entry} for each AC.

    By default returns only unchecked ([ ]) ACs; pass include_checked=True
    to also return checked ones.
    """
    human = extract_human_section(task_text)
    if not human:
        return []
    parts = re.split(r"\n(?=- \[[ xX]\])", human)
    out = []
    for p in parts:
        if include_checked:
            m = re.match(r"- \[[ xX]\]\s*(.*?)(?=\n|$)", p, re.DOTALL)
        else:
            m = re.match(r"- \[ \]\s*(.*?)(?=\n|$)", p, re.DOTALL)
        if not m:
            continue
        first_line = m.group(1).strip()
        pm = re.match(r"\[([A-Z][A-Z-]*)\]\s*(.*)", first_line)
        if pm:
            prefix, body = pm.group(1), pm.group(2)
        else:
            prefix, body = None, first_line
        rest = p[m.end():] if m.end() < len(p) else ""
        out.append({
            "prefix": prefix,
            "body": body.strip(),
            "rest": rest,
            "full_entry": p,
        })
    return out


_CMD_FIRST_TOK_RE = re.compile(r"^[a-zA-Z0-9_./-]+(\s|$)")


def _looks_like_command(s):
    """Heuristic: is this backtick content a shell command vs data/identifier?

    Rejects: things with `:` in first token (YAML-like: `revisit_at: 2026-05-01`),
    bare identifiers without separators, things starting with non-cmd chars.
    """
    s = s.strip()
    if not s:
        return False
    if s.startswith("/"):
        return False  # /slash skill invocations
    if s.startswith("-") and not s.startswith("--"):
        return False  # bare flags like `-v`
    # Reject YAML-like "key: value"
    first_token = s.split(None, 1)[0] if s else ""
    if first_token.endswith(":"):
        return False
    if ":" in first_token and not first_token.startswith(("http", "ssh", "file")):
        return False
    # First token must look like a command name or path
    if not _CMD_FIRST_TOK_RE.match(s):
        return False
    return True


def extract_steps_commands(ac_entry):
    """Extract candidate shell commands from the AC entry's Steps block.

    Returns list of (step_num, command_str) tuples. Filters out data/identifier
    backticks (e.g. YAML fragments, flag names without context).
    """
    sm = re.search(
        r"\*\*Steps(?:\s*\(to verify[^)]*\))?\:\*\*\s*\n(.*?)(?=\*\*Expected\*\*|\*\*If not\*\*|\*\*Evidence|\Z)",
        ac_entry, re.DOTALL,
    )
    if not sm:
        return []
    body = sm.group(1)
    cmds = []
    step_lines = re.split(r"\n\s*(?=\d+\.\s)", body)
    for sl in step_lines:
        sm2 = re.match(r"\s*(\d+)\.\s+(.*)", sl, re.DOTALL)
        if not sm2:
            continue
        num = sm2.group(1)
        rest = sm2.group(2)
        for cm in re.finditer(r"`([^`]+)`", rest):
            cmd = cm.group(1).strip()
            if _looks_like_command(cmd):
                cmds.append((num, cmd))
    return cmds


def extract_expected(ac_entry):
    sm = re.search(
        r"\*\*Expected\:\*\*\s*(.*?)(?=\*\*If not\*\*|\*\*Evidence|\n- \[|\Z)",
        ac_entry, re.DOTALL,
    )
    return sm.group(1).strip() if sm else None


def classify_ac(prefix, body, rest):
    """Return (class, confident_bool, reason_short).

    Same classifier as T-1884-S1 round 2 (87.5% on 72-AC corpus).
    """
    text = (body + "\n" + rest).lower()

    if any(kw in text for kw in SELF_REVIEW_KEYWORDS):
        return "OTHER", False, "inception self-review (skip)"

    if prefix == "REVIEWER":
        return "REVIEWER-AGENT-MISFILE", True, "REVIEWER prefix on Human AC"

    if any(kw in text for kw in OPERATOR_ACTION_KEYWORDS):
        return "OPERATOR-ACTION", True, "operator-action keyword match"

    if any(kw in text for kw in TIME_GATED_KEYWORDS):
        return "TIME-GATED", True, "time-gated keyword match"

    if prefix == "RUBBER-STAMP":
        if any(kw in text for kw in RELEASE_KEYWORDS):
            return "RUBBER-STAMP-RELEASE", True, "release keyword match"
        if any(kw in text for kw in MECHANICAL_KEYWORDS):
            return "RUBBER-STAMP-MECHANICAL", True, "mechanical keyword match"
        if any(kw in text for kw in OBSERVE_INFRA_KEYWORDS):
            return "OBSERVE-INFRA", True, "observe-infra keyword match (under RUBBER-STAMP)"
        return "RUBBER-STAMP-MECHANICAL", True, "RUBBER-STAMP prefix-trust default"

    if prefix == "REVIEW" or prefix is None:
        if any(kw in text for kw in OBSERVE_INFRA_KEYWORDS):
            return "OBSERVE-INFRA", True, "observe-infra keyword match"
        render_hits = sum(1 for kw in RENDER_KEYWORDS if kw in text)
        cli_hits = sum(1 for kw in CLI_KEYWORDS if kw in text)
        if render_hits > cli_hits and render_hits > 0:
            return "REVIEW-RENDER", True, f"render keywords x{render_hits}"
        if cli_hits > render_hits and cli_hits > 0:
            return "REVIEW-CLI", True, f"cli keywords x{cli_hits}"
        if render_hits == cli_hits and render_hits > 0:
            if "watch" in text:
                return "REVIEW-RENDER", False, "tied keywords, watch tiebreak"
            return "REVIEW-CLI", False, "tied keywords, cli tiebreak"
        return "OTHER", False, "no keyword match"

    return "OTHER", False, f"unknown prefix={prefix}"


def v01_class_routes_to(klass):
    """v0.1 validator routing — which validator handles this class.

    Returns one of:
      "cli"          — REVIEW-CLI capture + grep
      "watch"        — CLI-WATCH frame capture + stability check
      "release"      — gh release view
      "surface"      — surface-only (OPERATOR-ACTION / TIME-GATED / OTHER)
      "v02-defer"    — RUBBER-STAMP-MECHANICAL / OBSERVE-INFRA (needs remote-exec, v0.2)
      "misfile"      — REVIEWER-AGENT-MISFILE (template error, surface for fix)
    """
    return {
        "REVIEW-CLI": "cli",
        "REVIEW-RENDER": "watch",
        "RUBBER-STAMP-RELEASE": "release",
        "RUBBER-STAMP-MECHANICAL": "v02-defer",
        "OBSERVE-INFRA": "v02-defer",
        "OPERATOR-ACTION": "surface",
        "TIME-GATED": "surface",
        "OTHER": "surface",
        "REVIEWER-AGENT-MISFILE": "misfile",
    }.get(klass, "surface")
