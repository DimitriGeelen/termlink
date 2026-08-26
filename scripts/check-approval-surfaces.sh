#!/usr/bin/env bash
# check-approval-surfaces.sh — every pending operator action, on BOTH surfaces.
#
# Deliberately NOT a '# guard-layer:' member: it needs a live Watchtower to confirm a
# route answers, so it belongs with the runtime canaries that run from cron, not with
# the source-level static checks.
#
# WHY IT EXISTS. I spent three rounds naming the hv-hc quadrant and carrying none of it,
# and part of that was not attention — it was instrumentation. My previous check equated
# "operator action" with "unchecked Human AC" and only ever looked at /review/<id>. That
# is correct for BUILD tasks and blind for INCEPTION tasks, whose operator action is a
# go/no-go DECISION surfaced at /inception/<id>. Five hv-hc tasks were sitting in a
# quadrant I kept naming and my instrument could not express the question.
#
# Same shape as everything else in this arc: a check whose population cannot exercise
# the thing being asked returns the same answer as a healthy system.
#
# WHAT IT FIRES ON, and why this predicate and not a louder one. It does NOT fire merely
# because work is waiting on a human — that is the normal, healthy state and an alarm
# that always fires stops meaning anything. It fires on exactly one condition:
#
#     a task HAS a pending operator action but would open to a card
#     the operator cannot act on
#
# i.e. Human ACs pending with no substantive ## Recommendation (lib/review.sh:205-211
# blocks emission for this at partial-complete, but not earlier), or a recorded
# inception decision with no .reviewed marker to unblock `fw inception decide`.
# Rare, always actionable, and it caught T-2015 on the first run.
#
# PREDICATES ARE THE FRAMEWORK'S OWN wherever one exists. A check that decides whether a
# human SEES something must not re-implement the rule the system uses — mine failed both
# ways in one sitting when it did (too loose: printed four links opening to a blank form;
# too strict: withheld two that were fine).
#
# Exit: 0 nothing broken (pending items are listed, that is not a failure)
#       1 at least one pending action would open to an unusable card
#       2 could not run — never rendered as healthy
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 1

BASE="${WATCHTOWER_URL:-}"
if [ -z "$BASE" ] && [ -r .context/working/watchtower.url ]; then
  BASE="$(cat .context/working/watchtower.url)"
fi
BASE="${BASE:-http://192.168.10.107:3099}"

command -v python3 >/dev/null 2>&1 || { echo "check-approval-surfaces: no python3 — no verdict"; exit 2; }
[ -r .agentic-framework/web/shared.py ] || { echo "check-approval-surfaces: shared.py unreadable — no verdict"; exit 2; }

if [ "${1:-}" = "--self-test" ]; then
  # The classifier must not collapse "pending and usable" into "pending and broken",
  # and must not report a task with nothing pending at all. Calls the same function
  # the live path calls — a self-test that re-implements its subject proves only that
  # the copy works (learned the hard way on check-receiver-ack-lag).
  python3 - "$BASE" <<'PY'
import sys
sys.path.insert(0, "scripts/lib")
from approval_surfaces import classify   # noqa: E402
fail = 0
cases = [
    # (surface, pending, gate_ok, http, expected)
    ("build",     1, True,  200, "PENDING"),
    ("build",     1, False, 200, "BROKEN"),    # pending but blank card
    ("build",     0, True,  200, "NONE"),
    # DISCRIMINATING INPUTS. The four obvious cases all pin values where "pending first"
    # and "gate first" AGREE, because every not-pending case above has a SATISFIED gate —
    # so the gate branch never gets a chance to win. These two are the only inputs that
    # pin the ORDER rather than the presence of the branches. Added after a faithful
    # mutation (gate tested before pending) passed the whole suite; a task nobody is being
    # asked about must stay NONE even when its gate is unsatisfied, or this check fires on
    # every task in the repo and stops meaning anything.
    ("build",     0, False, 200, "NONE"),
    ("inception", 0, False, 200, "NONE"),
    ("owner-completion", 1, True,  200, "PENDING"),
    ("owner-completion", 1, True,  500, "BROKEN"),
    ("owner-completion", 0, False, 200, "NONE"),
    ("build",     1, True,  500, "BROKEN"),    # route does not answer
    ("inception", 1, True,  200, "PENDING"),
    ("inception", 1, False, 200, "BROKEN"),    # decision recorded, no marker
    ("inception", 0, True,  200, "NONE"),
]
for surface, pending, gate, code, want in cases:
    got = classify(surface, pending, gate, code)
    if got != want:
        print(f"self-test: FAIL {surface} pending={pending} gate={gate} http={code} -> {got}, want {want}")
        fail = 1

# defer_state is a SECOND predicate and needs its own arms. Shipping new branching
# logic with nothing that can go red is the exact failure this file documents.
from approval_surfaces import defer_state  # noqa: E402
import datetime as _d  # noqa: E402
_past  = (_d.date.today() - _d.timedelta(days=5)).isoformat()
_fut   = (_d.date.today() + _d.timedelta(days=5)).isoformat()
_today = _d.date.today().isoformat()
# Fixtures are whole task DOCUMENTS, not bare key lines: defer_state reads FRONTMATTER
# only, and a fixture without a `---` block cannot exercise that. The bare-string version
# passed until the frontmatter fix landed and then failed correctly — the suite discriminated
# between "reads the whole file" and "reads the frontmatter", which is the distinction that
# cost T-2090 a documented decision.
def _doc(front="", body=""):
    return f"---\nid: T-TEST\nworkflow_type: inception\n{front}---\n\n**Decision**: DEFER\n{body}"

for body, want, msg in [
    (_doc(f"revisit_at: {_fut}\n"),   False, "future date must NOT be pending"),
    (_doc(f"revisit_at: {_past}\n"),  True,  "overdue date MUST be pending"),
    # the boundary: due TODAY is due, not tomorrow's problem
    (_doc(f"revisit_at: {_today}\n"), True,  "due today must be pending"),
    (_doc(),                          True,  "no revisit_at must be pending"),
    # an unparseable FRONTMATTER value must be pending, not silently skipped
    (_doc("revisit_at: Not\n"),       True,  "unparseable must be pending"),
    # T-2090's real shape: the key appears only in PROSE, where it is a sentence and not
    # config. Must be read as "no frontmatter revisit_at", never as the value "Not".
    # T-2090's REAL shape, and the fixture matters: it has the key in prose THREE times,
    # once as `**revisit_at:**` (bold, which `^revisit_at:` never matches anyway) and twice
    # BARE at column 0. Only the bare form reproduces the bug. My first fixture used the
    # bold one, the whole-document mutation survived it, and the leg proved nothing —
    # a partial reproduction is not a control, which is the lesson this file keeps relearning.
    (_doc(body="revisit_at: Not set (no calendar trigger — consumer-demand-based).\n"),
                                      True,  "bare prose mention must not be read as config"),
    # THE DISCRIMINATING INPUT, and it took three tries to find. The two candidate readings
    # (whole-document vs frontmatter-only) AGREE on `pending` for every case above — an
    # unparseable prose value and an absent one are both pending, differing only in label.
    # They disagree on exactly one shape: a prose line carrying a VALID FUTURE date with no
    # such key in the frontmatter. Whole-document reads it and calls the task parked;
    # frontmatter-only correctly says there is no return path. Nothing else pins the scope.
    (_doc(body=f"revisit_at: {_fut} is when we might look again.\n"),
                                      True,  "a future date in PROSE must not park the task"),
]:
    _label, got = defer_state(body)
    if got != want:
        print(f"self-test: FAIL defer_state — {msg} (got pending={got})")
        fail = 1

if fail:
    sys.exit(2)
print("self-test: PASS — pending-and-usable, pending-but-unusable and nothing-pending are three distinct verdicts")
PY
  exit $?
fi

python3 - "$BASE" <<'PY'
import sys
sys.path.insert(0, "scripts/lib")
from approval_surfaces import scan, classify  # noqa: E402

base = sys.argv[1]
rows = scan(base)
if rows is None:
    print("check-approval-surfaces: could not scan tasks — NO VERDICT")
    sys.exit(2)

print(f"check-approval-surfaces: pending operator actions on both surfaces ({base})")
print("  PREDICATE: build -> count_unchecked_human_acs (web/shared.py) + a substantive")
print("             ## Recommendation (audit_inception_recommendation, lib/task-audit.sh).")
print("             inception -> a recorded ## Decision + the .reviewed marker that")
print("             unblocks `fw inception decide` (T-973 gate). Both -> route answers.")
print("  FIRES ONLY when a pending action would open to a card nobody can act on.")
print()

rc = 0
for kind, path in (("build", "review"), ("inception", "inception"),
                   ("owner-completion", "review")):
    sel = [r for r in rows if r["surface"] == kind]
    ready = [r for r in sel if classify(kind, r["pending"], r["gate"], r["http"]) == "PENDING"]
    broke = [r for r in sel if classify(kind, r["pending"], r["gate"], r["http"]) == "BROKEN"]
    print(f"  {kind} — {len(ready)} pending, {len(broke)} unusable")
    for r in ready:
        print(f"    ready     {r['id']:<8} {r['why']:<22} {base}/{path}/{r['id']}")
    for r in broke:
        print(f"    UNUSABLE  {r['id']:<8} {r['blocker']}")
        rc = 1
    print()

if rc == 0:
    print("  every pending operator action opens to a card the operator can act on.")
sys.exit(rc)
PY
