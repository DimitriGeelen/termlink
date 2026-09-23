#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# T-3083 — an arc's SLICE REGISTER goes stale and nothing detects it.
#
# WHY THIS EXISTS
# ---------------
# arc-011 carries a slice register (T-3066) whose stated purpose is that "a slice with
# no task is now a structural query, not an archaeology exercise". It was built because
# arc-003 closed asserting a capability that was disproved 82 days later, and because a
# deferred slice with no field to live in survived only as the word "future" inside a
# task that completed, and vanished.
#
# But the register's `status:` is HAND-MAINTAINED. A slice goes stale the moment its
# task completes and nobody returns to it, and that was caught by hand THREE TIMES IN
# TWO DAYS on the one arc that has a register at all:
#   * S6  read `unbuilt` after T-3070 shipped the sender ledger.
#   * S11 read "Crontab written, NOT installed" a full day after it was installed and
#         verified firing in journalctl.
#   * S7/S10 read `unbuilt` after the rail was proven end to end.
#
# The drift UNDER-claims, which is the expensive direction: a register saying `unbuilt`
# about shipped work invites the work to be done twice (the T-2800 duplicate class by
# another route). And it is silent — check-arc-claim-drift.sh passes over it cleanly,
# correctly, because that guard judges CLOSED arcs for prover bindings and says so in
# its own scope line. It was never asked this question.
#
# THE BOUNDARY BETWEEN THE TWO GUARDS, so they are not mistaken for each other:
#   check-arc-claim-drift.sh  — CLOSED arcs: is the capability claim bound to a runnable
#                               prover, and does that prover still pass?
#   check-arc-slice-drift.sh  — IN-PROGRESS arcs: is any slice's status stale against
#                               the location of the task it is bound to?
#
# SCOPE — read a green narrowly (T-2680). This answers ONE question: does any slice
# whose task has COMPLETED still read `unbuilt`, and does every `task:` resolve? It does
# NOT judge whether a slice's note is accurate, whether `partial` is the right call for
# a given slice, or whether the register describes the right slices at all. A clean run
# is not a statement that the register is correct.
#
# Exit: 0 clean · 1 drift · 2 tooling (FAIL-CLOSED — an unparseable arc, an arc with
# zero slices, or a missing python3 exits 2, never a vacuous pass).
set -u

ARCS_DIR="${ARC_SLICE_ARCS_DIR:-.context/arcs}"
TASKS_DIR="${ARC_SLICE_TASKS_DIR:-.tasks}"
ALLOWLIST="${ARC_SLICE_ALLOWLIST:-.context/checks/arc-slice-drift-allowlist}"
JSON=0; QUIET=0

usage() {
    cat <<'USAGE'
Usage: check-arc-slice-drift.sh [options]

Detects a slice whose status is STALE against the location of its bound task:
an IN-PROGRESS arc still calling a slice `unbuilt` after that task completed.

Options:
  --arcs-dir DIR    arc registers (default .context/arcs)
  --tasks-dir DIR   task tree (default .tasks)
  --allowlist PATH  acknowledged entries (default .context/checks/arc-slice-drift-allowlist)
  --json            machine-readable
  --quiet           print only when something fires
  --no-heartbeat    accepted and ignored (guard-layer runner compatibility)
  --help            this text

Exit: 0 clean · 1 drift · 2 tooling (fail-closed)
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --arcs-dir)     ARCS_DIR="${2:-}"; shift 2 ;;
        --tasks-dir)    TASKS_DIR="${2:-}"; shift 2 ;;
        --allowlist)    ALLOWLIST="${2:-}"; shift 2 ;;
        --json)         JSON=1; shift ;;
        --quiet)        QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        --help|-h)      usage; exit 0 ;;
        *) echo "check-arc-slice-drift: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

command -v python3 >/dev/null 2>&1 || {
    echo "check-arc-slice-drift: python3 not available — cannot read the registers" >&2
    exit 2   # fail-closed: not looking is never 'clean'
}
[ -d "$ARCS_DIR" ] || { echo "check-arc-slice-drift: no arcs dir at $ARCS_DIR" >&2; exit 2; }

ARC_SLICE_ARCS_DIR="$ARCS_DIR" ARC_SLICE_TASKS_DIR="$TASKS_DIR" \
ARC_SLICE_ALLOWLIST="$ALLOWLIST" ARC_SLICE_JSON="$JSON" ARC_SLICE_QUIET="$QUIET" \
python3 - <<'PY'
import glob, json, os, sys

arcs_dir = os.environ["ARC_SLICE_ARCS_DIR"]
tasks_dir = os.environ["ARC_SLICE_TASKS_DIR"]
allow_path = os.environ["ARC_SLICE_ALLOWLIST"]
as_json = os.environ["ARC_SLICE_JSON"] == "1"
quiet = os.environ["ARC_SLICE_QUIET"] == "1"

SCOPE = ("detects a slice STATUS stale against its task's LOCATION, on in-progress arcs. "
         "It cannot see a stale NOTE: arc-011 S11 read 'Crontab written, NOT installed' "
         "for a day after it was installed, while status/location agreed, and this check "
         "would have passed it. It also does not judge whether 'partial' is the right "
         "call or whether the register lists the right slices.")

try:
    import yaml
except Exception as e:                                   # fail-closed
    sys.stderr.write("check-arc-slice-drift: PyYAML unavailable (%s)\n" % e)
    sys.exit(2)

allowed = {}
if os.path.isfile(allow_path):
    try:
        for line in open(allow_path, encoding="utf-8"):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            sig, _, reason = line.partition("#")
            allowed[sig.strip()] = reason.strip()
    except Exception as e:
        sys.stderr.write("check-arc-slice-drift: allowlist unreadable (%s)\n" % e)
        sys.exit(2)

files = sorted(glob.glob(os.path.join(arcs_dir, "*.yaml")))
if not files:
    sys.stderr.write("check-arc-slice-drift: no arc files under %s\n" % arcs_dir)
    sys.exit(2)

def task_located(tid):
    """'completed' | 'active' | None — where the task file lives."""
    if glob.glob(os.path.join(tasks_dir, "completed", "%s-*.md" % tid)):
        return "completed"
    if glob.glob(os.path.join(tasks_dir, "active", "%s-*.md" % tid)):
        return "active"
    return None

firing, acknowledged = [], []
slices_checked = arcs_checked = 0

for f in files:
    try:
        doc = yaml.safe_load(open(f, encoding="utf-8"))
    except Exception as e:
        sys.stderr.write("check-arc-slice-drift: %s does not parse (%s)\n" % (f, e))
        sys.exit(2)                                      # fail-closed
    if not isinstance(doc, dict):
        continue
    slices = doc.get("slices")
    if slices is None:
        continue                                          # arcs without a register
    if not isinstance(slices, list) or not slices:
        # A register that exists but is empty asserts nothing while looking like it
        # does. That is the vacuous-pass shape (T-2831), so it is a tooling error.
        sys.stderr.write("check-arc-slice-drift: %s has a slices: key with no slices\n" % f)
        sys.exit(2)
    if str(doc.get("status", "")).strip() != "in-progress":
        continue                                          # closed arcs: T-2483 territory

    arcs_checked += 1
    slug = doc.get("slug") or os.path.basename(f)[:-5]
    for s in slices:
        if not isinstance(s, dict):
            continue
        slices_checked += 1
        sid = str(s.get("id", "?"))
        tid = str(s.get("task", "")).strip()
        status = str(s.get("status", "")).strip()
        sig = "%s::%s" % (slug, sid)
        why = None
        if not tid:
            why = "slice has no task: binding"
        else:
            where = task_located(tid)
            if where is None:
                why = "task %s resolves to no file" % tid
            elif where == "completed" and status == "unbuilt":
                why = "task %s is COMPLETED but the slice still reads unbuilt" % tid
        if why:
            entry = {"arc": slug, "slice": sid, "task": tid, "status": status, "why": why,
                     "signature": sig}
            if sig in allowed:
                entry["reason"] = allowed[sig]
                acknowledged.append(entry)
            else:
                firing.append(entry)

ok = not firing
if as_json:
    print(json.dumps({"ok": ok, "firing": firing, "firing_count": len(firing),
                      "acknowledged": acknowledged, "acknowledged_count": len(acknowledged),
                      "arcs_checked": arcs_checked, "slices_checked": slices_checked,
                      "scope": SCOPE}, indent=2))
elif firing:
    print("check-arc-slice-drift: %d slice(s) stale against their task" % len(firing))
    for e in firing:
        print("  %-14s %-5s %s" % (e["arc"], e["slice"], e["why"]))
    print("")
    print("  SCOPE: %s" % SCOPE)
    print("")
    print("Remediation: correct the slice status to match what shipped, or acknowledge")
    print("it in %s with a cited reason." % allow_path)
elif not quiet:
    print("check-arc-slice-drift: clean — %d slice(s) across %d in-progress arc(s), "
          "%d acknowledged" % (slices_checked, arcs_checked, len(acknowledged)))
    print("  SCOPE: %s" % SCOPE)

sys.exit(1 if firing else 0)
PY
