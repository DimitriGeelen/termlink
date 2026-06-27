#!/usr/bin/env bash
# T-2290 — Finalization-bypass canary (G-066 prevention).
#
# G-066: CTL-028 (T-2203) found 157 tasks in .tasks/completed/ whose `status`
# field still said `started-work` (not `work-completed`) and whose
# `date_finished` was empty — i.e. they were ARCHIVED into completed/ without
# going through the finalize routine (`fw task update --status work-completed`,
# which sets BOTH status and date_finished). Several shared identical move-commit
# timestamps, indicating bulk `git mv` / migration moves that skipped
# finalization. T-2203 repaired the 157 existing files, but the MECHANISM that
# lands tasks in completed/ without finalizing them is unaddressed and will
# recur on the next bulk move. This canary is the cheap structural defense
# (G-066 what_remains option 3a) — it pairs with the existing duplicate-task-ID
# audit check and surfaces a finalization bypass the day it happens instead of
# weeks later via manual archaeology.
#
# Model: scan every .tasks/completed/*.md. A task is a "finalization bypass" if
# its frontmatter `status:` is anything other than `work-completed`. A SEPARATE,
# softer class is a task that IS work-completed but has a null/empty
# `date_finished` (finalize half-ran) — reported as informational by default and
# foldable into the firing set with --strict.
#
# Empty output (in --quiet) = healthy — same convention as the mirror /
# substrate-preflight / framework-pickup / frozen-husk / topic-growth canaries.
# /canaries auto-discovers this canary via the .heartbeat companion + the cron
# log at .context/working/.task-finalization-canary.log.
#
# Exit codes:
#   0  — healthy (every completed/ task is status:work-completed)
#   1  — one or more finalization-bypass tasks detected (status != work-completed)
#   2  — tooling error (no tasks dir / parse failure)
#
# Usage:
#   check-task-finalization-freshness.sh                # human-readable, one-shot
#   check-task-finalization-freshness.sh --json         # JSON envelope for scripting
#   check-task-finalization-freshness.sh --quiet        # print only on firing (cron)
#   check-task-finalization-freshness.sh --strict       # ALSO fire on missing date_finished
#   check-task-finalization-freshness.sh --tasks-dir P  # override .tasks root
#   check-task-finalization-freshness.sh --no-heartbeat # suppress heartbeat touch

set -eu

TASKS_DIR="${TASKS_DIR:-.tasks}"
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.task-finalization-canary.heartbeat}"
FORMAT=human
QUIET=0
HEARTBEAT=1
STRICT=0

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  FORMAT=json ;;
        --quiet) QUIET=1 ;;
        --strict) STRICT=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        --tasks-dir) shift; TASKS_DIR="${1:-.tasks}" ;;
        --tasks-dir=*) TASKS_DIR="${1#*=}" ;;
        -h|--help) sed -n '2,42p' "$0"; exit 0 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

# Heartbeat first (prove the canary ran even on healthy/error cycles — T-1723).
if [ "$HEARTBEAT" = 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch -- "$HEARTBEAT_FILE" 2>/dev/null || true
fi

COMPLETED_DIR="$TASKS_DIR/completed"
if [ ! -d "$COMPLETED_DIR" ]; then
    if [ "$QUIET" != 1 ]; then
        if [ "$FORMAT" = json ]; then
            printf '{"ok": true, "tasks_dir": "%s", "reason": "no completed dir", "bypassed": [], "missing_date": [], "total": 0}\n' "$TASKS_DIR"
        else
            echo "task-finalization canary: healthy — no completed dir at $COMPLETED_DIR (nothing to check)"
        fi
    fi
    exit 0
fi

# Parse every completed/*.md frontmatter in python: classify each task by
# status and date_finished. Emits the rendered report on stdout and a trailing
# "FIRE=<n>" sentinel line for the shell.
REPORT="$(python3 - "$COMPLETED_DIR" "$TASKS_DIR" "$FORMAT" "$STRICT" <<'PY'
import sys, os, glob

completed_dir, tasks_dir, fmt, strict = sys.argv[1], sys.argv[2], sys.argv[3], (sys.argv[4] == "1")

def frontmatter(path):
    # Return dict of top-level scalar keys in the leading --- ... --- block.
    fm = {}
    try:
        with open(path, encoding="utf8", errors="replace") as f:
            lines = f.read().splitlines()
    except Exception:
        return None
    if not lines or lines[0].strip() != "---":
        return {}
    for ln in lines[1:]:
        if ln.strip() == "---":
            break
        if ":" not in ln or ln[:1] in (" ", "\t", "-", "#"):
            continue  # only top-level scalar keys
        k, _, v = ln.partition(":")
        fm[k.strip()] = v.strip().strip('"').strip("'")
    return fm

bypassed = []      # status != work-completed (the firing class)
missing_date = []  # work-completed but date_finished empty (softer class)
total = 0
parse_errors = 0

for path in sorted(glob.glob(os.path.join(completed_dir, "*.md"))):
    fm = frontmatter(path)
    if fm is None:
        parse_errors += 1
        continue
    if not fm:
        continue  # no frontmatter block — not a task file
    total += 1
    status = fm.get("status", "")
    date_finished = fm.get("date_finished", "")
    tid = fm.get("id", os.path.basename(path))
    if status != "work-completed":
        bypassed.append({"id": tid, "status": status or "(none)",
                         "file": os.path.basename(path)})
    elif date_finished in ("", "null", "~", "None"):
        missing_date.append({"id": tid, "file": os.path.basename(path)})

fire = len(bypassed) + (len(missing_date) if strict else 0)

if fmt == "json":
    import json
    print(json.dumps({
        "ok": fire == 0,
        "tasks_dir": tasks_dir,
        "strict": strict,
        "total": total,
        "bypassed": bypassed,
        "missing_date": [m["id"] for m in missing_date],
        "parse_errors": parse_errors,
    }))
else:
    if bypassed:
        print("task-finalization canary: %d completed/ task(s) with status != work-completed "
              "(finalization bypassed)" % len(bypassed))
        for b in bypassed:
            print("  [%s] %s  (%s)" % (b["status"], b["id"], b["file"]))
        print("  → these were archived without `fw task update --status work-completed`.")
        print("    Repair: fw task update <id> --status work-completed (sets status + date_finished),")
        print("    then root-cause the move path that skipped finalize (G-066: bulk git mv / migration).")
    if missing_date:
        label = "FIRING" if strict else "informational"
        print("task-finalization canary [%s]: %d work-completed task(s) with empty date_finished" % (
            label, len(missing_date)))
        for m in missing_date:
            print("  %s  (%s)" % (m["id"], m["file"]))
        if not strict:
            print("  → softer class (status is correct, date_finished half-ran). "
                  "Use --strict to fire on these.")

print("FIRE=%d" % fire)
print("PARSE_ERRORS=%d" % parse_errors)
PY
)" || { echo "task-finalization canary: parse error" >&2; exit 2; }

FIRE="$(printf '%s\n' "$REPORT" | sed -n 's/^FIRE=//p' | tail -1)"
BODY="$(printf '%s\n' "$REPORT" | grep -v -e '^FIRE=' -e '^PARSE_ERRORS=' || true)"

if [ "${FIRE:-0}" = 0 ]; then
    if [ "$QUIET" != 1 ]; then
        if [ "$FORMAT" = json ]; then
            printf '%s\n' "$BODY"
        elif [ -n "$BODY" ]; then
            # not firing but informational missing_date lines present
            printf '%s\n' "$BODY"
            echo "task-finalization canary: healthy — all completed/ tasks are work-completed (exit 0)."
        else
            echo "task-finalization canary: healthy — all completed/ tasks are status:work-completed"
        fi
    fi
    exit 0
fi

# Firing — always print (including --quiet, so the cron log captures it).
printf '%s\n' "$BODY"
exit 1
