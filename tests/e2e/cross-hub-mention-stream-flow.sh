#!/usr/bin/env bash
# T-1397 cross-hub mention + subscribe-streaming end-to-end flow.
#
# Closes the last two primitives in the agent-conversation arc:
#   * --mention (Matrix m.mention, T-1325)
#   * channel subscribe --follow (live tail)
# Verifies both work cross-hub.

set -euo pipefail

BIN=${BIN:-./target/release/termlink}
HUB_107=${HUB_107:-127.0.0.1:9100}
HUB_107_FROM_122=${HUB_107_FROM_122:-192.168.10.107:9100}
N=$RANDOM
TOPIC="xhub-mention-${N}"
WORK=$(mktemp -d -t xhub-mention.XXXXXX)
FOLLOW_OUT="$WORK/follow.jsonl"
FOLLOW_PID=
trap 'rm -rf "$WORK"; [ -n "$FOLLOW_PID" ] && kill "$FOLLOW_PID" 2>/dev/null || true' EXIT

step() { echo; echo "=== $* ==="; }
fail() { echo "FAIL: $*" >&2; exit 1; }

REMOTE_SESSION=$($BIN remote list ring20-management 2>/dev/null | awk '/^tl-/ {print $1; exit}')
[ -n "$REMOTE_SESSION" ] || fail "no live session on ring20-management"

step "Inventory"
"$BIN" --version
echo "Topic: $TOPIC"
echo "Remote: $REMOTE_SESSION"

step "Step 1: Create topic"
"$BIN" channel create "$TOPIC" --hub "$HUB_107" >/dev/null
echo "OK"

step "Step 2: Start subscribe --follow in background BEFORE any posts"
# --follow streams JSON-lines as new envelopes arrive. Capture to a file.
"$BIN" channel subscribe "$TOPIC" --follow --hub "$HUB_107" --json > "$FOLLOW_OUT" 2>&1 &
FOLLOW_PID=$!
echo "follow PID: $FOLLOW_PID"
sleep 1  # let the subscriber establish its long-poll

step "Step 3: 3 local posts on .107 (alice + carol)"
"$BIN" channel post "$TOPIC" --payload "alice: kicking off" --sender-id "alice-107" --hub "$HUB_107" >/dev/null
"$BIN" channel post "$TOPIC" --payload "carol: ack" --sender-id "carol-107" --hub "$HUB_107" >/dev/null
"$BIN" channel post "$TOPIC" --payload "alice: cool" --sender-id "alice-107" --hub "$HUB_107" >/dev/null
echo "OK: 3 local posts"

step "Step 4: bob (.122 cross-hub) posts WITH a mention of alice"
"$BIN" remote exec ring20-management "$REMOTE_SESSION" \
  "termlink channel post '$TOPIC' --payload 'bob: hey @alice can you confirm?' --sender-id 'bob-122' --mention alice --mention carol --hub $HUB_107_FROM_122 --json" \
  >/dev/null 2>&1 || fail "cross-hub mention post failed"
echo "OK: bob posted with mention"

step "Step 5: Wait for follow stream to capture all 4 posts (5s budget)"
DEADLINE=$(($(date +%s) + 5))
while [ "$(wc -l < "$FOLLOW_OUT" 2>/dev/null || echo 0)" -lt "4" ] && [ "$(date +%s)" -lt "$DEADLINE" ]; do
  sleep 0.5
done
LINES=$(wc -l < "$FOLLOW_OUT")
echo "follow received $LINES lines"
[ "$LINES" -ge "4" ] || { tail -20 "$FOLLOW_OUT"; fail "expected >=4 envelopes from follow stream, got $LINES"; }

step "Step 6: Verify follow stream contains the cross-hub post"
HAS_BOB=$(python3 -c '
import json
seen = []
for line in open("'"$FOLLOW_OUT"'"):
    line = line.strip()
    if not line: continue
    try:
        d = json.loads(line)
    except Exception:
        continue
    if d.get("sender_id") == "bob-122":
        seen.append(d.get("payload",""))
print("yes" if seen else "no")
print(seen)
')
echo "$HAS_BOB"
[ "$(echo "$HAS_BOB" | head -1)" = "yes" ] || fail "follow stream missed bob's cross-hub post"

step "Step 7: Verify metadata.mentions on bob's envelope contains 'alice'"
MENTIONS=$(python3 -c '
import json
for line in open("'"$FOLLOW_OUT"'"):
    line = line.strip()
    if not line: continue
    try:
        d = json.loads(line)
    except Exception:
        continue
    if d.get("sender_id") == "bob-122":
        meta = d.get("metadata") or {}
        print(meta.get("mentions","NONE"))
        break
')
echo "bob's envelope mentions: $MENTIONS"
echo "$MENTIONS" | grep -q alice || fail "metadata.mentions does not contain alice"

step "Step 8: Stop follow process cleanly"
kill "$FOLLOW_PID" 2>/dev/null || true
wait "$FOLLOW_PID" 2>/dev/null || true
FOLLOW_PID=
echo "OK: follow stopped"

step "Step 9: Subscribe non-follow with mention filter — find bob's mention by metadata"
# Walk the topic, find envelopes whose mentions include alice
"$BIN" channel subscribe "$TOPIC" --hub "$HUB_107" --json > "$WORK/full.jsonl"
MENTION_HITS=$(python3 -c '
import json
hits = []
for line in open("'"$WORK/full.jsonl"'"):
    line = line.strip()
    if not line: continue
    try:
        d = json.loads(line)
    except Exception:
        continue
    meta = d.get("metadata") or {}
    if "alice" in (meta.get("mentions") or ""):
        hits.append(d.get("sender_id","?") + "@" + str(d.get("offset","?")))
print(",".join(hits))
print(len(hits))
')
HITS=$(echo "$MENTION_HITS" | head -1)
COUNT=$(echo "$MENTION_HITS" | tail -1)
echo "envelopes mentioning alice: $HITS  (count=$COUNT)"
[ "$COUNT" -ge "1" ] || fail "no mention envelopes for alice"
echo "$HITS" | grep -q "bob-122" || fail "bob-122 mention not found"

echo
echo "MENTION-STREAM E2E PASSED"
echo "  Topic:           $TOPIC"
echo "  Posts captured:  $LINES via subscribe --follow"
echo "  Cross-hub post:  visible in stream"
echo "  Mention filter:  matched (alice mentioned by bob-122)"
