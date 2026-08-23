#!/usr/bin/env bash
# tests/stranded-finalized-check-fixtures.sh (T-2833)
#
# Fixtures for scripts/check-stranded-finalized-tasks.sh — hermetic, no live
# binary, no real .tasks tree.
#
# Weighted toward the FIRING cases and the FALSE-POSITIVE guards. The detector's
# entire precision lives in one discriminator — date_finished null vs set — and
# the first draft of this check got it wrong: it fired on all 58 T-193
# partial-complete tasks in the real corpus, which would have made it
# permanently red and therefore unread. Case 2 is the regression that pins that.
set -uo pipefail

SCRIPT="${SCRIPT:-scripts/check-stranded-finalized-tasks.sh}"
[ -f "$SCRIPT" ] || { echo "fixtures: $SCRIPT not found"; exit 2; }

PASS=0
FAIL=0
TMPROOT="$(mktemp -d)"
trap 'rm -rf "$TMPROOT"' EXIT

ok() { PASS=$((PASS+1)); echo "  ok   — $1"; }
no() { FAIL=$((FAIL+1)); echo "  FAIL — $1"; }

assert_rc() { if [ "$1" = "$2" ]; then ok "$3 (rc=$2)"; else no "$3 (expected rc=$1, got rc=$2)"; fi; }
assert_contains() { case "$1" in *"$2"*) ok "$3" ;; *) no "$3 (missing: $2)" ;; esac; }
assert_not_contains() { case "$1" in *"$2"*) no "$3 (unexpectedly present: $2)" ;; *) ok "$3" ;; esac; }

# mktask <dir> <name> <status> <date_finished> [owner]
mktask() {
    mkdir -p "$1/active"
    {
        echo '---'
        echo "id: T-9999"
        echo "name: \"fixture\""
        echo "status: $3"
        echo "owner: ${5:-agent}"
        echo "date_finished: $4"
        echo '---'
        echo
        echo '## Context'
        echo 'body text'
    } > "$1/active/$2.md"
}

run() { bash "$SCRIPT" --no-heartbeat --tasks-dir "$1" "${@:2}" 2>&1; }

echo "== case 1: work-completed in active/ with null date fires (the latch state) =="
C="$TMPROOT/c1"; mktask "$C" stranded work-completed null
out="$(run "$C")"; rc=$?
assert_rc 1 "$rc" "stranded task fires"
assert_contains "$out" "FIRING" "names the firing state"
assert_contains "$out" "date_finished=null" "prints the discriminating field"
assert_contains "$out" "DEADLOCKS" "explains why it is not cosmetic"
assert_contains "$out" "T-2290" "names the canary that is blind to it"

echo "== case 2: partial-complete (date SET) never fires — the 58-FP regression =="
# Measured on the real corpus: 58 tasks are work-completed in active/ with a
# real date and owner: human. That is T-193 by design. A check that fires on
# them is permanently red, and a permanently red check is an unread check.
C="$TMPROOT/c2"; mktask "$C" partial work-completed 2026-05-04T13:19:46Z human
out="$(run "$C")"; rc=$?
assert_rc 0 "$rc" "partial-complete does not fire"
assert_contains "$out" "1 partial-complete" "counted, not hidden"
assert_contains "$out" "healthy" "reports healthy"

echo "== case 3: both together — fires on one, counts the other =="
C="$TMPROOT/c3"
mktask "$C" stranded work-completed null
mktask "$C" partial work-completed 2026-05-04T13:19:46Z human
out="$(run "$C")"; rc=$?
assert_rc 1 "$rc" "mixed corpus fires"
assert_contains "$out" "1 task(s)" "fires on exactly the stranded one"
assert_not_contains "$out" "partial.md" "does not name the partial-complete task"

echo "== case 4: ordinary in-progress tasks are ignored =="
C="$TMPROOT/c4"; mktask "$C" wip started-work null
out="$(run "$C")"; rc=$?
assert_rc 0 "$rc" "started-work is clean"

echo "== case 5: date_finished absent entirely still fires =="
C="$TMPROOT/c5"
mkdir -p "$C/active"
printf -- '---\nid: T-1\nstatus: work-completed\n---\n\nbody\n' > "$C/active/nodate.md"
out="$(run "$C")"; rc=$?
assert_rc 1 "$rc" "missing date_finished field fires"
assert_contains "$out" "date_finished=absent" "reports the field as absent, not guessed"

echo "== case 6: a 'status:' line in the BODY is prose, not state =="
C="$TMPROOT/c6"
mkdir -p "$C/active"
printf -- '---\nid: T-1\nstatus: started-work\ndate_finished: null\n---\n\n## Context\n\nstatus: work-completed\n' > "$C/active/prose.md"
out="$(run "$C")"; rc=$?
assert_rc 0 "$rc" "body mention of status does not fire (frontmatter-only)"

echo "== case 7: allowlist suppresses but still counts =="
C="$TMPROOT/c7"; mktask "$C" acked work-completed null
AL="$TMPROOT/c7-allow"
echo "$C/active/acked.md  # fixture: deliberate" > "$AL"
out="$(run "$C" --allowlist "$AL")"; rc=$?
assert_rc 0 "$rc" "allowlisted task does not fire"
assert_contains "$out" "1 acknowledged" "acknowledged entry is still counted"

echo "== case 8: --json envelope shape =="
C="$TMPROOT/c8"; mktask "$C" stranded work-completed null
out="$(run "$C" --json)"; rc=$?
assert_rc 1 "$rc" "json mode preserves the exit contract"
assert_contains "$out" '"ok": false' "json carries ok:false when firing"
assert_contains "$out" '"firing_count": 1' "json carries firing_count"
assert_contains "$out" '"partial_complete_count"' "json carries the partial-complete count"
assert_contains "$out" '"scope"' "json carries the scope disclaimer"

echo "== case 9: fail-closed on tooling errors =="
out="$(bash "$SCRIPT" --no-heartbeat --tasks-dir "$TMPROOT/nope" 2>&1)"; rc=$?
assert_rc 2 "$rc" "missing tasks dir exits 2, never a false clean"
assert_contains "$out" "fail-closed" "says it failed closed"

mkdir -p "$TMPROOT/empty/active"
out="$(bash "$SCRIPT" --no-heartbeat --tasks-dir "$TMPROOT/empty" 2>&1)"; rc=$?
assert_rc 2 "$rc" "a corpus with zero task files exits 2, never a vacuous clean"

out="$(bash "$SCRIPT" --no-heartbeat --tasks-dir "$TMPROOT/c1" --bogus 2>&1)"; rc=$?
assert_rc 2 "$rc" "unknown flag exits 2"

echo "== case 10: --quiet is silent when healthy, still speaks when firing =="
out="$(run "$TMPROOT/c2" --quiet)"; rc=$?
assert_rc 0 "$rc" "quiet healthy keeps exit 0"
assert_not_contains "$out" "healthy" "quiet suppresses the healthy line"
out="$(run "$TMPROOT/c1" --quiet)"; rc=$?
assert_contains "$out" "FIRING" "quiet still reports a firing finding"

echo
echo "=== $((PASS+FAIL)) assertion(s): $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ] || exit 1
