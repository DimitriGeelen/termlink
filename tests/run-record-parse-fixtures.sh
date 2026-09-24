#!/usr/bin/env bash
# guard-layer: source
# T-3092 — fixtures for scripts/check-run-record-parse.sh.
#
# THE LOAD-BEARING CASE IS R1, and it is not synthetic: it extracts the T-3089
# run record exactly as round 3 left it at commit 782f5c815 and asserts the
# check fires on it. A fixture built from what I BELIEVE the corruption looked
# like would encode the same assumption that let it through; a fixture built
# from the corruption itself cannot.
#
# Weighted toward the FIRING and FAIL-CLOSED cases on purpose: a parse check is
# trivially green on a healthy tree, and a green that cannot go red is not a check.
set -u

CHK="${CHK:-scripts/check-run-record-parse.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

command -v python3 >/dev/null 2>&1 || { echo "SKIP: python3 unavailable"; exit 0; }
python3 -c 'import yaml' 2>/dev/null || { echo "SKIP: PyYAML unavailable"; exit 0; }
[ -r "$CHK" ] || { echo "SKIP: $CHK not readable"; exit 0; }

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
rc_of() { bash "$CHK" --dir "$1" >/dev/null 2>&1; echo $?; }

echo "R1 [THE REAL CORRUPTION]: T-3089's record as round 3 left it, extracted from git"
mkdir -p "$WORK/hist"
if git show 782f5c815:.context/runs/T-3089-procasfit-x4.yaml > "$WORK/hist/rec.yaml" 2>/dev/null; then
    out="$(bash "$CHK" --dir "$WORK/hist" 2>&1)"; rc=$?
    if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q 'UNPARSEABLE'; then
        pass "R1 fires on the genuine corruption (rc=$rc)"
    else
        fail "R1 expected rc=1 UNPARSEABLE, got rc=$rc: $out"
    fi
else
    fail "R1 could not extract the historical record — the load-bearing case did not run"
fi

echo "R2: the current tree is clean"
[ "$(bash "$CHK" >/dev/null 2>&1; echo $?)" = "0" ] && pass "R2 rc=0" || fail "R2 tree not clean"

echo "R3: a well-formed mapping passes"
mkdir -p "$WORK/ok"; printf 'run_id: x\nsteps:\n  - id: R1\n    state: complete\n' > "$WORK/ok/a.yaml"
[ "$(rc_of "$WORK/ok")" = "0" ] && pass "R3 rc=0" || fail "R3 expected 0"

echo "R4: the T-3084 shape — unquoted scalar wrapping onto a ': ' line — fires"
mkdir -p "$WORK/bad"
printf 'steps:\n  - T-1 (x) - deferred and out of THIS\n    project'"'"'s scope: penelope is /opt/x\n' > "$WORK/bad/a.yaml"
[ "$(rc_of "$WORK/bad")" = "1" ] && pass "R4 rc=1 on the recurring class" || fail "R4 did not fire"

echo "R5: a file that parses but is NOT a mapping fires (the reader subscripts it)"
mkdir -p "$WORK/list"; printf -- '- one\n- two\n' > "$WORK/list/a.yaml"
out="$(bash "$CHK" --dir "$WORK/list" 2>&1)"; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q 'NOT-A-MAPPING'; then
    pass "R5 rc=1 NOT-A-MAPPING"
else
    fail "R5 expected NOT-A-MAPPING rc=1, got rc=$rc"
fi

echo "R6: a bare string is also not a mapping"
mkdir -p "$WORK/str"; printf 'just a string\n' > "$WORK/str/a.yaml"
[ "$(rc_of "$WORK/str")" = "1" ] && pass "R6 rc=1" || fail "R6 expected 1"

echo "R7: one bad record among several still fires"
mkdir -p "$WORK/mix"; printf 'a: 1\n' > "$WORK/mix/good.yaml"; printf 'x: [unclosed\n' > "$WORK/mix/bad.yaml"
[ "$(rc_of "$WORK/mix")" = "1" ] && pass "R7 rc=1" || fail "R7 a bad record was masked by a good one"

echo "R8: an ABSENT directory is healthy — nothing was ever recorded is not a fault"
[ "$(rc_of "$WORK/nope")" = "0" ] && pass "R8 rc=0" || fail "R8 expected 0"

echo "R9 [VACUOUS-PASS GUARD]: a directory that EXISTS with zero records is tooling, not clean"
mkdir -p "$WORK/empty"
[ "$(rc_of "$WORK/empty")" = "2" ] && pass "R9 rc=2 — refuses to report clean on an empty corpus" \
                                   || fail "R9 expected 2 (T-2831 vacuous pass)"

echo "R10 [FAIL-CLOSED]: an unreadable file exits 2, never clean"
mkdir -p "$WORK/perm"; printf 'a: 1\n' > "$WORK/perm/a.yaml"; chmod 000 "$WORK/perm/a.yaml"
rc="$(rc_of "$WORK/perm")"; chmod 644 "$WORK/perm/a.yaml"
if [ "$rc" = "2" ]; then pass "R10 rc=2"
elif [ "$(id -u)" = "0" ]; then pass "R10 skipped — running as root, chmod 000 is not a barrier"
else fail "R10 expected 2, got $rc"; fi

echo "R11: --json carries counts, classes and the scope"
j="$(bash "$CHK" --dir "$WORK/bad" --json 2>/dev/null)"
if printf '%s' "$j" | python3 -c "
import json,sys
d=json.load(sys.stdin)
sys.exit(0 if d['ok'] is False and d['firing_count']==1 and d['checked']==1
         and d['firing'][0]['class']=='UNPARSEABLE' and d['scope'] else 1)" 2>/dev/null; then
    pass "R11 json envelope well-formed"
else
    fail "R11 json envelope wrong: $j"
fi

echo "R12: --quiet prints nothing on a clean run"
q="$(bash "$CHK" --dir "$WORK/ok" --quiet 2>&1)"
[ -z "$q" ] && pass "R12 silent when clean" || fail "R12 printed on clean: $q"

echo "R13: every output path states the SCOPE (T-2680)"
c="$(bash "$CHK" --dir "$WORK/ok" 2>&1)"; f="$(bash "$CHK" --dir "$WORK/bad" 2>&1)"
if printf '%s' "$c" | grep -q 'SCOPE' && printf '%s' "$f" | grep -q 'SCOPE'; then
    pass "R13 scope on both clean and firing paths"
else
    fail "R13 a path omits the scope disclaimer"
fi

echo "R14 [GUARD-LAYER MEMBER]: it carries the marker the runner discovers"
head -3 "$CHK" | grep -q '^# guard-layer: source' \
    && pass "R14 marked as a guard-layer member" \
    || fail "R14 missing the marker — it would never run automatically"

echo
echo "run-record-parse fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
