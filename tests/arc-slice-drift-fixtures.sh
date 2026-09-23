#!/usr/bin/env bash
# guard-layer: source
# T-3083 — fixtures for scripts/check-arc-slice-drift.sh.
#
# THE LOAD-BEARING CASE IS D1, and it is not synthetic: it runs the check against
# arc-011 exactly as it stood at commit 3f0962566, extracted from git. That is the
# tree in which I caught S6 by hand, and the check names S6, S7, S8 and S10.
# A fixture built from what I BELIEVE the drift looked like would encode the same
# assumption that let the drift survive; a fixture built from the drift itself cannot.
#
# WHAT THIS CHECK CANNOT DO, pinned here so nobody reads its green too widely (D6):
# it compares a slice's STATUS against its task's LOCATION. It cannot see a stale
# NOTE. arc-011 S11 read "Crontab written, NOT installed" for a full day after the
# crontab was installed and verified firing — and at that moment status said `unbuilt`
# while T-3068 was still active, so status and location AGREED. This check would have
# passed it, and D6 asserts exactly that rather than letting it be discovered later.
set -u

CHK="${CHK:-scripts/check-arc-slice-drift.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

command -v python3 >/dev/null 2>&1 || { echo "SKIP: python3 unavailable"; exit 0; }
python3 -c 'import yaml' 2>/dev/null || { echo "SKIP: PyYAML unavailable"; exit 0; }
[ -r "$CHK" ] || { echo "SKIP: $CHK not readable"; exit 0; }

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/arcs" "$WORK/tasks/active" "$WORK/tasks/completed"
: > "$WORK/empty-allowlist"

mkarc() { # mkarc <file> <status> <slice-id> <task> <slice-status>
    cat > "$WORK/arcs/$1" <<EOS
slug: fixarc
status: $2
slices:
  - id: $3
    spec_step: "fixture"
    task: $4
    status: $5
EOS
}
run() { ARC_SLICE_ARCS_DIR="$WORK/arcs" ARC_SLICE_TASKS_DIR="$WORK/tasks" \
        ARC_SLICE_ALLOWLIST="${1:-$WORK/empty-allowlist}" bash "$CHK" 2>&1; }
rc_of() { run "${1:-}" >/dev/null 2>&1; echo $?; }

echo "D1 [THE REAL DRIFT]: arc-011 at 3f0962566, extracted from git, not synthesised"
mkdir -p "$WORK/hist"
if git show 3f0962566:.context/arcs/arc-011.yaml > "$WORK/hist/arc-011.yaml" 2>/dev/null; then
    out="$(ARC_SLICE_ARCS_DIR="$WORK/hist" ARC_SLICE_ALLOWLIST="$WORK/empty-allowlist" bash "$CHK" 2>&1)"
    rc=$?
    if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q 'S6' && printf '%s' "$out" | grep -q 'S10'; then
        pass "D1 names the slices that were stale (rc=$rc)"
    else
        fail "D1 expected rc=1 naming S6 and S10, got rc=$rc: $out"
    fi
else
    fail "D1 could not extract the historical arc from git — the load-bearing case did not run"
fi

echo "D2: the current tree is clean"
out="$(bash "$CHK" 2>&1)"; rc=$?
[ "$rc" -eq 0 ] && pass "D2 rc=0" || fail "D2 expected 0 got $rc: $out"

echo "D3: a COMPLETED task whose slice still reads unbuilt fires"
mkarc a.yaml in-progress S1 T-9001 unbuilt
: > "$WORK/tasks/completed/T-9001-done.md"
[ "$(rc_of)" = "1" ] && pass "D3 rc=1" || fail "D3 expected 1 got $(rc_of)"

echo "D4: the same slice marked built is clean"
mkarc a.yaml in-progress S1 T-9001 built
[ "$(rc_of)" = "0" ] && pass "D4 rc=0" || fail "D4 expected 0 got $(rc_of)"

echo "D5: a CLOSED arc is out of scope (that is check-arc-claim-drift's question)"
mkarc a.yaml closed S1 T-9001 unbuilt
[ "$(rc_of)" = "0" ] && pass "D5 closed arc ignored" || fail "D5 closed arc was judged"

echo "D6 [STATED LIMIT]: a stale NOTE is invisible — status and location agree"
# The S11 shape. unbuilt + task still ACTIVE is consistent by this rule, however
# false the note is. Pinned so the limit is a claim, not a surprise.
mkarc a.yaml in-progress S11 T-9002 unbuilt
: > "$WORK/tasks/active/T-9002-open.md"
[ "$(rc_of)" = "0" ] && pass "D6 note-drift is out of scope, as documented" \
                     || fail "D6 unexpectedly fired on a consistent slice"

echo "D7: a task that resolves nowhere fires"
mkarc a.yaml in-progress S1 T-9999 unbuilt
[ "$(rc_of)" = "1" ] && pass "D7 unresolvable task binding fires" || fail "D7 did not fire"

echo "D8 [FAIL-CLOSED]: a slices: key with no slices is tooling, not clean"
cat > "$WORK/arcs/a.yaml" <<'EOS'
slug: fixarc
status: in-progress
slices: []
EOS
[ "$(rc_of)" = "2" ] && pass "D8 rc=2" || fail "D8 expected 2 got $(rc_of)"

echo "D9 [FAIL-CLOSED]: an unparseable arc exits 2, never clean"
printf 'slug: x\nstatus: in-progress\nslices:\n  - id: [unclosed\n' > "$WORK/arcs/a.yaml"
[ "$(rc_of)" = "2" ] && pass "D9 rc=2" || fail "D9 expected 2 got $(rc_of)"

echo "D10 [FAIL-CLOSED]: an absent arcs dir exits 2"
ARC_SLICE_ARCS_DIR="$WORK/nope" bash "$CHK" >/dev/null 2>&1
[ "$?" -eq 2 ] && pass "D10 rc=2" || fail "D10 expected 2"

echo "D11: an allowlisted slice is counted and reported but does NOT fire"
mkarc a.yaml in-progress S1 T-9001 unbuilt
printf 'fixarc::S1  # acknowledged for the fixture, with a cited reason\n' > "$WORK/allow"
[ "$(rc_of "$WORK/allow")" = "0" ] && pass "D11 acknowledged entry does not fire" \
                                   || fail "D11 allowlist ignored"

echo "D12: removing the allowlist entry re-fires that slice"
[ "$(rc_of)" = "1" ] && pass "D12 re-fires without the acknowledgement" || fail "D12 did not re-fire"

echo "D13: every output path states the scope (T-2680)"
mkarc a.yaml in-progress S1 T-9001 built
clean_out="$(run)"
mkarc a.yaml in-progress S1 T-9001 unbuilt
fire_out="$(run)"
if printf '%s' "$clean_out" | grep -q 'SCOPE' && printf '%s' "$fire_out" | grep -q 'SCOPE'; then
    pass "D13 scope stated on both the clean and the firing path"
else
    fail "D13 a path omits the scope disclaimer"
fi

echo "D14: --json carries the counts and the scope"
mkarc a.yaml in-progress S1 T-9001 unbuilt
j="$(ARC_SLICE_ARCS_DIR="$WORK/arcs" ARC_SLICE_TASKS_DIR="$WORK/tasks" \
     ARC_SLICE_ALLOWLIST="$WORK/empty-allowlist" bash "$CHK" --json 2>/dev/null)"
if printf '%s' "$j" | python3 -c "
import json,sys
d=json.load(sys.stdin)
sys.exit(0 if d['ok'] is False and d['firing_count']==1 and d['scope'] else 1)" 2>/dev/null; then
    pass "D14 json envelope is well-formed"
else
    fail "D14 json envelope wrong: $j"
fi

echo "D15 [GUARD-LAYER MEMBER]: it carries the marker the runner discovers"
head -3 "$CHK" | grep -q '^# guard-layer: source' \
    && pass "D15 marked as a guard-layer member" \
    || fail "D15 missing the guard-layer marker — it would never run automatically"

echo
echo "arc-slice-drift fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
