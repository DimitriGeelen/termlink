#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
#
# Human AC "Steps" heading canonical-form check (T-2859).
#
# The Watchtower review renderer matches the Steps heading with an EXACT
# startswith('**Steps:**') (.agentic-framework/web/blueprints/tasks.py:419).
# A deviating heading — '**Steps (copy-paste):**' — drops the ENTIRE Steps
# block from the rendered approval page, INCLUDING the operator's
# copy-pasteable command, while Expected and If-not still render. The page
# looks complete. The operator is asked to approve an action whose command
# is not shown.
#
# Measured on first run: 2 of 347 headings deviated (T-1696, T-2858). One of
# them had been reported to the operator as a verified, stamp-ready approval.
#
# SCOPE — read a green narrowly (T-2680). This checks the Steps heading FORM.
# It does not verify that a task HAS a Steps block, that the steps are correct,
# or that anything else on the page renders.
#
# Exit 0 = every heading canonical · 1 = a deviating heading · 2 = tooling.
# Fail-closed: a missing tasks dir, absent python3, or a corpus with zero task
# files exits 2, never a vacuous clean.

set -uo pipefail

TASKS_DIR="${TASKS_DIR:-.tasks/active}"
ALLOWLIST="${STEPS_HEADING_ALLOWLIST:-.context/checks/steps-heading-allowlist}"
JSON=0; QUIET=0

while [ $# -gt 0 ]; do
  case "$1" in
    --tasks-dir) TASKS_DIR="$2"; shift 2 ;;
    --allowlist) ALLOWLIST="$2"; shift 2 ;;
    --json)  JSON=1; shift ;;
    --quiet) QUIET=1; shift ;;
    --no-heartbeat) shift ;;
    -h|--help) sed -n '3,25p' "$0"; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

command -v python3 >/dev/null 2>&1 || { echo "check-human-ac-steps-heading: python3 not found" >&2; exit 2; }
[ -d "$TASKS_DIR" ] || { echo "check-human-ac-steps-heading: tasks dir not found: $TASKS_DIR" >&2; exit 2; }

TASKS_DIR="$TASKS_DIR" ALLOWLIST="$ALLOWLIST" JSON="$JSON" QUIET="$QUIET" python3 - <<'PY'
import json, os, re, sys, glob

tasks_dir = os.environ["TASKS_DIR"]
allow_path = os.environ["ALLOWLIST"]
as_json = os.environ["JSON"] == "1"
quiet = os.environ["QUIET"] == "1"

files = sorted(glob.glob(os.path.join(tasks_dir, "*.md")))
if not files:
    print(f"check-human-ac-steps-heading: no task files under {tasks_dir}", file=sys.stderr)
    sys.exit(2)

allow = set()
if os.path.exists(allow_path):
    try:
        for ln in open(allow_path, encoding="utf-8"):
            ln = ln.split("#", 1)[0].strip()
            if ln:
                allow.add(ln)
    except OSError as e:
        print(f"check-human-ac-steps-heading: cannot read allowlist {allow_path}: {e}", file=sys.stderr)
        sys.exit(2)

CANON = "**Steps:**"
head_re = re.compile(r"^\s*\*\*Steps\b[^\n]*")

firing, acknowledged, checked = [], [], 0
for f in files:
    try:
        text = open(f, encoding="utf-8").read()
    except OSError as e:
        print(f"check-human-ac-steps-heading: cannot read {f}: {e}", file=sys.stderr)
        sys.exit(2)
    # Template guidance lives inside HTML comments; blank them out (keep line count).
    stripped = re.sub(r"(?s)<!--.*?-->", lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)
    for i, line in enumerate(stripped.split("\n"), 1):
        m = head_re.match(line)
        if not m:
            continue
        checked += 1
        if m.group(0).strip() == CANON:
            continue
        rel = os.path.relpath(f)
        sig = f"{os.path.basename(f)}::{i}"
        entry = {"file": rel, "line": i, "heading": m.group(0).strip()}
        (acknowledged if sig in allow else firing).append(entry)

ok = not firing
if as_json:
    print(json.dumps({
        "ok": ok, "checked": checked,
        "firing": firing, "firing_count": len(firing),
        "acknowledged": acknowledged, "acknowledged_count": len(acknowledged),
        "canonical": CANON,
        "scope": "Steps heading FORM only; does not verify a task has steps or that they are correct",
    }, indent=2))
elif firing:
    print(f"check-human-ac-steps-heading: FIRING — {len(firing)} non-canonical heading(s) of {checked} scanned")
    print("  A heading that is not exactly '**Steps:**' drops the WHOLE Steps block")
    print("  from the rendered approval page — the operator's command included.")
    for e in firing:
        print(f"    {e['file']}:{e['line']}: {e['heading']}")
    print("  Fix: use exactly '**Steps:**'. Put any qualifier inside the steps themselves.")
elif not quiet:
    print(f"check-human-ac-steps-heading: clean — {checked} heading(s) scanned, all exactly '{CANON}'")
    if acknowledged:
        print(f"  ({len(acknowledged)} acknowledged in {allow_path})")
    print("  SCOPE: heading FORM only — says nothing about whether steps exist or are correct.")

sys.exit(0 if ok else 1)
PY
