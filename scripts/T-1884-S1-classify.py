#!/usr/bin/env python3
"""
T-1884 S1 spike — classifier for the review-waiting queue (refined).

Reads all active task files, finds those with unchecked ### Human ACs,
classifies each unchecked AC into one of:
  REVIEWER-AGENT-MISFILE    [REVIEWER] prefix under ### Human (should be ### Agent)
  REVIEW-RENDER              UI/dashboard/watch surface (ux-review territory)
  REVIEW-CLI                 CLI output natural-reading (script -c + grep)
  RUBBER-STAMP-MECHANICAL    shell/file/cron/MCP-listing-validatable
  RUBBER-STAMP-RELEASE       GitHub release / gh CLI validatable
  OBSERVE-INFRA              remote-host state observation (termlink remote exec)
  OPERATOR-ACTION            human must do first, agent verifies after
  TIME-GATED                 deferred validation (e.g. "on next deploy")
  OTHER                      ambiguous, needs manual sort

Refinements vs v1 (T-1884 S1 round 1):
  - Trust [RUBBER-STAMP] prefix: default to MECHANICAL absent a stronger
    REVIEW signal (operator intent encoded in prefix).
  - New class OBSERVE-INFRA: state observation against a remote host.
  - New class OPERATOR-ACTION: human-must-do, agent can only verify.
  - New class TIME-GATED: deferred validation gated on future event.
  - Self-exclude T-1884 (don't classify own inception's own AC).

Emits a markdown table + summary distribution. Validates A1 (≥80%
confident routing). Does NOT modify any task files (read-only).
"""
import glob
import re
from pathlib import Path
from collections import Counter

try:
    import yaml
except ImportError:
    yaml = None

ROOT = Path(__file__).resolve().parent.parent
ACTIVE = ROOT / ".tasks" / "active"


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
    # crude fallback if yaml is unavailable
    out = {}
    for line in raw.splitlines():
        m2 = re.match(r"^([a-zA-Z_]+):\s*(.*)$", line)
        if m2:
            out[m2.group(1)] = m2.group(2).strip().strip('"')
    return out


def extract_human_section(text):
    """Return the body of the ### Human subsection of ## Acceptance Criteria."""
    m = re.search(
        r"###\s+Human\s*\n(.*?)(?=\n##\s+|\Z)", text, re.DOTALL
    )
    return m.group(1) if m else None


def extract_unchecked_acs(human_section):
    """Return a list of unchecked AC tuples: (prefix, body, steps_text)."""
    if not human_section:
        return []
    # Each AC starts with "- [ ]" (unchecked) or "- [x]" (checked).
    # We split on AC-start anchors so nested Steps/Expected text stays grouped.
    parts = re.split(r"\n(?=- \[[ xX]\])", human_section)
    out = []
    for p in parts:
        m = re.match(r"- \[ \]\s*(.*?)(?=\n|$)", p, re.DOTALL)
        if not m:
            continue
        first_line = m.group(1).strip()
        # extract optional [PREFIX]
        pm = re.match(r"\[([A-Z][A-Z-]*)\]\s*(.*)", first_line)
        if pm:
            prefix, body = pm.group(1), pm.group(2)
        else:
            prefix, body = None, first_line
        # capture rest of the AC entry (Steps/Expected/If-not) for content
        # classification — strip the first checkbox line.
        rest = p[m.end():] if m.end() < len(p) else ""
        out.append((prefix, body.strip(), rest))
    return out


# ----- classifier --------------------------------------------------------

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
# OBSERVE-INFRA: state of a remote host/service, validated via termlink
# remote exec / shell probe / hub-status query.
OBSERVE_INFRA_KEYWORDS = [
    "var/log", "below 50", "stops rebooting", "running flag-off",
    "≥7 days", ">= 7 days", "ct 200", "ct 101", "production hubs",
    "hub.capabilities", "on next .122 deploy", "on next deploy",
    "post-bake", "post-deploy of t-1418", "freshness signal",
    "verify production", "warning is visible in hub stderr",
    "fleet check", "fleet doctor green", "re-pin",
    "running on new binary", "hub restarted",
]
# OPERATOR-ACTION: a human action that must happen before any verification.
OPERATOR_ACTION_KEYWORDS = [
    "rotate", "revoke", "re-enable onedev", "decide whether",
    "review response artifact and approve",
    "applying the warning",  # "informative without being noisy" needs human taste
]
# TIME-GATED: deferred-event verifications.
TIME_GATED_KEYWORDS = [
    "on next .122 deploy", "on next deploy", "after t-1166 bake",
    "after prod cut", "post-bake", "running flag-off for ≥7 days",
    "running for ≥7 days",
]
# Inception-self-review AC pattern (skip).
SELF_REVIEW_KEYWORDS = [
    "review exploration findings",
    "approve go/no-go decision",
]


def classify(prefix, body, rest, task_id=""):
    """Return (class, confident_bool, reason_short)."""
    text = (body + "\n" + rest).lower()

    # Self-exclude: this inception's own Human-AC template
    if any(kw in text for kw in SELF_REVIEW_KEYWORDS):
        return "OTHER", False, "inception self-review (skip)"

    # Strong: REVIEWER prefix under Human → misfile pattern
    if prefix == "REVIEWER":
        return "REVIEWER-AGENT-MISFILE", True, "REVIEWER prefix on Human AC"

    # OPERATOR-ACTION dominates other classifications — if a human must do
    # something first, no validator can run.
    if any(kw in text for kw in OPERATOR_ACTION_KEYWORDS):
        return "OPERATOR-ACTION", True, "operator-action keyword match"

    # TIME-GATED dominates after OPERATOR-ACTION — a deferred event can't be
    # validated until it occurs.
    if any(kw in text for kw in TIME_GATED_KEYWORDS):
        return "TIME-GATED", True, "time-gated keyword match"

    if prefix == "RUBBER-STAMP":
        if any(kw in text for kw in RELEASE_KEYWORDS):
            return "RUBBER-STAMP-RELEASE", True, "release keyword match"
        if any(kw in text for kw in MECHANICAL_KEYWORDS):
            return "RUBBER-STAMP-MECHANICAL", True, "mechanical keyword match"
        if any(kw in text for kw in OBSERVE_INFRA_KEYWORDS):
            return "OBSERVE-INFRA", True, "observe-infra keyword match (under RUBBER-STAMP)"
        # RUBBER-STAMP without keyword hit — TRUST THE PREFIX (operator intent).
        # Default to MECHANICAL — the most common form, and the orchestrator
        # will dry-run the Steps to confirm.
        return "RUBBER-STAMP-MECHANICAL", True, "RUBBER-STAMP prefix-trust default"

    if prefix == "REVIEW" or prefix is None:
        # Try OBSERVE-INFRA first — these often look CLI-ish but need host probe
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


# ----- main --------------------------------------------------------------

def main():
    rows = []
    for path in sorted(glob.glob(str(ACTIVE / "T-*.md"))):
        text = Path(path).read_text(encoding="utf-8", errors="replace")
        fm = parse_frontmatter(text)
        task_id = fm.get("id", Path(path).stem.split("-", 2)[0:2])
        if isinstance(task_id, list):
            task_id = "-".join(task_id)
        owner = fm.get("owner", "?")
        status = fm.get("status", "?")

        human = extract_human_section(text)
        unchecked = extract_unchecked_acs(human)
        if not unchecked:
            continue

        for (prefix, body, rest) in unchecked:
            klass, confident, reason = classify(prefix, body, rest, task_id)
            rows.append({
                "task": task_id,
                "owner": owner,
                "status": status,
                "prefix": prefix or "(none)",
                "class": klass,
                "confident": confident,
                "ac_preview": body[:72],
                "reason": reason,
            })

    # ---- emit table ----
    print("# T-1884 S1 — classifier output")
    print()
    print(f"Total tasks scanned with unchecked ### Human ACs: "
          f"{len(set(r['task'] for r in rows))}")
    print(f"Total unchecked ACs: {len(rows)}")
    print()

    # Distribution
    print("## Class distribution")
    print()
    print("| Class | Count | Confident | Confident% |")
    print("|---|---:|---:|---:|")
    classes = Counter(r["class"] for r in rows)
    confident_by_class = Counter(r["class"] for r in rows if r["confident"])
    for klass, total in sorted(classes.items(), key=lambda kv: -kv[1]):
        conf = confident_by_class.get(klass, 0)
        pct = (conf / total * 100) if total else 0
        print(f"| {klass} | {total} | {conf} | {pct:.0f}% |")
    print()

    total = len(rows)
    confident = sum(1 for r in rows if r["confident"])
    overall_pct = (confident / total * 100) if total else 0
    print(f"**Overall confidence: {confident}/{total} = {overall_pct:.1f}%**")
    print()
    print(f"GO threshold = ≥80%. "
          f"{'PASS' if overall_pct >= 80 else 'NO-GO' if overall_pct < 50 else 'MARGINAL'}")
    print()

    # Per-task table
    print("## Per-AC classification")
    print()
    print("| Task | Prefix | Class | Conf | AC preview |")
    print("|---|---|---|:---:|---|")
    for r in rows:
        c = "✓" if r["confident"] else "?"
        prev = r["ac_preview"].replace("|", "\\|").replace("\n", " ")
        print(f"| {r['task']} | `{r['prefix']}` | {r['class']} | {c} | {prev} |")
    print()

    # OTHER bucket for manual sort
    others = [r for r in rows if r["class"] == "OTHER"]
    if others:
        print("## OTHER bucket (manual sort needed)")
        print()
        for r in others:
            print(f"- {r['task']}: `{r['prefix']}` — {r['ac_preview']}")
        print()


if __name__ == "__main__":
    main()
