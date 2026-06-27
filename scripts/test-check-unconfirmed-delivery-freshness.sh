#!/usr/bin/env bash
# T-2295 (arc-003 reliable-comms, V3b) — tests for the unconfirmed-delivery canary.
# Hub-independent via TERMLINK_UNCONFIRMED_TEST_JSON (mirrors TERMLINK_GROWTH_TEST_JSON).
#
# Covers:
#   T1 --help → 0
#   T2 unknown arg → 2
#   T3 stale row → exit 1 (firing) + names the topic
#   T4 fresh row → exit 0 (healthy)
#   T5 empty tracker → exit 0 (healthy)
#   T6 --json on stale → ok:false, stale_count:1, parseable
#   T7 --json on empty → ok:true, stale_count:0
#   T8 --threshold-secs huge → fresh-classifies an otherwise-stale row (exit 0)
#   T9 malformed JSON → exit 2 (tooling error)
set -u

SCRIPT="${SCRIPT:-scripts/check-unconfirmed-delivery-freshness.sh}"
DATA="scripts/testdata"

PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

# Regenerate testdata with current-relative timestamps so age math is stable.
now_ms=$(( $(date +%s) * 1000 ))
old_ms=$(( now_ms - 3600*1000 ))
fresh_ms=$(( now_ms - 5*1000 ))
mkdir -p "$DATA"
printf '{"exists":true,"pending":1,"rows":[{"attempts":3,"client_msg_id":"abc","dm_topic":"dm:aa:bb","enqueued_ms":%s,"msg_offset":0,"recipient_sender_id":"bb"}]}\n' "$old_ms"   > "$DATA/unconfirmed-stale.json"
printf '{"exists":true,"pending":1,"rows":[{"attempts":1,"client_msg_id":"def","dm_topic":"dm:cc:dd","enqueued_ms":%s,"msg_offset":0,"recipient_sender_id":"dd"}]}\n' "$fresh_ms" > "$DATA/unconfirmed-fresh.json"
printf '{"exists":false,"pending":0,"rows":[]}\n' > "$DATA/unconfirmed-empty.json"

run() { TERMLINK_UNCONFIRMED_TEST_JSON="$1" bash "$SCRIPT" "${@:2}" --no-heartbeat 2>&1; }

echo "T1: --help → 0"
out="$(bash "$SCRIPT" --help 2>/dev/null)"; rc=$?
{ [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF "Usage:"; } && pass "T1 exit=$rc" || fail "T1 exit=$rc"

echo "T2: unknown arg → 2"
bash "$SCRIPT" --bogus >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T2 exit=$rc" || fail "T2 expected 2 got $rc"

echo "T3: stale → exit 1 + names topic"
out="$(run "$DATA/unconfirmed-stale.json")"; rc=$?
{ [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -qF "dm:aa:bb"; } && pass "T3 exit=$rc" || fail "T3 exit=$rc out=$out"

echo "T4: fresh → exit 0"
run "$DATA/unconfirmed-fresh.json" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 0 ] && pass "T4 exit=$rc" || fail "T4 expected 0 got $rc"

echo "T5: empty → exit 0"
run "$DATA/unconfirmed-empty.json" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 0 ] && pass "T5 exit=$rc" || fail "T5 expected 0 got $rc"

echo "T6: --json stale → ok:false stale_count:1"
out="$(run "$DATA/unconfirmed-stale.json" --json)"; rc=$?
ok="$(printf '%s' "$out" | jq -r '.ok' 2>/dev/null)"; sc="$(printf '%s' "$out" | jq -r '.stale_count' 2>/dev/null)"
{ [ "$ok" = "false" ] && [ "$sc" = "1" ] && [ "$rc" -eq 1 ]; } && pass "T6 ok=$ok stale_count=$sc" || fail "T6 ok=$ok sc=$sc rc=$rc"

echo "T7: --json empty → ok:true stale_count:0"
out="$(run "$DATA/unconfirmed-empty.json" --json)"; rc=$?
ok="$(printf '%s' "$out" | jq -r '.ok' 2>/dev/null)"; sc="$(printf '%s' "$out" | jq -r '.stale_count' 2>/dev/null)"
{ [ "$ok" = "true" ] && [ "$sc" = "0" ] && [ "$rc" -eq 0 ]; } && pass "T7 ok=$ok stale_count=$sc" || fail "T7 ok=$ok sc=$sc rc=$rc"

echo "T8: huge --threshold-secs → otherwise-stale row classifies healthy"
run "$DATA/unconfirmed-stale.json" --threshold-secs 999999999 >/dev/null 2>&1; rc=$?
[ "$rc" -eq 0 ] && pass "T8 exit=$rc" || fail "T8 expected 0 got $rc"

echo "T9: malformed JSON → exit 2"
bad="$(mktemp)"; printf 'not json{{{' > "$bad"
run "$bad" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T9 exit=$rc" || fail "T9 expected 2 got $rc"
rm -f "$bad"

echo ""
echo "Results: $PASS pass / $FAIL fail"
[ "$FAIL" -eq 0 ]
