#!/usr/bin/env bash
# T-2589 — load-bearing fixtures for scripts/check-outbox.sh outbound_unread.
#
# Drives the whole wrapper through a mock `termlink` (via TERMLINK_BIN) and
# asserts the per-envelope tail-scan count. These tests FAIL against the old
# `count - 1 - peer_acked` approximation (temp-revert proven) and PASS against
# the T-2589 fix — making the "caught-up-is-reachable" invariant load-bearing.
#
# No live hub, no network. Exit 0 = all pass, 1 = a failure.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/../scripts/check-outbox.sh"
[ -f "$SCRIPT" ] || { echo "FAIL: $SCRIPT not found"; exit 1; }

SELF="aaaa000011112222"
PEER="bbbb333344445555"
TOPIC="dm:${SELF}:${PEER}"

WORK="$(mktemp -d -t check-outbox-fixtures.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

# ---- Build the mock termlink -------------------------------------------------
# Dispatches on the full arg string. Emits canned JSON for the four calls the
# wrapper makes. Envelope stream (subscribe) is switched by MOCK_MODE:
#   partial  — count=21, peer_acked=12, 2 self-authored non-receipt posts unacked
#   caughtup — peer acked everything, only a peer receipt sits past the offset
MOCK="$WORK/termlink"
cat > "$MOCK" <<MOCKEOF
#!/usr/bin/env bash
SELF="$SELF"
PEER="$PEER"
TOPIC="$TOPIC"
args="\$*"
case "\$args" in
  "channel info agent-presence --json"*)
    printf '%s\n' "{\"senders\":[{\"sender_id\":\"\$SELF\",\"posts\":5}]}"
    ;;
  "channel list --prefix dm: --json"*)
    printf '%s\n' "{\"channels\":[{\"name\":\"\$TOPIC\"}]}"
    ;;
  "channel info \$TOPIC --json"*)
    if [ "\${MOCK_MODE:-partial}" = "caughtup" ]; then
      # count=9: offsets 0..8. Peer acked up_to=7; the receipt that set that
      # ack is itself stored at offset 8 (> 7), so it inflates count. The old
      # formula count-1-peer_acked = 9-1-7 = 1 (the always->=1 false positive);
      # the fix correctly reports 0 (no self posts past offset 7).
      printf '%s\n' "{\"count\":9,\"senders\":[{\"sender_id\":\"\$SELF\",\"posts\":3}],\"receipts\":[{\"sender_id\":\"\$PEER\",\"up_to\":7}]}"
    else
      printf '%s\n' "{\"count\":21,\"senders\":[{\"sender_id\":\"\$SELF\",\"posts\":3}],\"receipts\":[{\"sender_id\":\"\$PEER\",\"up_to\":12}]}"
    fi
    ;;
  "channel subscribe \$TOPIC "*)
    if [ "\${MOCK_MODE:-partial}" = "caughtup" ]; then
      # Only the peer's own receipt sits past the acked offset — 0 self posts.
      printf '%s\n' "{\"offset\":8,\"sender_id\":\"\$PEER\",\"msg_type\":\"receipt\"}"
    else
      # offsets 13..20 (the tail past peer_acked=12). Self non-receipt: 15, 18.
      printf '%s\n' "{\"offset\":13,\"sender_id\":\"\$PEER\",\"msg_type\":\"chat\"}"
      printf '%s\n' "{\"offset\":14,\"sender_id\":\"\$PEER\",\"msg_type\":\"receipt\"}"
      printf '%s\n' "{\"offset\":15,\"sender_id\":\"\$SELF\",\"msg_type\":\"chat\"}"
      printf '%s\n' "{\"offset\":16,\"sender_id\":\"\$SELF\",\"msg_type\":\"receipt\"}"
      printf '%s\n' "{\"offset\":17,\"sender_id\":\"\$PEER\",\"msg_type\":\"chat\"}"
      printf '%s\n' "{\"offset\":18,\"sender_id\":\"\$SELF\",\"msg_type\":\"note\"}"
      printf '%s\n' "{\"offset\":19,\"sender_id\":\"\$PEER\",\"msg_type\":\"reaction\"}"
      printf '%s\n' "{\"offset\":20,\"sender_id\":\"\$SELF\",\"msg_type\":\"receipt\"}"
    fi
    ;;
  *)
    # Unhandled call — emit nothing, exit 0 (wrapper tolerates empty).
    :
    ;;
esac
MOCKEOF
chmod +x "$MOCK"

fail=0
pass() { echo "PASS: $1"; }
bad()  { echo "FAIL: $1"; fail=1; }

# ---- Test 1: partial ack — exact self-authored count (2), NOT the old 8 ------
out="$(MOCK_MODE=partial TERMLINK_BIN="$MOCK" bash "$SCRIPT" --json 2>/dev/null)"
unread="$(printf '%s' "$out" | jq -r '.topics[0].outbound_unread // "MISSING"' 2>/dev/null)"
if [ "$unread" = "2" ]; then
  pass "partial ack → outbound_unread=2 (self-authored non-receipt posts beyond peer_acked)"
else
  bad "partial ack → expected outbound_unread=2, got '$unread' (old formula count-1-peer_acked would give 8)"
  printf '  raw: %s\n' "$out"
fi

# ---- Test 2: caught up — outbound_unread=0, topic drops out ------------------
out="$(MOCK_MODE=caughtup TERMLINK_BIN="$MOCK" bash "$SCRIPT" --json 2>/dev/null)"
nr="$(printf '%s' "$out" | jq -r '.summary.topics_with_unread // "MISSING"' 2>/dev/null)"
if [ "$nr" = "0" ]; then
  pass "caught up → 0 topics with unread (all-caught-up state is reachable)"
else
  bad "caught up → expected topics_with_unread=0, got '$nr' (old formula would keep the topic via the peer's own receipt envelope)"
  printf '  raw: %s\n' "$out"
fi

# ---- Test 3: caught up — human-format renders the caught-up line -------------
out="$(MOCK_MODE=caughtup TERMLINK_BIN="$MOCK" bash "$SCRIPT" 2>/dev/null)"
if printf '%s' "$out" | grep -q "all peers caught up"; then
  pass "caught up → human output shows 'all peers caught up'"
else
  bad "caught up → human output missing 'all peers caught up'"
  printf '  raw: %s\n' "$out"
fi

echo
if [ "$fail" -eq 0 ]; then
  echo "check-outbox-fixtures: ALL PASS"
  exit 0
else
  echo "check-outbox-fixtures: FAILURES"
  exit 1
fi
