#!/usr/bin/env python3
"""T-1697 — Human AC mechanical-misclassification audit.

Scans .tasks/{active,completed}/*.md, extracts every unticked Human AC,
classifies as mechanical/human-only/ambiguous, emits a punch-list report.
Idempotent: deterministic ordering, no datetime in output content.
"""
from __future__ import annotations
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[0] if (Path.cwd() / ".tasks").exists() else Path.cwd()
ROOT = Path.cwd()
TASKS_ACTIVE = ROOT / ".tasks" / "active"
TASKS_DONE = ROOT / ".tasks" / "completed"
OUT = ROOT / "docs" / "reports" / "T-1697-human-ac-audit.md"

TASK_ID_RE = re.compile(r"^(T-\d+)")
HEADER_HUMAN = re.compile(r"^###\s+Human\b")
SECTION_END = re.compile(r"^(##|###)\s+\S")
CHECKBOX = re.compile(r"^- \[ \]\s+(.*)$")

MECH_PATTERNS = [
    r"\bgrep\b", r"\bcurl\b", r"\bwget\b", r"\bcat\b", r"\bjq\b",
    r"\bawk\b", r"\bsed\b", r"\btest\b", r"\bls\b", r"\bsha256\w*\b",
    r"\bfingerprint\b", r"git ls-remote", r"\bgit log\b", r"\bgit diff\b",
    r"python3\s+-m\s+json\.tool", r"\bjson\b.*\bshow", r"shows\s+`",
    r"returns\s+`?[0-9A-Za-z]", r"\bSHA[0-9]?\b", r"exit\s+code",
    r"matches?\b", r"\bcontains?\b", r"\bequal", r"==\s*",
    r"\bregex\b", r"\bmtime\b", r"\bcron\b", r"`/etc/cron\.d/",
    r"systemctl\b", r"sudo cp\b", r"\bbash scripts/", r"\bfw\s+\w+",
    r"`.*\.json`", r"`.*\.yaml`", r"`.*\.toml`", r"target/release/termlink",
    r"^\s*\d+\.\s+`", r"\bdpkg\b", r"\bapt\b", r"\bsystemd\b",
    r"\bprometheus\b", r"netstat", r"\bss\b\s+-", r"journalctl",
    r"checksum", r"\bdiff\b", r"\bps\b\s+-", r"\bmd5", r"\bonedev\b",
    r"binary equiv", r"byte-equiv", r"installed at",
    r"file exists", r"exists at", r"present", r"count\s*=?\s*\d",
]

HUMAN_PATTERNS = [
    r"\bopen\b.*\bbrowser\b", r"\bin a browser\b", r"\bin the browser\b",
    r"\bnavigate to\b", r"\bclick\b", r"\bscroll\b", r"\btap\b",
    r"\bscreenshot\b", r"\brender", r"\bvisually\b", r"\blook at\b",
    r"\bread\b\s+through", r"\bfeels?\b", r"\bdelight", r"\bbeauty\b",
    r"\bsubjective", r"\bjudg(e|ment|ing)", r"\baesthetic", r"\bUX\b",
    r"\bUI\b", r"\bGitHub\.com\b", r"github\.com web", r"\bWatchtower\b\s+(panel|page|view|UI)",
    r"\bdashboard\b\s+(panel|view|UI)", r"reach for this", r"\bscannable",
    r"\breadable", r"intuitive", r"\bclear\b\s+to\s+(a|the)\s+human",
    r"would\s+you\b", r"do\s+you\s+(prefer|like|think)",
    r"\btone\b", r"\barchitecture\b\s+(call|decision)",
    r"\b(human|operator)\b\s+(verifies?|confirms?|inspects?|reviews?)",
    r"\bphone\b", r"\bdesktop notification",
    r"manually", r"by hand", r"\bonedev\b.*(panel|UI|web)",
    r"\boauth\b", r"\bsign in\b", r"\blogin\b\s+(to|into)",
]

# Subjective predicates: when these appear in the TITLE/Expected (the success criterion),
# they signal human judgment regardless of Steps-block commands. Strong override.
SUBJECTIVE_TITLE_PATTERNS = [
    r"\breads?\s+naturally\b", r"\bfeels?\b", r"\bfluent\b", r"\bfluently\b",
    r"\bsteady\b", r"\bscannable\b", r"\breadable\b", r"\bintuitive\b",
    r"\bdelight", r"\baesthetic", r"\boperator[- ]fluent",
    r"\binformative without being noisy\b", r"\bdiscoverable\b", r"\boperator-actionable\b",
    r"\boperator-friendly\b", r"\busable\b", r"\buseful\b\s+(as|for)",
    r"\bclearly\b\s+(named|stated|described)", r"\bworth (the|its)",
    r"\bworth doing", r"\bgood enough", r"\bclean\b\s+output",
    r"\bvalue-added\b", r"\bworth\b\s+(reach|reaching)",
    r"\bsmooth\b", r"\bsensible\b", r"\bpolished\b",
    r"\bend-to-end\s+from\s+a\s+real\b",
    r"\breach\s+for\s+(this|it)", r"\bbabysit\b", r"\bno flicker\b",
    r"\bno\s+row\s+jitter\b", r"\binformative\b",
    # T-1296 / T-1294 case — "verify reboot persistence" is operator-only by
    # virtue of needing physical/credentialed access to that host
    r"\breboot persistence\b",
]

SUBJ_RE = [re.compile(p, re.I) for p in SUBJECTIVE_TITLE_PATTERNS]

MECH_RE = [re.compile(p, re.I) for p in MECH_PATTERNS]
HUMAN_RE = [re.compile(p, re.I) for p in HUMAN_PATTERNS]


def extract_human_acs(path: Path):
    """Yield (idx, marker, title, body, raw_block) for each unticked AC under ### Human."""
    text = path.read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()
    in_human = False
    i = 0
    acs = []
    while i < len(lines):
        line = lines[i]
        if HEADER_HUMAN.match(line):
            in_human = True
            i += 1
            continue
        if in_human and SECTION_END.match(line):
            in_human = False
        if in_human:
            m = CHECKBOX.match(line)
            if m:
                title = m.group(1).strip()
                body_lines = []
                j = i + 1
                while j < len(lines):
                    nxt = lines[j]
                    if CHECKBOX.match(nxt):
                        break
                    if SECTION_END.match(nxt):
                        break
                    if HEADER_HUMAN.match(nxt):
                        break
                    body_lines.append(nxt)
                    j += 1
                body = "\n".join(body_lines).strip()
                marker = ""
                m2 = re.match(r"\[(RUBBER-STAMP|REVIEW)\]\s*(.*)", title)
                if m2:
                    marker = m2.group(1)
                    title = m2.group(2).strip()
                acs.append((marker, title, body))
                i = j
                continue
        i += 1
    return acs


def classify(marker: str, title: str, body: str):
    full = f"{title}\n{body}"
    mech = sum(1 for r in MECH_RE if r.search(full))
    hum = sum(1 for r in HUMAN_RE if r.search(full))
    # Subjective-predicate override: when the TITLE / EXPECTED contains
    # explicit judgment language ("reads naturally", "is steady", "is operator-fluent"),
    # the success criterion is human regardless of Steps-block commands.
    # Look in title + the "Expected:" line specifically.
    expected_match = re.search(r"\*\*Expected:\*\*\s*([^\n]+(?:\n(?!\s*\*\*)[^\n]+)*)", body)
    expected_text = expected_match.group(1) if expected_match else ""
    subjective_zone = f"{title}\n{expected_text}"
    subj_hits = [r.pattern for r in SUBJ_RE if r.search(subjective_zone)]
    if subj_hits:
        # Strong override unless overwhelming mech signals in Expected (rare).
        sample = subj_hits[0]
        return ("human-only", mech, hum,
                f"subjective predicate in title/Expected: {sample!r} (mech={mech}, human={hum})")
    # Marker tie-breakers
    if marker == "RUBBER-STAMP" and mech > 0 and hum <= 1:
        return ("mechanical", mech, hum, f"RUBBER-STAMP + {mech} mech signals, {hum} human signals")
    if marker == "REVIEW" and hum > 0:
        if mech >= hum:
            return ("ambiguous", mech, hum, f"REVIEW with {mech} mech / {hum} human signals — measurable but author flagged judgment")
        return ("human-only", mech, hum, f"REVIEW + {hum} human signals > {mech} mech signals")
    # No-marker pure classification
    if mech > 0 and hum == 0:
        return ("mechanical", mech, hum, f"{mech} mech signals, 0 human signals")
    if hum > 0 and mech == 0:
        return ("human-only", mech, hum, f"{hum} human signals, 0 mech signals")
    if mech > 0 and hum > 0:
        if mech >= 2 * hum:
            return ("mechanical", mech, hum, f"mech-dominant ({mech} vs {hum})")
        if hum >= 2 * mech:
            return ("human-only", mech, hum, f"human-dominant ({hum} vs {mech})")
        return ("ambiguous", mech, hum, f"mixed signals ({mech} mech / {hum} human)")
    return ("ambiguous", 0, 0, "no strong signals — author intent unclear")


def task_id_of(path: Path) -> str:
    m = TASK_ID_RE.match(path.stem)
    return m.group(1) if m else path.stem


def main():
    rows = []  # (task_id, source, marker, title, body, classification, rationale)
    files = sorted(list(TASKS_ACTIVE.glob("*.md"))) + sorted(list(TASKS_DONE.glob("*.md")))
    for p in files:
        try:
            acs = extract_human_acs(p)
        except Exception as e:
            print(f"WARN: failed to parse {p}: {e}", file=sys.stderr)
            continue
        if not acs:
            continue
        src = "active" if p.parent.name == "active" else "completed"
        tid = task_id_of(p)
        for marker, title, body in acs:
            cls, mech, hum, rat = classify(marker, title, body)
            rows.append((tid, src, marker, title, body, cls, rat, mech, hum))

    # Deterministic sort: task_id numerically, then marker, then title
    def key(r):
        m = re.match(r"T-(\d+)", r[0])
        return (int(m.group(1)) if m else 0, r[2], r[3])
    rows.sort(key=key)

    by_cls = {"mechanical": [], "human-only": [], "ambiguous": []}
    for r in rows:
        by_cls[r[5]].append(r)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    out = []
    out.append("# T-1697 — Human AC mechanical-misclassification audit\n")
    out.append("Punch list per AC for operator review. Classifications are heuristic-based; the operator decides which migrations to authorize per the T-1480/T-1481 (PL-169) pattern.\n")
    out.append(f"**Header counts:** {len(by_cls['mechanical'])} mechanical, {len(by_cls['human-only'])} human-only, {len(by_cls['ambiguous'])} ambiguous (scanned {len(files)} task files; {len(rows)} unticked Human ACs total).\n")
    out.append("Scope: every unticked `- [ ]` under `### Human` in `.tasks/active/*.md` and `.tasks/completed/*.md`.\n")
    out.append("Classifier signals: mechanical = string-match / exit-code / file-exists / fields-present / cron/systemctl/grep/jq/git verbs; human-only = browser/UI/click/render/subjective/judgment language; ambiguous = mixed or no strong signals.\n")
    out.append("\n---\n")

    def row_md(r, include_migration=False):
        tid, src, marker, title, body, cls, rat, mech, hum = r
        mk = f"[{marker}] " if marker else ""
        title_short = title.split("\n", 1)[0]
        if len(title_short) > 180:
            title_short = title_short[:177] + "..."
        line = f"- {tid} ({src}) — {mk}{title_short}\n  - **Why:** {rat}"
        if include_migration:
            migration = (
                f"  - **Proposed migration:** move this AC verbatim under `### Agent` "
                f"in `{tid}` (delete from `### Human`), then have the agent run the Steps "
                f"and tick the box. Original [REVIEW]/[RUBBER-STAMP] marker can drop."
            )
            line += "\n" + migration
        return line

    out.append("## Mechanical\n")
    out.append(f"_{len(by_cls['mechanical'])} ACs_ — success criterion is mechanically verifiable; candidate for `### Agent` migration.\n")
    if by_cls["mechanical"]:
        for idx, r in enumerate(by_cls["mechanical"]):
            out.append(row_md(r, include_migration=(idx < 5)))
    else:
        out.append("_(none)_")
    out.append("\n---\n")

    out.append("## Human-Only\n")
    out.append(f"_{len(by_cls['human-only'])} ACs_ — needs human verification (UI rendering / subjective judgment / authenticated session); leave as `### Human`.\n")
    if by_cls["human-only"]:
        for r in by_cls["human-only"]:
            out.append(row_md(r))
    else:
        out.append("_(none)_")
    out.append("\n---\n")

    out.append("## Ambiguous\n")
    out.append(f"_{len(by_cls['ambiguous'])} ACs_ — operationalizable but author may have intended judgment; flag for operator review.\n")
    if by_cls["ambiguous"]:
        for r in by_cls["ambiguous"]:
            out.append(row_md(r))
    else:
        out.append("_(none)_")

    OUT.write_text("\n".join(out) + "\n", encoding="utf-8")
    print(f"wrote {OUT}")
    print(f"mechanical={len(by_cls['mechanical'])} human-only={len(by_cls['human-only'])} ambiguous={len(by_cls['ambiguous'])} total={len(rows)} files={len(files)}")

    # Spot-check: T-1480 / T-1481 must not appear in mechanical
    bad = [r for r in by_cls["mechanical"] if r[0] in ("T-1480", "T-1481")]
    if bad:
        print(f"SPOT-CHECK FAILED: {bad}", file=sys.stderr)
        sys.exit(2)
    print("spot-check passed (T-1480/T-1481 not in mechanical list)")


if __name__ == "__main__":
    main()
