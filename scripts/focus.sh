#!/usr/bin/env bash
# T-3655 — the operator-set focus anchor (adopted from 1409-sprind T-1954 fokus.yaml / tools/fokus.py).
#
# The anchor is .context/project/focus.yaml. ONLY the operator sets it; this script never writes it.
#
# Usage: focus.sh show [--line]   print the anchor (--line: one line "FOCUS: <goal> | check: <question>")
#        focus.sh check           is the current task focus inside the anchor?
#
# check is aligned (exit 0) when the current task (.context/working/focus.yaml current_task) is
#   the anchor task, or carries the anchor arc (frontmatter arc_id, or an `arc:<id>` tag), or is named
#   in an in_scope line, or is a WM-* item, or no task is focused at all.
# Exit codes: 0 aligned, 1 mismatch (one line naming focus vs anchor), 2 anchor missing/unreadable.
#
# Seams (fixtures): FOCUS_ANCHOR (anchor file), FOCUS_WORKING (working focus file), FOCUS_TASKS_DIR (.tasks dir).
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
ANCHOR="${FOCUS_ANCHOR:-$ROOT/.context/project/focus.yaml}"
WORKING="${FOCUS_WORKING:-$ROOT/.context/working/focus.yaml}"
TASKS="${FOCUS_TASKS_DIR:-$ROOT/.tasks}"

cmd="${1:-show}"; shift || true
line=0
for a in "$@"; do
    case "$a" in
        --line) line=1 ;;
        *) echo "focus.sh: unknown arg $a" >&2; exit 2 ;;
    esac
done
case "$cmd" in
    show|check) : ;;
    -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
    *) echo "focus.sh: unknown command $cmd (show|check)" >&2; exit 2 ;;
esac

exec python3 -I - "$cmd" "$line" "$ANCHOR" "$WORKING" "$TASKS" <<'PY'
import os, re, sys, glob
cmd, line, anchor, working, tasks = sys.argv[1], sys.argv[2] == "1", sys.argv[3], sys.argv[4], sys.argv[5]
try:
    import yaml
except Exception as e:
    print(f"FOCUS NOT CHECKED: python3 yaml unavailable ({e})"); sys.exit(2)

def one(s):
    return re.sub(r"\s+", " ", str(s or "")).strip()

try:
    with open(anchor) as f:
        a = yaml.safe_load(f)
    if not isinstance(a, dict) or not one(a.get("goal")):
        raise ValueError("no goal")
except Exception as e:
    print(f"FOCUS NOT CHECKED: anchor {anchor} missing or unreadable ({type(e).__name__}: {e})")
    sys.exit(2)

goal, question = one(a.get("goal")), one(a.get("check_question"))
a_task, a_arc = one(a.get("task")), one(a.get("arc"))
in_scope = [one(x) for x in (a.get("in_scope") or [])]

if cmd == "show":
    if line:
        print(f"FOCUS: {goal} | check: {question}")
        sys.exit(0)
    print(f"FOCUS (operator-set, {anchor})")
    print(f"  goal:  {goal}")
    print(f"  arc:   {a_arc or '-'}   task: {a_task or '-'}")
    print(f"  check: {question}")
    for k in ("in_scope", "parked", "never"):
        items = a.get(k) or []
        print(f"  {k}:")
        for x in items:
            print(f"    - {one(x)}")
    sys.exit(0)

# ---- check ----
try:
    with open(working) as f:
        w = yaml.safe_load(f) or {}
    cur = one(w.get("current_task")) if isinstance(w, dict) else ""
except FileNotFoundError:
    cur = ""
except Exception as e:
    print(f"FOCUS NOT CHECKED: working focus {working} unreadable ({type(e).__name__}: {e})")
    sys.exit(2)
if cur in ("", "None", "null"):
    print("FOCUS OK: no task focused"); sys.exit(0)
if cur.startswith("WM-"):
    print(f"FOCUS OK: {cur} (WM item)"); sys.exit(0)
if a_task and cur == a_task:
    print(f"FOCUS OK: {cur} is the anchor task"); sys.exit(0)

def task_arc(tid):
    paths = glob.glob(os.path.join(tasks, "active", f"{tid}-*.md")) + glob.glob(os.path.join(tasks, "completed", f"{tid}-*.md"))
    for p in paths:
        try:
            txt = open(p).read()
            m = re.match(r"^---\n(.*?)\n---", txt, re.S)
            fm = yaml.safe_load(m.group(1)) if m else {}
        except Exception:
            continue
        if not isinstance(fm, dict):
            continue
        if one(fm.get("arc_id")):
            return one(fm.get("arc_id"))
        for t in fm.get("tags") or []:
            t = one(t)
            if t.startswith("arc:"):
                return t[4:].strip()
        return ""
    return ""

arc = task_arc(cur)
if a_arc and arc == a_arc:
    print(f"FOCUS OK: {cur} is in {a_arc}"); sys.exit(0)
if any(re.search(rf"(?<![\w-]){re.escape(cur)}(?![\w])", s) for s in in_scope):
    print(f"FOCUS OK: {cur} is named in in_scope"); sys.exit(0)
print(f"FOCUS MISMATCH: working on {cur} (arc: {arc or 'none'}), anchor is {a_task or '-'} / {a_arc or '-'} "
      f"and in_scope does not name it. Ask: {question}")
sys.exit(1)
PY
