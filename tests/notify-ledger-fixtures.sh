#!/usr/bin/env bash
# guard-layer: source
# T-3070 — fixtures for scripts/notify-ledger.sh (the sender's own ledger).
#
# Hermetic: temp ledger, canned receipts via LEDGER_TEST_RECEIPTS. No hub.
#
# THE TWO LOAD-BEARING CASES are the ones that protect the ledger from becoming
# a confident liar:
#   T5  a rung NEVER advances without a receipt actually read back
#   T7  a stage=read receipt carrying evidence=wake-consumer is IGNORED
# T7 matters because such receipts EXIST on live topics right now — offset 110 on
# dm:3bba15e681b3a078:d1993c2c3ec44c94 is one. That evidence kind was withdrawn as
# untruthful (a consumer noticing a flag has injected nothing). If this ledger
# honoured them it would relaunder a retracted claim as fact.
set -u

LED="${LED:-scripts/notify-ledger.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

command -v sqlite3 >/dev/null 2>&1 || { echo "SKIP: sqlite3 not available"; exit 0; }
command -v jq      >/dev/null 2>&1 || { echo "SKIP: jq not available"; exit 0; }

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
L="$WORK/ledger.sqlite"
T="dm:aaaa:bbbb"
run() { bash "$LED" "$@" --ledger "$L"; }
rung() { sqlite3 "$L" "SELECT rung FROM outbox WHERE topic='$T' AND offset=$1;"; }

echo "T1: --help exits 0"
bash "$LED" --help >/dev/null 2>&1 && pass "T1" || fail "T1"

echo "T2: an unknown verb exits 2"
bash "$LED" bogus >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T2 rc=$rc" || fail "T2 expected 2 got $rc"

echo "T3: record requires --offset (never invent one)"
run record --topic "$T" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T3 rc=$rc" || fail "T3 expected 2 got $rc"

echo "T4: a recorded message starts at rung 'sent'"
run record --topic "$T" --offset 5 --peer bbbb --summary "first" --quiet >/dev/null 2>&1
[ "$(rung 5)" = "sent" ] && pass "T4 rung=sent" || fail "T4 rung=$(rung 5)"

echo "T5 [INTEGRITY]: with NO receipts, sync must NOT advance the rung"
printf '' > "$WORK/none.ndjson"
LEDGER_TEST_RECEIPTS="$WORK/none.ndjson" run sync --quiet >/dev/null 2>&1
if [ "$(rung 5)" = "sent" ]; then
    pass "T5 rung unchanged — absence of a receipt is not progress"
else
    fail "T5 LEDGER INVENTED A RUNG — rung=$(rung 5) with zero receipts"
fi

echo "T6: an L2 receipt advances 'sent' -> 'delivered'"
cat > "$WORK/l2.ndjson" <<'EOS'
{"msg_type":"receipt","sender_id":"bbbb","offset":9,"metadata":{"stage":"delivered","up_to":"5"}}
EOS
LEDGER_TEST_RECEIPTS="$WORK/l2.ndjson" run sync --quiet >/dev/null 2>&1
[ "$(rung 5)" = "delivered" ] && pass "T6 rung=delivered" || fail "T6 rung=$(rung 5)"

echo "T7 [INTEGRITY]: a stage=read carrying evidence=wake-consumer is IGNORED"
cat > "$WORK/wc.ndjson" <<'EOS'
{"msg_type":"receipt","sender_id":"bbbb","offset":9,"metadata":{"stage":"delivered","up_to":"5"}}
{"msg_type":"receipt","sender_id":"bbbb","offset":10,"metadata":{"stage":"read","up_to":"5","evidence":"wake-consumer"}}
EOS
LEDGER_TEST_RECEIPTS="$WORK/wc.ndjson" run sync --quiet >/dev/null 2>&1
if [ "$(rung 5)" = "delivered" ]; then
    pass "T7 a withdrawn evidence kind does not advance the rung"
else
    fail "T7 A DISAVOWED RECEIPT WAS HONOURED — rung=$(rung 5)"
fi

echo "T8: a stage=read with VALID evidence does advance to 'read'"
cat > "$WORK/l3.ndjson" <<'EOS'
{"msg_type":"receipt","sender_id":"bbbb","offset":11,"metadata":{"stage":"read","up_to":"5","evidence":"idle-gated-inject"}}
EOS
LEDGER_TEST_RECEIPTS="$WORK/l3.ndjson" run sync --quiet >/dev/null 2>&1
[ "$(rung 5)" = "read" ] && pass "T8 rung=read" || fail "T8 rung=$(rung 5)"

echo "T9: OUR OWN receipts do not advance our own rung"
run record --topic "$T" --offset 20 --quiet >/dev/null 2>&1
cat > "$WORK/self.ndjson" <<'EOS'
{"msg_type":"receipt","sender_id":"aaaa","offset":30,"metadata":{"stage":"delivered","up_to":"20"}}
EOS
LEDGER_TEST_RECEIPTS="$WORK/self.ndjson" run sync --self-fp aaaa --quiet >/dev/null 2>&1
if [ "$(rung 20)" = "sent" ]; then
    pass "T9 self-authored receipts are not confirmation"
else
    fail "T9 OUR OWN RECEIPT CONFIRMED OUR OWN SEND — rung=$(rung 20)"
fi

echo "T10: status shows what has not reached 'read'"
OUT="$(run status 2>&1)"
printf '%s' "$OUT" | grep -q '20' && pass "T10 outstanding message listed" \
                                  || fail "T10 outstanding not shown: $OUT"

echo "T11: --stuck exits 1 when something has sat too long"
run status --stuck --older-than 0 >/dev/null 2>&1; rc=$?
[ "$rc" -eq 1 ] && pass "T11 rc=$rc" || fail "T11 expected 1 got $rc"

echo "T12: --stuck exits 0 when nothing is old enough"
run status --stuck --older-than 99999 >/dev/null 2>&1; rc=$?
[ "$rc" -eq 0 ] && pass "T12 rc=$rc" || fail "T12 expected 0 got $rc"

echo "T13: an all-read ledger reports nothing outstanding"
L2="$WORK/clean.sqlite"
bash "$LED" record --topic "$T" --offset 1 --ledger "$L2" --quiet >/dev/null 2>&1
sqlite3 "$L2" "UPDATE outbox SET rung='read';"
OUT="$(bash "$LED" status --ledger "$L2" 2>&1)"
printf '%s' "$OUT" | grep -q 'nothing outstanding' && pass "T13 clean ledger is legible" \
                                                   || fail "T13: $OUT"

echo "T14 [fail-closed]: an uncreatable ledger exits 2, not a clean 'nothing stuck'"
bash "$LED" status --stuck --ledger /proc/cannot/exist/l.sqlite >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T14 rc=$rc" || fail "T14 expected 2 got $rc"

echo "T15: record is idempotent (same topic+offset does not duplicate)"
before="$(sqlite3 "$L" "SELECT COUNT(*) FROM outbox;")"
run record --topic "$T" --offset 20 --quiet >/dev/null 2>&1
after="$(sqlite3 "$L" "SELECT COUNT(*) FROM outbox;")"
[ "$before" = "$after" ] && pass "T15 no duplicate row" || fail "T15 $before -> $after"

echo
echo "notify-ledger fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
