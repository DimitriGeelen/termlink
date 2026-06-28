#!/usr/bin/env bash
# T-1804 — smoke test for scripts/agent-send.sh against a local hub.
#
# No second agent required: we self-post the receipt to simulate the receiver's
# ack. Proves BOTH deterministic paths:
#   A) receipt appears while waiting  -> exit 0, "DELIVERED"
#   B) no receipt                     -> exit non-zero, "FAILED", re-rings capped
#
# The doorbell (inject) targets a non-existent session on purpose, exercising
# the non-fatal inject path — the receipt logic is what's under test.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SEND="$HERE/agent-send.sh"
TERMLINK="${TERMLINK_BIN:-termlink}"

command -v "$TERMLINK" >/dev/null 2>&1 || { echo "SKIP: termlink not on PATH"; exit 0; }
"$TERMLINK" hub status >/dev/null 2>&1   || { echo "SKIP: no local hub running"; exit 0; }
command -v jq >/dev/null 2>&1            || { echo "SKIP: jq not available"; exit 0; }

topic="agent-send-test-$$"
nosess="no-such-session-$$"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fail=0

# --- Path A: a receipt arrives during the wait -> DELIVERED, exit 0 ---
cidA="cidA-$$"
(
    "$SEND" --to-session "$nosess" --topic "$topic" --message "hello A" \
            --conversation-id "$cidA" --timeout 6 --max-rings 2 >"$tmp/A.out" 2>&1
    echo $? >"$tmp/A.rc"
) &
bg=$!
sleep 1
# simulate the receiver acking this conversation
"$TERMLINK" channel post "$topic" --msg-type receipt --metadata conversation_id="$cidA" \
            --metadata up_to=0 --ensure-topic --json >/dev/null
wait "$bg" || true
rcA="$(cat "$tmp/A.rc" 2>/dev/null || echo X)"
if [ "$rcA" = "0" ] && grep -q "DELIVERED" "$tmp/A.out"; then
    echo "PASS A: delivered on receipt (rc=0)"
else
    echo "FAIL A: expected rc=0 + DELIVERED (got rc=$rcA)"; sed 's/^/  A| /' "$tmp/A.out"; fail=1
fi

# --- Path B: no receipt -> FAILED, non-zero, exactly max-rings rings ---
cidB="cidB-$$"
set +e
"$SEND" --to-session "$nosess" --topic "$topic" --message "hello B" \
        --conversation-id "$cidB" --timeout 2 --max-rings 3 >"$tmp/B.out" 2>&1
rcB=$?
set -e
rings="$(grep -cE "ring [0-9]+/3 ->" "$tmp/B.out" || true)"
if [ "$rcB" != "0" ] && grep -q "FAILED" "$tmp/B.out" && [ "$rings" = "3" ]; then
    echo "PASS B: not acked -> rc=$rcB after $rings rings"
else
    echo "FAIL B: expected non-zero + FAILED + 3 rings (got rc=$rcB, rings=$rings)"; sed 's/^/  B| /' "$tmp/B.out"; fail=1
fi

# --- Path C (T-1808): a STALE receipt from an earlier turn on the same cid must
#     NOT satisfy a later turn's wait. Pre-ack turn-1, then send turn-2 unacked. ---
cidC="cidC-$$"
"$TERMLINK" channel post "$topic" --msg-type turn --payload "turn1 C" \
            --metadata conversation_id="$cidC" --ensure-topic --json >/dev/null
o1="$("$TERMLINK" channel subscribe "$topic" --conversation-id "$cidC" --cursor 0 --limit 100 --json 2>/dev/null \
        | jq -s '[.[]|select(.msg_type=="turn")][0].offset // 0')"
# ack turn-1 only (receipt up_to = turn-1 offset)
"$TERMLINK" channel post "$topic" --msg-type receipt \
            --metadata conversation_id="$cidC" --metadata up_to="$o1" --ensure-topic --json >/dev/null
set +e
"$SEND" --to-session "$nosess" --topic "$topic" --message "turn2 C" \
        --conversation-id "$cidC" --timeout 2 --max-rings 2 >"$tmp/C.out" 2>&1
rcC=$?
set -e
if [ "$rcC" != "0" ] && grep -q "FAILED" "$tmp/C.out"; then
    echo "PASS C: stale turn-1 receipt did NOT false-deliver turn-2 (rc=$rcC)"
else
    echo "FAIL C: stale receipt caused false DELIVERED for turn-2 (got rc=$rcC)"; sed 's/^/  C| /' "$tmp/C.out"; fail=1
fi

# Helper: offset of the turn this cid posted (for an offset-correct receipt ack).
turn_offset() {
    "$TERMLINK" channel subscribe "$topic" --conversation-id "$1" --cursor 0 --limit 200 --json 2>/dev/null \
        | jq -s '[.[]|select(.msg_type=="turn")][0].offset // 0'
}

# --- Path D (T-1811): --await-reply -> delivered AND a reply turn arrives ->
#     exit 0, the reply payload is printed. ---
cidD="cidD-$$"
(
    set +e  # capture rc even when $SEND exits non-zero (parent runs set -e)
    "$SEND" --to-session "$nosess" --topic "$topic" --message "ping D" \
            --conversation-id "$cidD" --timeout 6 --max-rings 2 --await-reply 8 \
            >"$tmp/D.out" 2>&1
    echo $? >"$tmp/D.rc"
) &
bgD=$!
sleep 1
oD="$(turn_offset "$cidD")"
# ack the turn (offset-correct), then post the peer's reply turn.
"$TERMLINK" channel post "$topic" --msg-type receipt \
            --metadata conversation_id="$cidD" --metadata up_to="$oD" --ensure-topic --json >/dev/null
"$TERMLINK" channel post "$topic" --msg-type turn --payload "PONG_D_REPLY" \
            --metadata conversation_id="$cidD" --ensure-topic --json >/dev/null
wait "$bgD" || true
rcD="$(cat "$tmp/D.rc" 2>/dev/null || echo X)"
if [ "$rcD" = "0" ] && grep -q "REPLY at offset=" "$tmp/D.out" && grep -q "PONG_D_REPLY" "$tmp/D.out"; then
    echo "PASS D: delivered + reply round-trip (rc=0, payload printed)"
else
    echo "FAIL D: expected rc=0 + reply payload (got rc=$rcD)"; sed 's/^/  D| /' "$tmp/D.out"; fail=1
fi

# --- Path E (T-1811): --await-reply -> delivered but NO reply turn -> exit 4. ---
cidE="cidE-$$"
(
    set +e  # capture rc even when $SEND exits non-zero (parent runs set -e)
    "$SEND" --to-session "$nosess" --topic "$topic" --message "ping E" \
            --conversation-id "$cidE" --timeout 6 --max-rings 2 --await-reply 3 \
            >"$tmp/E.out" 2>&1
    echo $? >"$tmp/E.rc"
) &
bgE=$!
sleep 1
oE="$(turn_offset "$cidE")"
# ack only — never post a reply.
"$TERMLINK" channel post "$topic" --msg-type receipt \
            --metadata conversation_id="$cidE" --metadata up_to="$oE" --ensure-topic --json >/dev/null
wait "$bgE" || true
rcE="$(cat "$tmp/E.rc" 2>/dev/null || echo X)"
if [ "$rcE" = "4" ] && grep -q "DELIVERED but no reply" "$tmp/E.out"; then
    echo "PASS E: delivered, no reply -> rc=4 (distinct from not-acked rc=3)"
else
    echo "FAIL E: expected rc=4 + 'no reply' (got rc=$rcE)"; sed 's/^/  E| /' "$tmp/E.out"; fail=1
fi

# --- Path F (T-2295/V3b): --no-await-ack -> POSTED, exit 0, NO doorbell rings,
#     NO receipt wait (fire-and-forget opt-out). ---
cidF="cidF-$$"
set +e
"$SEND" --to-session "$nosess" --topic "$topic" --message "fire-and-forget F" \
        --conversation-id "$cidF" --no-await-ack >"$tmp/F.out" 2>&1
rcF=$?
set -e
ringsF="$(grep -cE "ring [0-9]+/" "$tmp/F.out" || true)"
if [ "$rcF" = "0" ] && grep -q "POSTED" "$tmp/F.out" && [ "$ringsF" = "0" ]; then
    echo "PASS F: --no-await-ack -> POSTED rc=0, 0 rings (fire-and-forget)"
else
    echo "FAIL F: expected rc=0 + POSTED + 0 rings (got rc=$rcF, rings=$ringsF)"; sed 's/^/  F| /' "$tmp/F.out"; fail=1
fi

# --- Path G (T-2295/V3b): --no-await-ack is mutex with --await-reply -> exit 2. ---
set +e
"$SEND" --to-session "$nosess" --topic "$topic" --message "bad G" \
        --no-await-ack --await-reply 3 >"$tmp/G.out" 2>&1
rcG=$?
set -e
if [ "$rcG" = "2" ] && grep -q "mutex with --await-reply" "$tmp/G.out"; then
    echo "PASS G: --no-await-ack + --await-reply rejected (rc=2)"
else
    echo "FAIL G: expected rc=2 + mutex message (got rc=$rcG)"; sed 's/^/  G| /' "$tmp/G.out"; fail=1
fi

if [ "$fail" = "0" ]; then echo "test-agent-send: ALL PASS"; else echo "test-agent-send: FAILURES"; fi
exit "$fail"
