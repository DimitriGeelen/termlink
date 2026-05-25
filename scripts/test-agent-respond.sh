#!/usr/bin/env bash
# T-1805 — round-trip smoke test for the doorbell+mail loop.
#
# Pairs the REAL scripts/agent-send.sh (T-1804) with the REAL
# scripts/agent-respond.sh (T-1805) on a local hub — no second live agent:
#
#   A) send waits; respond posts the receipt for the same cid  -> send rc 0, DELIVERED
#   B) send waits; nobody responds                             -> send rc 3, FAILED
#
# Path A is the load-bearing proof: agent-respond.sh's receipt is the exact shape
# agent-send.sh polls for, so the sender learns delivery. The doorbell (inject)
# targets a non-existent session on purpose (non-fatal); the receipt is under test.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SEND="$HERE/agent-send.sh"
RESPOND="$HERE/agent-respond.sh"
TERMLINK="${TERMLINK_BIN:-termlink}"

command -v "$TERMLINK" >/dev/null 2>&1 || { echo "SKIP: termlink not on PATH"; exit 0; }
"$TERMLINK" hub status >/dev/null 2>&1   || { echo "SKIP: no local hub running"; exit 0; }
command -v jq >/dev/null 2>&1            || { echo "SKIP: jq not available"; exit 0; }

topic="agent-respond-test-$$"
nosess="no-such-session-$$"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fail=0

# --- Path A: respond posts the receipt -> send sees DELIVERED, exit 0 ---
cidA="cidA-$$"
(
    "$SEND" --to-session "$nosess" --topic "$topic" --message "hello A" \
            --conversation-id "$cidA" --timeout 8 --max-rings 3 >"$tmp/A.out" 2>&1
    echo $? >"$tmp/A.rc"
) &
bg=$!
sleep 2
# the receiver wakes and acks (with an actual reply, exercising both posts)
"$RESPOND" --topic "$topic" --conversation-id "$cidA" --reply "ack A" >"$tmp/R.out" 2>&1 \
    || { echo "FAIL A: agent-respond.sh errored"; sed 's/^/  R| /' "$tmp/R.out"; fail=1; }
wait "$bg" || true
rcA="$(cat "$tmp/A.rc" 2>/dev/null || echo X)"
if [ "$rcA" = "0" ] && grep -q "DELIVERED" "$tmp/A.out" && grep -q "receipt posted" "$tmp/R.out"; then
    echo "PASS A: respond -> send delivered (rc=0)"
else
    echo "FAIL A: expected rc=0 + DELIVERED + receipt posted (got rc=$rcA)"
    sed 's/^/  A| /' "$tmp/A.out"; sed 's/^/  R| /' "$tmp/R.out"; fail=1
fi

# --- Path B: no responder -> send FAILED, non-zero ---
cidB="cidB-$$"
set +e
"$SEND" --to-session "$nosess" --topic "$topic" --message "hello B" \
        --conversation-id "$cidB" --timeout 2 --max-rings 2 >"$tmp/B.out" 2>&1
rcB=$?
set -e
if [ "$rcB" != "0" ] && grep -q "FAILED" "$tmp/B.out"; then
    echo "PASS B: no responder -> send failed (rc=$rcB)"
else
    echo "FAIL B: expected non-zero + FAILED (got rc=$rcB)"; sed 's/^/  B| /' "$tmp/B.out"; fail=1
fi

# --- Arg validation: missing cid -> rc 2 ---
set +e
"$RESPOND" --topic "$topic" >"$tmp/V.out" 2>&1
rcV=$?
set -e
if [ "$rcV" = "2" ] && grep -q "missing --conversation-id" "$tmp/V.out"; then
    echo "PASS V: missing --conversation-id -> rc 2"
else
    echo "FAIL V: expected rc 2 on missing cid (got rc=$rcV)"; sed 's/^/  V| /' "$tmp/V.out"; fail=1
fi

if [ "$fail" = "0" ]; then echo "test-agent-respond: ALL PASS"; else echo "test-agent-respond: FAILURES"; fi
exit "$fail"
