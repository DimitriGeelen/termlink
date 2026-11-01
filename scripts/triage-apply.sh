#!/usr/bin/env bash
# T-3649 — apply the operator's Watchtower /triage answers to tasks.
#
# The /triage page (vendored web/blueprints/triage.py, local divergence) records
# each click as {ts, batch, task, choice} in .context/triage/answers.jsonl; the
# last line per task wins. The page never touches a task. This script is the
# agent-side step that turns answers into task changes, through the normal gates:
#
#   prioritize -> fw task update T --horizon now
#   keep       -> no change
#   park       -> fw task update T --horizon later
#   drop       -> NOT deleted, NOT closed: appended to drop-pending.jsonl for the
#                 operator to confirm a close (the agent then closes it normally)
#   close      -> fw task update T --status work-completed (never --force); a gate
#                 refusal is reported SKIPPED with its reason and retried next run
#   more       -> appended to more-pending.jsonl: the agent owes an explanation
#
# Dry-run is the default (prints the plan). --apply acts. Every applied answer is
# recorded in applied.jsonl as {ts, batch, task, choice, result}; an answer whose
# (batch, task, choice) is already recorded is skipped, so a rerun is idempotent.
# A changed answer (the operator clicked again) is applied again.
#
# Usage:  bash scripts/triage-apply.sh [--apply] [--json]
# Exit:   0 = nothing failed (SKIPPED gate refusals included), 1 = an action
#         FAILED, 2 = tooling (cannot read the batch/answers, fw missing).
# Seams (PL-213): TRIAGE_DIR (default <root>/.context/triage),
#   TRIAGE_FW (default <root>/.agentic-framework/bin/fw),
#   TRIAGE_TASKS_DIR (default <root>/.tasks; used only to see already-closed tasks).
# Fixtures: bash tests/triage-apply-fixtures.sh

set -uo pipefail

APPLY=0
JSON=0
while [ $# -gt 0 ]; do
    case "$1" in
        --apply) APPLY=1 ;;
        --dry-run) APPLY=0 ;;
        --json) JSON=1 ;;
        -h|--help) sed -n '2,29p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "triage-apply: unknown flag: $1" >&2; exit 2 ;;
    esac
    shift
done

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export TRIAGE_DIR="${TRIAGE_DIR:-$ROOT/.context/triage}"
export TRIAGE_FW="${TRIAGE_FW:-$ROOT/.agentic-framework/bin/fw}"
export TRIAGE_TASKS_DIR="${TRIAGE_TASKS_DIR:-$ROOT/.tasks}"
export TRIAGE_APPLY="$APPLY" TRIAGE_JSON="$JSON" TRIAGE_ROOT="$ROOT"

command -v python3 >/dev/null 2>&1 || { echo "triage-apply: python3 not found (fail-closed)" >&2; exit 2; }

exec python3 - <<'PY'
import glob, json, os, subprocess, sys
from datetime import datetime, timezone

D      = os.environ["TRIAGE_DIR"]
FW     = os.environ["TRIAGE_FW"]
TASKS  = os.environ["TRIAGE_TASKS_DIR"]
APPLY  = os.environ["TRIAGE_APPLY"] == "1"
JSON   = os.environ["TRIAGE_JSON"] == "1"
ROOT   = os.environ["TRIAGE_ROOT"]
CHOICES = ("prioritize", "keep", "park", "drop", "close", "more")

def now():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

def die(msg):
    sys.stderr.write(f"triage-apply: {msg} (fail-closed)\n")
    sys.exit(2)

def read_jsonl(path):
    out = []
    if not os.path.exists(path):
        return out
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                rec = json.loads(line)
            except ValueError:
                continue
            if isinstance(rec, dict):
                out.append(rec)
    return out

def append(path, rec):
    with open(path, "a", encoding="utf-8") as f:
        f.write(json.dumps(rec, ensure_ascii=False) + "\n")

batch_path = os.path.join(D, "batch.json")
try:
    with open(batch_path, encoding="utf-8") as f:
        batch = json.load(f)
    batch_id = str(batch["batch"])
    titles = {r["task"]: r.get("title", "") for r in batch["rows"] if isinstance(r, dict) and "task" in r}
except (OSError, ValueError, KeyError, TypeError) as exc:
    die(f"cannot read {batch_path}: {exc}")

answers = {}
for rec in read_jsonl(os.path.join(D, "answers.jsonl")):
    if rec.get("batch") == batch_id and rec.get("choice") in CHOICES and rec.get("task") in titles:
        answers[rec["task"]] = rec

applied_path = os.path.join(D, "applied.jsonl")
done = {(r.get("batch"), r.get("task"), r.get("choice")) for r in read_jsonl(applied_path)}

if APPLY and not os.access(FW, os.X_OK):
    die(f"fw not executable at {FW}")

def is_completed(task):
    return bool(glob.glob(os.path.join(TASKS, "completed", f"{task}-*.md")) or
                glob.glob(os.path.join(TASKS, "completed", f"{task}.md")))

def fw(*args):
    p = subprocess.run([FW, *args], cwd=ROOT, capture_output=True, text=True)
    lines = [l.strip() for l in (p.stdout + "\n" + p.stderr).splitlines() if l.strip()]
    return p.returncode, lines

results = []
failed = 0
counts = {}

def report(task, choice, outcome, detail):
    counts[outcome] = counts.get(outcome, 0) + 1
    results.append({"task": task, "choice": choice, "outcome": outcome, "detail": detail})
    if not JSON:
        print(f"{outcome:9} {task:8} {choice:10} {detail}")

def record(task, choice, result):
    append(applied_path, {"ts": now(), "batch": batch_id, "task": task, "choice": choice, "result": result})

def task_num(t):
    try:
        return int(t.split("-", 1)[1])
    except (IndexError, ValueError):
        return 0

for task in sorted(answers, key=task_num):
    choice = answers[task]["choice"]
    if (batch_id, task, choice) in done:
        report(task, choice, "ALREADY", "applied on an earlier run")
        continue

    if choice == "keep":
        if APPLY:
            record(task, choice, "no-change")
        report(task, choice, "OK" if APPLY else "WOULD", "keep: no change")
        continue

    if choice in ("drop", "more"):
        sink = "drop-pending.jsonl" if choice == "drop" else "more-pending.jsonl"
        why = ("drop: listed for the operator to confirm a close (nothing deleted)" if choice == "drop"
               else "more: listed for the agent to explain to the operator")
        if APPLY:
            append(os.path.join(D, sink), {"ts": now(), "batch": batch_id, "task": task,
                                           "title": titles.get(task, "")})
            record(task, choice, "listed")
        report(task, choice, "OK" if APPLY else "WOULD", f"{why} -> {sink}")
        continue

    if choice in ("prioritize", "park"):
        horizon = "now" if choice == "prioritize" else "later"
        argv = ["task", "update", task, "--horizon", horizon]
    else:  # close
        if is_completed(task):
            if APPLY:
                record(task, choice, "already-completed")
            report(task, choice, "OK" if APPLY else "WOULD", "close: already completed, nothing to do")
            continue
        argv = ["task", "update", task, "--status", "work-completed"]

    if not APPLY:
        report(task, choice, "WOULD", "fw " + " ".join(argv))
        continue
    rc, lines = fw(*argv)
    if rc == 0:
        record(task, choice, "applied")
        report(task, choice, "OK", "fw " + " ".join(argv))
    elif choice == "close":
        reason = " | ".join(lines[:3]) or f"rc {rc}"
        report(task, choice, "SKIPPED", f"close refused by a gate (rc {rc}): {reason}")
    else:
        failed += 1
        reason = " | ".join(lines[:3]) or f"rc {rc}"
        report(task, choice, "FAILED", f"fw {' '.join(argv)} rc {rc}: {reason}")

unanswered = len(titles) - len(answers)
summary = {"batch": batch_id, "mode": "apply" if APPLY else "dry-run", "answers": len(answers),
           "unanswered": unanswered, "counts": counts}
if JSON:
    print(json.dumps({"summary": summary, "results": results}, indent=2))
else:
    tally = ", ".join(f"{k} {v}" for k, v in sorted(counts.items())) or "nothing to do"
    print(f"triage-apply [{summary['mode']}] batch {batch_id}: {len(answers)} answered, "
          f"{unanswered} unanswered; {tally}")
    if not APPLY and answers:
        print("Dry run: nothing changed. Re-run with --apply to act.")
sys.exit(1 if failed else 0)
PY
