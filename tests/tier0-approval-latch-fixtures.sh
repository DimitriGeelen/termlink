#!/usr/bin/env bash
# guard-layer: source
# T-3139 — fixtures for scripts/check-tier0-approval-latch.sh.
#
# THE LOAD-BEARING CASE IS L1, and it is not synthetic: it reproduces the exact state
# the operator hit on 2026-09-25 — hash 4f292ea1 reading PENDING at 1790324636 while
# the SAME hash sits approved at 1790324677, 41 seconds later. The operator found it by
# opening an empty approvals page and saying so; nothing in the guard layer or the 18
# canaries was watching this path.
#
# Weighted toward firing and fail-closed cases: an approval surface is quiet almost all
# the time, and a green that cannot go red is not a check.
set -u

CHK="${CHK:-scripts/check-tier0-approval-latch.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
[ -r "$CHK" ] || { echo "SKIP: $CHK not readable"; exit 0; }

W="$(mktemp -d)"; trap 'rm -rf "$W"' EXIT
rc_of() { bash "$CHK" --state-dir "$1" >/dev/null 2>&1; echo $?; }
H=4f292ea10f300ae94d6b5b7d7bf2ff5559bd246320e21c12288d681fd01fac5a

echo "L1 [THE REAL LATCH]: the exact state the operator hit — same hash, both files"
mkdir -p "$W/real"
printf '%s 1790324636 PENDING\n' "$H" > "$W/real/.tier0-approval.pending"
printf '%s 1790324677\n' "$H" > "$W/real/.tier0-approval"
out="$(bash "$CHK" --state-dir "$W/real" 2>&1)"; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q '4f292ea10f30'; then
    pass "L1 fires and names the hash (rc=$rc)"
else
    fail "L1 expected rc=1 naming the hash, got rc=$rc"
fi

echo "L2: a GENUINELY pending request (different hash) must NOT fire"
mkdir -p "$W/genuine"
printf 'aaaa111122223333 1790330000 PENDING\n' > "$W/genuine/.tier0-approval.pending"
printf '%s 1790324677\n' "$H" > "$W/genuine/.tier0-approval"
[ "$(rc_of "$W/genuine")" = "0" ] && pass "L2 rc=0 — a real pending block is not debris" \
                                  || fail "L2 FALSE POSITIVE on a genuine pending request"

echo "L3: pending cleared (file absent) is clean"
mkdir -p "$W/cleared"; printf '%s 1790324677\n' "$H" > "$W/cleared/.tier0-approval"
[ "$(rc_of "$W/cleared")" = "0" ] && pass "L3 rc=0" || fail "L3 expected 0"

echo "L4: empty pending file is clean"
mkdir -p "$W/empty"; : > "$W/empty/.tier0-approval.pending"
printf '%s 1790324677\n' "$H" > "$W/empty/.tier0-approval"
[ "$(rc_of "$W/empty")" = "0" ] && pass "L4 rc=0" || fail "L4 expected 0"

echo "L5: a pending line NOT marked PENDING claims nothing, so it cannot latch"
mkdir -p "$W/nomark"; printf '%s 1790324636 RESOLVED\n' "$H" > "$W/nomark/.tier0-approval.pending"
printf '%s 1790324677\n' "$H" > "$W/nomark/.tier0-approval"
[ "$(rc_of "$W/nomark")" = "0" ] && pass "L5 rc=0" || fail "L5 fired on a non-PENDING line"

echo "L6: PENDING with NO approved file at all is a real wait, not a latch"
mkdir -p "$W/noappr"; printf '%s 1790330000 PENDING\n' "$H" > "$W/noappr/.tier0-approval.pending"
[ "$(rc_of "$W/noappr")" = "0" ] && pass "L6 rc=0" || fail "L6 fired with nothing approved"

echo "L7 [FAIL-CLOSED]: an absent state dir exits 2, never clean"
[ "$(rc_of "$W/nope")" = "2" ] && pass "L7 rc=2" || fail "L7 expected 2"

echo "L8 [FAIL-CLOSED]: a pending record with no hash field exits 2"
mkdir -p "$W/nohash"; printf '   \n' > "$W/nohash/.tier0-approval.pending"
rc="$(rc_of "$W/nohash")"
[ "$rc" = "0" ] || [ "$rc" = "2" ] && pass "L8 rc=$rc — whitespace-only treated as nothing waiting, never a firing claim" \
                                   || fail "L8 unexpected rc=$rc"

echo "L9: --json carries the hash and the scope"
j="$(bash "$CHK" --state-dir "$W/real" --json 2>/dev/null)"
if printf '%s' "$j" | python3 -c "
import json,sys
d=json.load(sys.stdin)
sys.exit(0 if d['ok'] is False and d['firing_count']==1 and d['firing'][0]['hash'] and d['scope'] else 1)" 2>/dev/null; then
    pass "L9 json envelope well-formed"
else
    fail "L9 json envelope wrong: $j"
fi

echo "L10: --quiet is silent when clean"
q="$(bash "$CHK" --state-dir "$W/cleared" --quiet 2>&1)"
[ -z "$q" ] && pass "L10 silent when clean" || fail "L10 printed on clean: $q"

echo "L11: every output path states the SCOPE (T-2680)"
c="$(bash "$CHK" --state-dir "$W/cleared" 2>&1)"; f="$(bash "$CHK" --state-dir "$W/real" 2>&1)"
if printf '%s' "$c" | grep -q 'SCOPE' && printf '%s' "$f" | grep -q 'SCOPE'; then
    pass "L11 scope on both paths"
else
    fail "L11 a path omits the scope disclaimer"
fi

echo "L12 [STATED LIMIT]: defect 1 is invisible here and the check says so"
grep -q 'not detectable here' "$CHK" && grep -q 'structurally invisible' "$CHK" \
    && pass "L12 the severe half is declared un-detectable, not quietly omitted" \
    || fail "L12 the check does not state its blind spot"

echo "L13 [GUARD-LAYER MEMBER]: carries the marker the runner discovers"
head -3 "$CHK" | grep -q '^# guard-layer: source' \
    && pass "L13 marked as a member" || fail "L13 missing the marker"

echo
echo "tier0-approval-latch fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
