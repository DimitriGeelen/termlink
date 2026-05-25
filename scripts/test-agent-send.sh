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

if [ "$fail" = "0" ]; then echo "test-agent-send: ALL PASS"; else echo "test-agent-send: FAILURES"; fi
exit "$fail"
