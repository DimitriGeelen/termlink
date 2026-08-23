#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
#
# scripts/check-stranded-finalized-tasks.sh (T-2833)
#
# Detects a task whose frontmatter says `status: work-completed` while the file
# is still sitting in `.tasks/active/`.
#
# WHY THIS STATE EXISTS, AND WHY NOTHING SAW IT
# ---------------------------------------------
# `update-task.sh` writes the status field unconditionally (line ~1681) and then
# runs the finalize block — set `date_finished`, `git mv` to `completed/`, clear
# focus, generate episodic — 221 lines later, behind this guard (line ~1902):
#
#     if [ -n "$NEW_STATUS" ] && [ "$NEW_STATUS" = "work-completed" ] \
#        && [ "$OLD_STATUS" != "work-completed" ]; then
#
# The two writes are not atomic and the guard reads the value the first write
# already committed to disk. So if a run writes the status and then dies before
# reaching the finalize block, every LATER invocation sees OLD_STATUS is already
# `work-completed`, the condition is false, and the whole finalize block is
# skipped — silently. Not an error; an `if` that does not match. The state is a
# latch: once entered, the normal command can never leave it.
#
# It deadlocks commits, which is why it is not cosmetic. P-002 refuses a commit
# while focus is on a completed task; the T-1730 focus-drift gate refuses one
# while focus is on any other task. The only exits are a logged Tier-2 bypass or
# attributing the commit to an unrelated task.
#
# The T-2290 task-finalization canary cannot see it. That canary scans
# `.tasks/completed/` only, so a task that never leaves `active/` is outside its
# corpus by construction. Same end state as the G-066 finalization-bypass class
# (work done, register disagrees) reached from the opposite direction.
#
# SCOPE — read a green here narrowly (T-2680).
# It answers exactly one question: is any file in `.tasks/active/` declaring
# `status: work-completed`? It does NOT audit whether a task's work is actually
# done, whether its ACs are honest, or whether tasks in `completed/` finalized
# cleanly — that last one is T-2290's corpus, and `--strict` there covers the
# empty-`date_finished` class this defect also produces after the partial
# recovery branch moves a file without stamping the date.
#
# Exit: 0 = no stranded task · 1 = stranded · 2 = tooling (fail-closed).
set -uo pipefail

TASKS_DIR="${TASKS_DIR_OVERRIDE:-.tasks}"
ALLOWLIST_DEFAULT=".context/checks/stranded-finalized-allowlist"
ALLOWLIST=""
JSON=false
QUIET=false
HEARTBEAT=true

die2() { echo "check-stranded-finalized-tasks: $1 (fail-closed: exit 2)" >&2; exit 2; }

while [ $# -gt 0 ]; do
    case "$1" in
        --tasks-dir)  TASKS_DIR="${2:-}"; shift 2 ;;
        --allowlist)  ALLOWLIST="${2:-}"; shift 2 ;;
        --json)       JSON=true; shift ;;
        --quiet)      QUIET=true; shift ;;
        --no-heartbeat) HEARTBEAT=false; shift ;;
        -h|--help)
            sed -n '3,40p' "$0"; exit 0 ;;
        *) die2 "unknown flag: $1" ;;
    esac
done

command -v python3 >/dev/null 2>&1 || die2 "python3 not found"
[ -d "$TASKS_DIR" ] || die2 "tasks dir not found: $TASKS_DIR"
[ -d "$TASKS_DIR/active" ] || die2 "no active/ under $TASKS_DIR"

if [ -z "$ALLOWLIST" ]; then
    ALLOWLIST="$ALLOWLIST_DEFAULT"
fi
# An unreadable allowlist is a tooling error, never a silent "nothing acknowledged".
if [ -e "$ALLOWLIST" ] && [ ! -r "$ALLOWLIST" ]; then
    die2 "allowlist exists but is not readable: $ALLOWLIST"
fi

export _TASKS_DIR="$TASKS_DIR" _ALLOWLIST="$ALLOWLIST" _JSON="$JSON" _QUIET="$QUIET"

python3 - <<'PY'
import json, os, re, sys, glob

tasks_dir = os.environ["_TASKS_DIR"]
allowlist_path = os.environ["_ALLOWLIST"]
as_json = os.environ["_JSON"] == "true"
quiet = os.environ["_QUIET"] == "true"

active_dir = os.path.join(tasks_dir, "active")
files = sorted(glob.glob(os.path.join(active_dir, "*.md")))

# A corpus with zero task files is a tooling error, never a vacuous clean.
# (T-2747 lesson: "0 agrees with 0" is vacuously true and hides a broken scan.)
if not files:
    print("check-stranded-finalized-tasks: no task files under %s "
          "(fail-closed: exit 2)" % active_dir, file=sys.stderr)
    sys.exit(2)

acked = {}
if os.path.exists(allowlist_path):
    with open(allowlist_path, "r", encoding="utf-8", errors="replace") as fh:
        for raw in fh:
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            entry = re.split(r"(?:^|\s)#", line, maxsplit=1)
            sig = entry[0].strip()
            reason = entry[1].strip() if len(entry) > 1 else ""
            if sig:
                acked[sig] = reason

FM_STATUS = re.compile(r"^status:\s*(\S+)", re.M)
FM_DATE = re.compile(r"^date_finished:\s*(\S+)", re.M)

firing, acknowledged, partial_complete = [], [], []

for path in files:
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            head = fh.read(4096)
    except OSError as e:
        print("check-stranded-finalized-tasks: cannot read %s: %s "
              "(fail-closed: exit 2)" % (path, e), file=sys.stderr)
        sys.exit(2)

    # Frontmatter only — a `status:` mentioned in the body is prose, not state.
    if head.startswith("---"):
        end = head.find("\n---", 3)
        fm = head[:end] if end != -1 else head
    else:
        fm = head

    m = FM_STATUS.search(fm)
    if not m or m.group(1) != "work-completed":
        continue

    d = FM_DATE.search(fm)
    date_finished = d.group(1) if d else "absent"
    tid = os.path.basename(path).split("-")[0:2]
    tid = "-".join(tid) if len(tid) == 2 else os.path.basename(path)

    rec = {
        "task": tid,
        "file": os.path.relpath(path),
        "status": "work-completed",
        "date_finished": date_finished,
    }

    # DISCRIMINATOR — this is the whole precision of the check.
    #
    # `work-completed` sitting in active/ is NOT by itself a defect. T-193
    # partial-complete is exactly that state by design: the agent's ACs pass,
    # human ACs remain, so the task stays in active/ with owner: human awaiting
    # human verification. Measured here: 58 such tasks, every one owner: human.
    # Firing on them would make this check permanently red and therefore unread.
    #
    # What separates the defect is `date_finished`. The finalize block stamps it
    # (line ~1904) BEFORE it branches on PARTIAL_COMPLETE, so a partial-complete
    # task always carries a date. The latch skips that block wholesale, and the
    # partial-complete re-check branch that later moves the file never stamps it
    # either — so once latched, the field is unreachable by any code path and
    # stays null. Null date + work-completed is the state no healthy path
    # produces.
    #
    # Keyed on date_finished rather than `owner: human` deliberately: owner is a
    # convention that a future task could legitimately vary, while the date is
    # produced by the code path itself.
    if date_finished not in ("null", "absent"):
        partial_complete.append(rec)
        continue

    # An allowlist signature may be written as the repo-relative path (the normal
    # case), the absolute path, or the bare filename. relpath() alone is not
    # enough: with --tasks-dir pointing outside cwd it renders as a stack of
    # `../`, which no human would ever type into an allowlist. Match any form.
    candidates = {
        rec["file"],
        os.path.abspath(path),
        path,
        os.path.basename(path),
    }
    hit = next((c for c in candidates if c in acked), None)
    if hit is not None:
        rec["reason"] = acked[hit]
        acknowledged.append(rec)
    else:
        firing.append(rec)

SCOPE = ("detects tasks in .tasks/active/ that declare status: work-completed "
         "with date_finished null/absent; T-193 partial-complete tasks (which "
         "carry a date and stay in active/ by design) are counted, never fired "
         "on; does NOT audit whether work is done, nor how tasks in completed/ "
         "finalized (that is T-2290, whose --strict covers empty date_finished)")

if as_json:
    print(json.dumps({
        "ok": not firing,
        "checked": len(files),
        "firing_count": len(firing),
        "firing": firing,
        "acknowledged_count": len(acknowledged),
        "acknowledged": acknowledged,
        "partial_complete_count": len(partial_complete),
        "scope": SCOPE,
    }, indent=2))
    sys.exit(1 if firing else 0)

if firing:
    print("check-stranded-finalized-tasks: FIRING — %d task(s) declare "
          "work-completed in %s with no date_finished" % (len(firing), active_dir))
    for r in firing:
        print("  %s  date_finished=%s" % (r["file"], r["date_finished"]))
    print("")
    print("  This state DEADLOCKS commits: P-002 refuses a commit while focus is on")
    print("  a completed task, and the T-1730 focus-drift gate refuses one while")
    print("  focus is on any other task. The T-2290 canary is structurally blind to")
    print("  it — that canary scans completed/ only.")
    print("")
    print("  Fix: re-run `fw task update <id> --status work-completed`. The")
    print("  partial-complete re-check branch moves the file even though the")
    print("  finalize block is latched out. It does NOT stamp date_finished —")
    print("  that field is unreachable once the status write has landed.")
    print("")
    print("  scope: %s" % SCOPE)
elif not quiet:
    print("check-stranded-finalized-tasks: healthy — %d active task(s), "
          "0 stranded, %d acknowledged, %d partial-complete (by design)"
          % (len(files), len(acknowledged), len(partial_complete)))
    print("  scope: %s" % SCOPE)

sys.exit(1 if firing else 0)
PY
rc=$?

if [ "$HEARTBEAT" = true ]; then
    hb=".context/working/.stranded-finalized-canary.log.heartbeat"
    mkdir -p "$(dirname "$hb")" 2>/dev/null || true
    date -u +%FT%TZ > "$hb" 2>/dev/null || true
fi

exit $rc
