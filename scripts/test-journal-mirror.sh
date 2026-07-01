#!/usr/bin/env bash
# T-2298 (arc-003 reliable-comms, V6 slice S1) — tests for the per-conversation
# journal mirror + query verb. No second host: self-post to a dm: topic on the
# local hub, mirror it, assert the journal row + query round-trip, and prove
# idempotency (a second mirror pass inserts 0 rows).
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MIRROR="$HERE/journal-mirror.sh"
QUERY="$HERE/agent-journal.sh"
TERMLINK="${TERMLINK_BIN:-termlink}"

command -v "$TERMLINK" >/dev/null 2>&1 || { echo "SKIP: termlink not on PATH"; exit 0; }
"$TERMLINK" hub status >/dev/null 2>&1  || { echo "SKIP: no local hub running"; exit 0; }
command -v jq >/dev/null 2>&1           || { echo "SKIP: jq not available"; exit 0; }
command -v sqlite3 >/dev/null 2>&1      || { echo "SKIP: sqlite3 not available"; exit 0; }
command -v python3 >/dev/null 2>&1      || { echo "SKIP: python3 not available"; exit 0; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
J="$tmp/journal.sqlite"
# A dm:-prefixed topic so the default enumeration also finds it.
topic="dm:v6s1test:$$"
cid="cid-v6s1-$$"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

# Seed: two turns on the same conversation_id.
"$TERMLINK" channel post "$topic" --msg-type turn --payload "first message" \
    --metadata conversation_id="$cid" --ensure-topic --json >/dev/null 2>&1
"$TERMLINK" channel post "$topic" --msg-type turn --payload "second message" \
    --metadata conversation_id="$cid" --ensure-topic --json >/dev/null 2>&1

echo "T1: mirror --topic inserts 2 rows"
out="$(bash "$MIRROR" --topic "$topic" --journal "$J" --json 2>&1)"; rc=$?
ins="$(printf '%s' "$out" | jq -r '.rows_inserted' 2>/dev/null)"
{ [ "$rc" -eq 0 ] && [ "$ins" = "2" ]; } && pass "T1 inserted=$ins" || fail "T1 rc=$rc inserted=$ins out=$out"

echo "T2: DB actually holds 2 rows for the topic"
n="$(sqlite3 "$J" "SELECT COUNT(*) FROM messages WHERE topic='$topic';" 2>/dev/null)"
[ "$n" = "2" ] && pass "T2 rows=$n" || fail "T2 rows=$n"

echo "T3: agent-journal <topic> --json returns 2 messages, payload decoded"
out="$(bash "$QUERY" "$topic" --journal "$J" --json 2>&1)"; rc=$?
cnt="$(printf '%s' "$out" | jq -r '.count' 2>/dev/null)"
p0="$(printf '%s' "$out" | jq -r '.messages[0].payload' 2>/dev/null)"
{ [ "$rc" -eq 0 ] && [ "$cnt" = "2" ] && [ "$p0" = "first message" ]; } \
    && pass "T3 count=$cnt payload0='$p0'" || fail "T3 rc=$rc count=$cnt payload0='$p0' out=$out"

echo "T4: idempotent — second mirror pass inserts 0 new rows"
out="$(bash "$MIRROR" --topic "$topic" --journal "$J" --json 2>&1)"
ins2="$(printf '%s' "$out" | jq -r '.rows_inserted' 2>/dev/null)"
n2="$(sqlite3 "$J" "SELECT COUNT(*) FROM messages WHERE topic='$topic';" 2>/dev/null)"
{ [ "$ins2" = "0" ] && [ "$n2" = "2" ]; } && pass "T4 reinserted=$ins2 total=$n2" || fail "T4 reinserted=$ins2 total=$n2"

echo "T5: query by conversation_id also resolves the thread"
cnt="$(bash "$QUERY" "$cid" --journal "$J" --json 2>&1 | jq -r '.count' 2>/dev/null)"
[ "$cnt" = "2" ] && pass "T5 count=$cnt" || fail "T5 count=$cnt"

echo "T6: --since-offset filters"
cnt="$(bash "$QUERY" "$topic" --journal "$J" --since-offset 1 --json 2>&1 | jq -r '.count' 2>/dev/null)"
[ "$cnt" = "1" ] && pass "T6 count=$cnt (offset>=1)" || fail "T6 count=$cnt"

echo "T7: missing journal → exit 2"
bash "$QUERY" "$topic" --journal "$tmp/nope.sqlite" >/dev/null 2>&1; rc=$?
[ "$rc" = "2" ] && pass "T7 exit=$rc" || fail "T7 expected 2 got $rc"

echo "T8: unknown-conversation query → ok, 0 messages"
cnt="$(bash "$QUERY" "dm:nobody:here" --journal "$J" --json 2>&1 | jq -r '.count' 2>/dev/null)"
rc=$?
{ [ "$rc" -eq 0 ] && [ "$cnt" = "0" ]; } && pass "T8 count=$cnt" || fail "T8 rc=$rc count=$cnt"

echo ""
echo "Results: $PASS pass / $FAIL fail"
[ "$FAIL" -eq 0 ]
