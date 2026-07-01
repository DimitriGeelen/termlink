#!/usr/bin/env bash
# T-2300 (arc-003 reliable-comms, V6 slice S3) — tests for the notify sidecar's
# --auto-confirm mode (recipient-side journaled receipt) + agent-send.sh's
# stage-aware DELIVERED line.
#
# No second host. On this shared host every post signs with the same identity, so
# genuine "unread" is manufactured the way the hub computes it (channel unread =
# content past the reader's own latest m.receipt.up_to): post turn1, a receipt at
# up_to=0, then turn2 — turn2 is now unread. TERMLINK_NOTIFY_TEST_TOPICS scopes the
# sidecar's enumeration to the throwaway topic so production dm: topics are never
# touched. TERMLINK_JOURNAL_PATH isolates the S1 journal store.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SIDECAR="$HERE/notify-sidecar.sh"
SEND="$HERE/agent-send.sh"
TERMLINK="${TERMLINK_BIN:-termlink}"

command -v "$TERMLINK" >/dev/null 2>&1 || { echo "SKIP: termlink not on PATH"; exit 0; }
"$TERMLINK" hub status >/dev/null 2>&1  || { echo "SKIP: no local hub running"; exit 0; }
command -v jq >/dev/null 2>&1           || { echo "SKIP: jq not available"; exit 0; }
command -v sqlite3 >/dev/null 2>&1      || { echo "SKIP: sqlite3 not available"; exit 0; }

selffp="$("$TERMLINK" channel info agent-presence --json 2>/dev/null | jq -r '.senders[0].sender_id // empty')"
[ -n "$selffp" ] || { echo "SKIP: cannot resolve self sender_id from local hub"; exit 0; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
export TERMLINK_JOURNAL_PATH="$tmp/journal.sqlite"
export TERMLINK_NOTIFY_DIR="$tmp/notify"

PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

# Seed a dm: topic with genuine unread for self (turn1 / receipt up_to=0 / turn2).
seed_topic() {
    local t="$1"
    "$TERMLINK" channel post "$t" --msg-type turn --payload "turn1" --ensure-topic --json >/dev/null 2>&1
    "$TERMLINK" channel post "$t" --msg-type receipt --metadata up_to=0 --json >/dev/null 2>&1
    "$TERMLINK" channel post "$t" --msg-type turn --payload "turn2" --json >/dev/null 2>&1
}
delivered_count() {
    "$TERMLINK" channel subscribe "$1" --cursor 0 --limit 300 --json 2>/dev/null \
        | jq -s '[ .[] | select(.msg_type=="receipt" and .metadata.stage=="delivered") ] | length'
}
run_sidecar() {  # $1=topic ; passes remaining args through
    local t="$1"; shift
    TERMLINK_NOTIFY_TEST_TOPICS="$t" bash "$SIDECAR" --agent-id s3test --self-fp "$selffp" --once "$@" >/dev/null 2>&1
}

# -------- T1: default OFF → no receipt, no journal write (V3a byte-for-byte) --------
echo "T1: no --auto-confirm → no stage=delivered receipt, no journal row"
t1="dm:s3t1-$$:${selffp}"; seed_topic "$t1"
run_sidecar "$t1"                         # NO --auto-confirm
dc="$(delivered_count "$t1")"
jr="$(sqlite3 "$TERMLINK_JOURNAL_PATH" "SELECT count(*) FROM messages WHERE topic='$t1';" 2>/dev/null || echo 0)"
{ [ "$dc" = "0" ] && [ "${jr:-0}" = "0" ]; } && pass "T1 default-off is a no-op (receipts=$dc journal=$jr)" || fail "T1 receipts=$dc journal=$jr"

# -------- T2: --auto-confirm → journal row written --------
echo "T2: --auto-confirm mirrors the topic into the S1 journal"
t2="dm:s3t2-$$:${selffp}"; seed_topic "$t2"
run_sidecar "$t2" --auto-confirm
jr="$(sqlite3 "$TERMLINK_JOURNAL_PATH" "SELECT count(*) FROM messages WHERE topic='$t2';" 2>/dev/null || echo 0)"
[ "${jr:-0}" -ge 1 ] && pass "T2 journal populated (rows=$jr)" || fail "T2 journal rows=$jr (expected >=1)"

# -------- T3: --auto-confirm → stage=delivered receipt at the content watermark --------
echo "T3: --auto-confirm posts a mechanism-A stage=delivered receipt"
dc="$(delivered_count "$t2")"
upto="$("$TERMLINK" channel subscribe "$t2" --cursor 0 --limit 300 --json 2>/dev/null \
        | jq -s -r '[ .[] | select(.metadata.stage=="delivered") ][0].metadata.up_to // empty')"
{ [ "$dc" = "1" ] && [ "$upto" = "2" ]; } && pass "T3 one stage=delivered receipt, up_to=$upto (content watermark)" || fail "T3 receipts=$dc up_to=$upto"

# -------- T4: offset guard → re-run posts NO duplicate receipt --------
echo "T4: second --once on unchanged topic posts no duplicate receipt"
run_sidecar "$t2" --auto-confirm
dc2="$(delivered_count "$t2")"
[ "$dc2" = "1" ] && pass "T4 offset guard held (receipts still $dc2)" || fail "T4 receipts=$dc2 (expected 1 — duplicate posted)"

# -------- T5: agent-send.sh surfaces the receipt stage in the DELIVERED line --------
echo "T5: agent-send.sh DELIVERED line surfaces stage=delivered"
t5="dm:s3t5-$$:${selffp}"
cid="cid-s3t5-$$"
(
    set +e
    "$SEND" --to-session "no-such-$$" --topic "$t5" --message "ping S5" \
            --conversation-id "$cid" --timeout 6 --max-rings 2 >"$tmp/t5.out" 2>&1
    echo $? >"$tmp/t5.rc"
) &
bg=$!
sleep 1
# offset of the turn agent-send just posted
o5="$("$TERMLINK" channel subscribe "$t5" --conversation-id "$cid" --cursor 0 --limit 200 --json 2>/dev/null \
        | jq -s '[.[]|select(.msg_type=="turn")][0].offset // 0')"
# ack it WITH a stage=delivered tag (the mechanism-A shape the sidecar produces)
"$TERMLINK" channel post "$t5" --msg-type receipt --metadata conversation_id="$cid" \
            --metadata up_to="$o5" --metadata stage=delivered --ensure-topic --json >/dev/null 2>&1
wait "$bg" || true
rc5="$(cat "$tmp/t5.rc" 2>/dev/null || echo X)"
if [ "$rc5" = "0" ] && grep -q "DELIVERED (stage=delivered)" "$tmp/t5.out"; then
    pass "T5 stage surfaced in DELIVERED line (rc=0)"
else
    fail "T5 rc=$rc5 out=$(tr '\n' '|' <"$tmp/t5.out")"
fi

echo ""
echo "Results: $PASS pass / $FAIL fail"
[ "$FAIL" -eq 0 ]
