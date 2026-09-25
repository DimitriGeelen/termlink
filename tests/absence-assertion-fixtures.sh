#!/usr/bin/env bash
# tests/absence-assertion-fixtures.sh — T-3148 fixtures for check-absence-assertion.sh.
#
# The check says: a `## Verification` leg asserting something is ABSENT must first
# establish the search could have succeeded, or it reports green over a check that never
# ran.
#
# The FALSE-POSITIVE guards are the load-bearing half of this suite, not the firing case.
# The T-3144 census measured 41 of 71 real absence assertions as already CORRECT — authors
# get this right more often than not — so a check that cannot recognise the correct shape
# would be red against a majority of good code on day one, and would be switched off within
# a week. Cases 3-5 pin each of the three legitimate companions; case 10's mutant proves
# that recognition is real and not decorative.
#
# Host-independent (PL-213): builds its own fixture tasks dir, touches nothing real.
#
# Usage: bash tests/absence-assertion-fixtures.sh
# Exit:  0 = all pass, 1 = a fixture regressed, 2 = harness problem.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/check-absence-assertion.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  PASS  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL  %s\n' "$1" >&2; [ -n "${2:-}" ] && printf '          %s\n' "$2" >&2; }

[ -r "$SCRIPT" ] || { echo "absence-assertion-fixtures: cannot read $SCRIPT" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "absence-assertion-fixtures: python3 missing" >&2; exit 2; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
TASKS="$SCRATCH/active"
EMPTY_ACK="$SCRATCH/ack-empty"
mkdir -p "$TASKS"
: > "$EMPTY_ACK"

# Each fixture task is a minimal file carrying only a Verification block.
mktask() {
    local name="$1"; shift
    { printf -- '---\nid: %s\n---\n\n## Verification\n\n' "$name"
      printf '%s\n' "$@"
    } > "$TASKS/$name.md"
}

run() { bash "$SCRIPT" --tasks-dir "$TASKS" --allowlist "$EMPTY_ACK" --no-heartbeat "$@" 2>&1; }

echo "T-3148 absence-assertion fixtures"
echo

# --- 1. naked negated grep FIRES --------------------------------------------
rm -f "$TASKS"/*.md
mktask T-001 '! grep -q "BADWORD" src/thing.sh'
out=$(run); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "VACUOUS-RISK" && printf '%s' "$out" | grep -q "T-001"; then
    ok "naked '! grep -q PATTERN FILE' fires"
else
    bad "naked negated grep fires" "exit $rc; out: $out"
fi

# --- 2. count-equals-zero over a COMMAND fires ------------------------------
# The shape that goes green precisely when the command could not run.
rm -f "$TASKS"/*.md
mktask T-002 '[ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]'
out=$(run); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "T-002"; then
    ok "count-equals-zero over a command's output fires"
else
    bad "count-equals-zero fires" "exit $rc; out: $out"
fi

# --- 3. FALSE-POSITIVE GUARD: existence test on the same line ---------------
rm -f "$TASKS"/*.md
mktask T-003 'test -f src/thing.sh && ! grep -q "BADWORD" src/thing.sh'
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "existence test on the same line clears it"
else
    bad "same-line existence test must clear" "exit $rc; out: $out"
fi

# --- 4. FALSE-POSITIVE GUARD: existence test elsewhere in the block ---------
rm -f "$TASKS"/*.md
mktask T-004 '! grep -q "BADWORD" src/thing.sh' 'test -f src/thing.sh'
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "existence test elsewhere in the same block clears it"
else
    bad "block-level existence test must clear" "exit $rc; out: $out"
fi

# --- 5. FALSE-POSITIVE GUARD: positive companion grep on the same file ------
rm -f "$TASKS"/*.md
mktask T-005 'grep -q "fn known_marker" src/thing.rs' '! grep -q "BADWORD" src/thing.rs'
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "positive grep on the same file clears it"
else
    bad "positive companion must clear" "exit $rc; out: $out"
fi

# --- 6. FALSE-POSITIVE GUARD: &&-joined producer ----------------------------
rm -f "$TASKS"/*.md
mktask T-006 'cargo clippy --workspace > /tmp/.out 2>&1 && ! grep -q "^error" /tmp/.out'
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "&&-joined producer clears it (producer's exit code carries the failure)"
else
    bad "&&-joined producer must clear" "exit $rc; out: $out"
fi

# --- 7. a leg must NOT clear itself -----------------------------------------
# `! grep -q P f` contains the substring `grep -q ... f`. If the positive-companion rule
# matched its own line, EVERY naked assertion would clear and the check would be inert —
# green on a corpus full of the defect, which is the defect.
rm -f "$TASKS"/*.md
mktask T-007 '! grep -q "BADWORD" src/only.sh'
out=$(run); rc=$?
if [ "$rc" -eq 1 ]; then
    ok "a negated leg does not satisfy its own positive-companion rule"
else
    bad "leg must not clear itself" "exit $rc; out: $out"
fi

# --- 8. comments and prose are ignored --------------------------------------
rm -f "$TASKS"/*.md
mktask T-008 '# ! grep -q "BADWORD" src/thing.sh' 'test -f README.md'
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "a commented-out assertion is not a candidate"
else
    bad "comments must be ignored" "exit $rc; out: $out"
fi

# --- 9. FAIL-CLOSED: empty corpus is exit 2, never a clean census -----------
# "0 of 0 are wrong" is vacuously true. A scan that silently stopped matching must not
# read as a clean bill (T-2747).
EMPTYDIR="$SCRATCH/no-tasks"; mkdir -p "$EMPTYDIR"
out=$(bash "$SCRIPT" --tasks-dir "$EMPTYDIR" --allowlist "$EMPTY_ACK" --no-heartbeat 2>&1); rc=$?
if [ "$rc" -eq 2 ]; then
    ok "empty corpus exits 2 (fail-closed), never a vacuous clean"
else
    bad "empty corpus must be tooling, not clean" "exit $rc; out: $out"
fi

# --- 9b. FAIL-CLOSED: missing tasks dir is exit 2 ---------------------------
out=$(bash "$SCRIPT" --tasks-dir "$SCRATCH/does-not-exist" --allowlist "$EMPTY_ACK" --no-heartbeat 2>&1); rc=$?
if [ "$rc" -eq 2 ]; then
    ok "missing tasks dir exits 2 (fail-closed)"
else
    bad "missing tasks dir must be tooling" "exit $rc; out: $out"
fi

# --- 10. MUTANT: breaking companion-detection reddens the guards ------------
# A suite that cannot go red is not a suite. Strip the companion rules and the CORRECT
# shapes must start firing — which is what proves cases 3-6 are load-bearing rather than
# passing for some unrelated reason.
rm -f "$TASKS"/*.md
mktask T-010 'test -f src/thing.sh && ! grep -q "BADWORD" src/thing.sh'
MUT="$SCRATCH/mutant.sh"
# Disable the clearing branch outright: every candidate becomes a finding, including the
# CORRECT one in this fixture. If cases 3-6 passed only because the check never fires at
# all, this mutant would stay green and expose that.
sed 's|^            if guarded:|            if False:|' "$SCRIPT" > "$MUT"
if ! cmp -s "$SCRIPT" "$MUT"; then
    mout=$(bash "$MUT" --tasks-dir "$TASKS" --allowlist "$EMPTY_ACK" --no-heartbeat 2>&1); mrc=$?
    if [ "$mrc" -eq 1 ]; then
        ok "mutant with companion-detection disabled fires on a CORRECT leg — the guards are load-bearing"
    else
        bad "mutant should fire on the correct shape" "exit $mrc; out: $mout"
    fi
else
    bad "mutant identical to script" "the sed anchors did not match — update them"
fi

# --- 11. allowlist: an acknowledged finding does not fire, but IS reported --
rm -f "$TASKS"/*.md
mktask T-011 '! grep -q "BADWORD" src/thing.sh'
ACK="$SCRATCH/ack"
printf 'T-011::! grep -q "BADWORD" src/thing.sh  # acknowledged for the fixture\n' > "$ACK"
out=$(bash "$SCRIPT" --tasks-dir "$TASKS" --allowlist "$ACK" --no-heartbeat 2>&1); rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q "ACK" && ! printf '%s' "$out" | grep -q "VACUOUS-RISK"; then
    ok "acknowledged finding clears the exit code but is still named in the output"
else
    bad "acknowledged finding must be reported, not muted" "exit $rc; out: $out"
fi

# --- 12. removing the acknowledgement re-fires ------------------------------
printf '# nothing acknowledged\n' > "$ACK"
out=$(bash "$SCRIPT" --tasks-dir "$TASKS" --allowlist "$ACK" --no-heartbeat 2>&1); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "VACUOUS-RISK"; then
    ok "removing the acknowledgement re-fires the finding"
else
    bad "ledger must not be a one-way door" "exit $rc; out: $out"
fi

# --- 13. an ABSENT ledger acknowledges nothing ------------------------------
out=$(bash "$SCRIPT" --tasks-dir "$TASKS" --allowlist "$SCRATCH/no-such-ledger" --no-heartbeat 2>&1); rc=$?
if [ "$rc" -eq 1 ]; then
    ok "an absent ledger acknowledges nothing rather than excusing everything"
else
    bad "absent ledger must not fail open" "exit $rc; out: $out"
fi

# --- 14. --json carries counts, the acknowledged set, and the scope ---------
out=$(bash "$SCRIPT" --tasks-dir "$TASKS" --allowlist "$EMPTY_ACK" --no-heartbeat --json 2>&1)
if printf '%s' "$out" | python3 -c '
import sys, json
d = json.load(sys.stdin)
assert d["ok"] is False, d
assert d["firing_count"] == 1, d
assert d["acknowledged_count"] == 0, d
assert "scope" in d and "ACTIVE tasks only" in d["scope"], d.get("scope")
' 2>/dev/null; then
    ok "--json carries ok/firing_count/acknowledged_count and an explicit scope string"
else
    bad "--json envelope" "got: $out"
fi

# --- 15. the clean path states its scope ------------------------------------
rm -f "$TASKS"/*.md
mktask T-015 'test -f src/thing.sh && ! grep -q "BADWORD" src/thing.sh'
out=$(run); rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q "Scope:" && printf '%s' "$out" | grep -q "does NOT audit"; then
    ok "a green states what it does not cover (T-2680)"
else
    bad "clean path must carry the scope disclaimer" "exit $rc; out: $out"
fi

echo
echo "absence-assertion-fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
