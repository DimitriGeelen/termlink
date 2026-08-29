#!/usr/bin/env bash
#
# hook-counter-integrity-fixtures.sh (T-2795)
#
# Fixture suite for scripts/check-hook-counter-integrity.sh. Hermetic: every case builds a
# counter file in a temp dir and points the check at it with --counter. Never reads the
# live `.context/working/.hook-counter` except in group E, which is a deliberate
# characterisation of the real tree.
set -u

SCRIPT="${SCRIPT:-scripts/check-hook-counter-integrity.sh}"
PASS=0; FAIL=0
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

ok()   { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  FAIL %s\n     %s\n' "$1" "${2:-}"; }

# assert_rc <label> <expected-rc> <counter-file> [extra args...]
assert_rc() {
    local label="$1" want="$2" file="$3"; shift 3
    local out rc
    out="$(bash "$SCRIPT" --counter "$file" "$@" 2>&1)"; rc=$?
    if [ "$rc" = "$want" ]; then ok "$label"; else bad "$label" "expected rc=$want got rc=$rc — $out"; fi
}

# assert_contains <label> <needle> <counter-file> [extra args...]
assert_contains() {
    local label="$1" needle="$2" file="$3"; shift 3
    local out
    out="$(bash "$SCRIPT" --counter "$file" "$@" 2>&1 || true)"
    if grep -qF "$needle" <<< "$out"; then ok "$label"
    else bad "$label" "missing '$needle' in: $out"; fi
}

assert_not_contains() {
    local label="$1" needle="$2" file="$3"; shift 3
    local out
    out="$(bash "$SCRIPT" --counter "$file" "$@" 2>&1 || true)"
    if grep -qF "$needle" <<< "$out"; then bad "$label" "unexpected '$needle' in: $out"
    else ok "$label"; fi
}

echo "== A: clean file passes =="
printf 'alpha=3\nbeta=7\ngamma=1\n' > "$TMP/clean"
assert_rc       "A1 clean file exits 0"                0 "$TMP/clean"
assert_contains "A2 clean names the count"             "3 line(s)" "$TMP/clean"
assert_contains "A3 clean still states scope"          "SCOPE:" "$TMP/clean"
assert_not_contains "A4 clean does not claim trustworthy telemetry" "FIRING" "$TMP/clean"

echo "== B: duplicate keys (the concurrent-rewrite artefact) =="
printf 'alpha=3\nbeta=29\ngamma=1\nbeta=28\n' > "$TMP/dup"
assert_rc       "B1 duplicate key fires"               1 "$TMP/dup"
assert_contains "B2 names the duplicated key"          "beta" "$TMP/dup"
assert_contains "B3 reports DUPLICATE class"           "DUPLICATE keys" "$TMP/dup"

echo "== C: reader disagreement is computed, not asserted =="
# first-match reader sees 29; summing reader sees 29+28=57
assert_contains "C1 first-match value shown"           "29" "$TMP/dup"
assert_contains "C2 summing value shown"               "57" "$TMP/dup"
assert_contains "C3 disagreement section present"      "READER DISAGREEMENT" "$TMP/dup"
assert_contains "C4 names the suppression direction"   "false silence" "$TMP/dup"

echo "== D: malformed line (caught mid-write) =="
printf 'alpha\nbeta=7\nalpha=12\n' > "$TMP/mal"
assert_rc       "D1 malformed line fires"              1 "$TMP/mal"
assert_contains "D2 reports MALFORMED class"           "MALFORMED" "$TMP/mal"
assert_contains "D3 empty-read is surfaced"            "(empty)" "$TMP/mal"
# The live check-active-task shape: bare key AND a valid entry for the same key.
assert_contains "D4 summing reader still sees 12"      "12" "$TMP/mal"

echo "== E: fail-closed on absent / empty input =="
assert_rc "E1 missing file exits 2 (not clean)"        2 "$TMP/does-not-exist"
: > "$TMP/empty"
assert_rc "E2 empty file exits 2 (not clean)"          2 "$TMP/empty"
assert_contains "E3 empty refusal is explicit"         "refusing to report clean" "$TMP/empty"

echo "== F: JSON envelope =="
J="$(bash "$SCRIPT" --counter "$TMP/dup" --json 2>&1 || true)"
if python3 -c "import json,sys; json.loads(sys.argv[1])" "$J" 2>/dev/null; then ok "F1 JSON parses"
else bad "F1 JSON parses" "$J"; fi
if python3 -c "
import json,sys
d=json.loads(sys.argv[1])
assert d['ok'] is False, 'ok should be False on corrupt'
assert d['duplicate_keys']==1, d['duplicate_keys']
assert any(x['key']=='beta' and x['summing_reader']==57 for x in d['disagreements']), d['disagreements']
assert 'scope' in d
" "$J" 2>/dev/null; then ok "F2 JSON carries corruption detail"; else bad "F2 JSON carries corruption detail" "$J"; fi

JC="$(bash "$SCRIPT" --counter "$TMP/clean" --json 2>&1 || true)"
if python3 -c "
import json,sys
d=json.loads(sys.argv[1])
assert d['ok'] is True
assert d['duplicate_keys']==0 and d['malformed']==0
assert 'scope' in d, 'clean path must still carry scope (T-2680)'
" "$JC" 2>/dev/null; then ok "F3 clean JSON carries scope"; else bad "F3 clean JSON carries scope" "$JC"; fi

echo "== G: tier declaration — must NOT be a guard-layer member =="
# The whole argument in the header is that enrolling a permanently-firing check damages
# the roll-up. If someone adds the marker later, this suite must go red.
if grep -qE '^#[[:space:]]*guard-layer:' "$SCRIPT"; then
    bad "G1 no guard-layer marker" "marker present — see header rationale before enrolling"
else ok "G1 no guard-layer marker"; fi
if grep -q "TIER: on-demand diagnostic" "$SCRIPT"; then ok "G2 tier is declared in header"
else bad "G2 tier is declared in header" "header must state its tier"; fi
if grep -q "L-023" "$SCRIPT"; then ok "G3 cites the recurring learning"
else bad "G3 cites the recurring learning" "L-023 citation missing"; fi

echo "== H: quiet mode =="
QOUT="$(bash "$SCRIPT" --counter "$TMP/clean" --quiet 2>&1 || true)"
if [ -z "$QOUT" ]; then ok "H1 quiet is silent when clean"; else bad "H1 quiet is silent when clean" "$QOUT"; fi
QOUT2="$(bash "$SCRIPT" --counter "$TMP/dup" --quiet 2>&1 || true)"
if grep -qF "FIRING" <<< "$QOUT2"; then ok "H2 quiet still reports corruption"
else bad "H2 quiet still reports corruption" "$QOUT2"; fi

echo
printf 'hook-counter-integrity fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
