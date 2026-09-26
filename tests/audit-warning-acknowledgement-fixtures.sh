#!/usr/bin/env bash
# Fixtures for scripts/check-audit-warning-acknowledgement.sh (T-3167).
#
# Weighted to the FIRING cases and the fail-closed contract. A ledger-driven check
# is trivially green when everything is listed, and a green check that cannot go
# red is not a check (the T-2812 vendor-divergence lesson). Two mutants are pinned:
# one disables the ledger match, one disables the FAIL guard.
set -uo pipefail

SCRIPT="${SCRIPT:-scripts/check-audit-warning-acknowledgement.sh}"
TMP="$(mktemp -d)"
trap 'rm -rf -- "$TMP"' EXIT
PASS=0; FAIL=0

ok()   { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  FAIL %s\n       %s\n' "$1" "${2:-}"; }
check(){ # check <desc> <expected_rc> <actual_rc> [extra]
    if [ "$2" = "$3" ]; then ok "$1"; else bad "$1" "expected rc=$2 got rc=$3 ${4:-}"; fi
}

mk_audit() { # mk_audit <path> <body>
    mkdir -p "$(dirname "$1")"
    printf '%s\n' "$2" > "$1"
}

AUD_OK="$TMP/a1.yaml"
mk_audit "$AUD_OK" 'timestamp: 2026-01-01T00:00:00Z
summary:
  pass: 1
  warn: 2
  fail: 0
findings:
  - level: PASS
    check: "something fine"
  - level: WARN
    check: "Alpha rail reports NOT EVALUATED: candidate set empty (0 files)"
    mitigation: "the scan walked nothing"
  - level: WARN
    check: "Beta thing is unexplained — 17 of 42 examined"'

LED_BOTH="$TMP/led-both"
printf '%s\n' \
  '# ledger' \
  'Alpha rail reports  # acknowledged: vendored, filed upstream. DELETE when upstream fixes it.' \
  'Beta thing is unexplained  # acknowledged: deliberate, revisit at v2.' > "$LED_BOTH"

LED_ONE="$TMP/led-one"
printf '%s\n' 'Alpha rail reports  # acknowledged: vendored. DELETE when fixed.' > "$LED_ONE"

echo "== Case 1-2: clean vs unexamined =="
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$LED_BOTH" >/dev/null 2>&1; check "all warnings acknowledged -> rc 0" 0 "$?"
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$LED_ONE"  >/dev/null 2>&1; check "one warning unacknowledged -> rc 1 (FIRES)" 1 "$?"

echo "== Case 3: absent ledger acknowledges NOTHING (never excuses everything) =="
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$TMP/does-not-exist" >/dev/null 2>&1
check "absent ledger -> rc 1, not a free pass" 1 "$?"
out="$(bash "$SCRIPT" --audit "$AUD_OK" --ledger "$TMP/does-not-exist" --json 2>&1)"
if printf '%s' "$out" | grep -q '"acknowledged_count": 0'; then ok "absent ledger reports 0 acknowledged"; else bad "absent ledger acknowledged_count" "$out"; fi
if printf '%s' "$out" | grep -q '"ledger_exists": false'; then ok "absent ledger is reported, not silent"; else bad "ledger_exists flag" "$out"; fi

echo "== Case 4: a FAIL can never be acknowledged =="
AUD_FAIL="$TMP/afail.yaml"
mk_audit "$AUD_FAIL" 'findings:
  - level: FAIL
    check: "Gamma is broken"
  - level: WARN
    check: "Alpha rail reports NOT EVALUATED"'
LED_FAIL="$TMP/led-fail"
printf '%s\n' \
  'Alpha rail reports  # acknowledged: vendored. DELETE when fixed.' \
  'Gamma is broken  # attempting to silence a FAIL, which must not work' > "$LED_FAIL"
bash "$SCRIPT" --audit "$AUD_FAIL" --ledger "$LED_FAIL" >/dev/null 2>&1
check "FAIL present + ledger entry for it -> still rc 1" 1 "$?"
o="$(bash "$SCRIPT" --audit "$AUD_FAIL" --ledger "$LED_FAIL" --json 2>&1)"
if printf '%s' "$o" | grep -q '"fail_count": 1'; then ok "FAIL is counted separately"; else bad "fail_count" "$o"; fi

echo "== Case 5: fail-closed on malformed ledger =="
L="$TMP/led-noreason"; printf '%s\n' 'Alpha rail reports' > "$L"
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$L" >/dev/null 2>&1
check "ledger entry with no '# reason' -> rc 2" 2 "$?"
L="$TMP/led-emptyreason"; printf '%s\n' 'Alpha rail reports  #   ' > "$L"
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$L" >/dev/null 2>&1
check "ledger entry with empty reason -> rc 2" 2 "$?"
L="$TMP/led-emptypat"; printf '%s\n' '   # only a reason, no pattern' > "$L"
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$L" >/dev/null 2>&1
check "comment-only line is skipped, not an error -> rc 1 (both warns unacked)" 1 "$?"

echo "== Case 6: fail-closed on bad audit input =="
bash "$SCRIPT" --audit "$TMP/nope.yaml" --ledger "$LED_BOTH" >/dev/null 2>&1
check "absent audit file -> rc 2" 2 "$?"
mk_audit "$TMP/empty.yaml" 'timestamp: x
findings: []'
bash "$SCRIPT" --audit "$TMP/empty.yaml" --ledger "$LED_BOTH" >/dev/null 2>&1
check "audit with zero findings -> rc 2 (never a vacuous clean)" 2 "$?"
printf '%s\n' '- just' '- a list' > "$TMP/notmap.yaml"
bash "$SCRIPT" --audit "$TMP/notmap.yaml" --ledger "$LED_BOTH" >/dev/null 2>&1
check "audit that is not a mapping -> rc 2" 2 "$?"
bash "$SCRIPT" --dir "$TMP/no-such-dir" --ledger "$LED_BOTH" >/dev/null 2>&1
check "absent audit dir -> rc 2" 2 "$?"

echo "== Case 7: the selector skips non-date files (the bug found in development) =="
D="$TMP/auds"; mkdir -p "$D"
mk_audit "$D/2026-09-26.yaml" 'findings:
  - level: WARN
    check: "Alpha rail reports NOT EVALUATED"'
mk_audit "$D/upgrades.yaml" 'some_other_shape: true'
bash "$SCRIPT" --dir "$D" --ledger "$LED_ONE" >/dev/null 2>&1
check "picks the dated audit, not upgrades.yaml -> rc 0" 0 "$?"
o="$(bash "$SCRIPT" --dir "$D" --ledger "$LED_ONE" --json 2>&1)"
if printf '%s' "$o" | grep -q '2026-09-26.yaml'; then ok "selected the date-stamped file"; else bad "selector" "$o"; fi

echo "== Case 8: stale ledger entries — rot is visible =="
LED_STALE="$TMP/led-stale"
printf '%s\n' \
  'Alpha rail reports  # acknowledged: vendored. DELETE when fixed.' \
  'Beta thing is unexplained  # acknowledged: deliberate.' \
  'Delta never appears anywhere  # an entry whose finding is gone' > "$LED_STALE"
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$LED_STALE" >/dev/null 2>&1
check "stale entry is non-firing by default -> rc 0" 0 "$?"
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$LED_STALE" --strict >/dev/null 2>&1
check "stale entry FIRES under --strict -> rc 1" 1 "$?"
o="$(bash "$SCRIPT" --audit "$AUD_OK" --ledger "$LED_STALE" --json 2>&1)"
if printf '%s' "$o" | grep -q '"stale_entry_count": 1'; then ok "stale entry is counted"; else bad "stale_entry_count" "$o"; fi

echo "== Case 9: the count-drift hazard the format warns about =="
LED_DRIFT="$TMP/led-drift"
printf '%s\n' 'Beta thing is unexplained — 17 of 42 examined  # pattern carries counts (the documented mistake)' > "$LED_DRIFT"
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$LED_DRIFT" >/dev/null 2>&1
check "count-bearing pattern matches while counts hold -> rc 1 (other warn unacked)" 1 "$?"
AUD_MOVED="$TMP/moved.yaml"
mk_audit "$AUD_MOVED" 'findings:
  - level: WARN
    check: "Beta thing is unexplained — 18 of 43 examined"'
o="$(bash "$SCRIPT" --audit "$AUD_MOVED" --ledger "$LED_DRIFT" --json 2>&1)"
if printf '%s' "$o" | grep -q '"acknowledged_count": 0'; then ok "count moved -> pattern stops matching (why the format says use the NAME)"; else bad "drift demo" "$o"; fi
if printf '%s' "$o" | grep -q '"stale_entry_count": 1'; then ok "and the drifted entry surfaces as stale, not silent"; else bad "drift->stale" "$o"; fi

echo "== Case 10: scope disclaimer on BOTH paths (T-2680) =="
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$LED_BOTH" > "$TMP/clean.out" 2>&1
grep -q 'never that the project has no warnings' "$TMP/clean.out" && ok "clean path carries the scope disclaimer" || bad "clean scope" ""
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$LED_ONE" > "$TMP/fire.out" 2>&1
grep -q 'never that the project has no warnings' "$TMP/fire.out" && ok "firing path carries the scope disclaimer" || bad "firing scope" ""
grep -q 'reason:' "$TMP/clean.out" && ok "acknowledged entries print their cited reason" || bad "reason printed" ""

echo "== Case 11: --quiet and guard-layer parity =="
o="$(bash "$SCRIPT" --audit "$AUD_OK" --ledger "$LED_BOTH" --quiet 2>&1)"
[ -z "$o" ] && ok "--quiet prints nothing when clean" || bad "--quiet clean" "$o"
o="$(bash "$SCRIPT" --audit "$AUD_OK" --ledger "$LED_ONE" --quiet 2>&1)"
[ -n "$o" ] && ok "--quiet still speaks when firing" || bad "--quiet firing" ""
bash "$SCRIPT" --audit "$AUD_OK" --ledger "$LED_BOTH" --no-heartbeat >/dev/null 2>&1
check "--no-heartbeat accepted (guard-layer marker parity)" 0 "$?"
bash "$SCRIPT" --bogus-flag >/dev/null 2>&1
check "unknown flag -> rc 2" 2 "$?"
grep -q '^# guard-layer: source' "$SCRIPT" && ok "carries the guard-layer marker (T-2683: else nothing runs it)" || bad "guard-layer marker" ""

echo "== Case 12: MUTANTS — prove the logic is load-bearing =="
M="$TMP/mutant-match.sh"
sed 's|hit = next((e for e in entries if e\["pattern"\] in w\["check"\]), None)|hit = None|' "$SCRIPT" > "$M"
bash "$M" --audit "$AUD_OK" --ledger "$LED_BOTH" >/dev/null 2>&1
check "mutant disabling the ledger match -> rc 1 (acknowledged become unexamined)" 1 "$?"
o="$(bash "$M" --audit "$AUD_OK" --ledger "$LED_BOTH" --json 2>&1)"
if printf '%s' "$o" | grep -q '"unexamined_count": 2'; then ok "mutant reproduces the pre-ledger world (8-style flat count)"; else bad "mutant match" "$o"; fi
M2="$TMP/mutant-fail.sh"
sed 's|fires = bool(unexamined) or bool(fails) or (strict and bool(stale))|fires = bool(unexamined) or (strict and bool(stale))|' "$SCRIPT" > "$M2"
bash "$M2" --audit "$AUD_FAIL" --ledger "$LED_FAIL" >/dev/null 2>&1
check "mutant dropping the FAIL guard -> rc 0, i.e. a silenced FAIL" 0 "$?"
ok "  (that mutant MUST differ from the real script, which returns 1 — pinned above)"

printf '\n%s\n' "audit-warning-acknowledgement fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
