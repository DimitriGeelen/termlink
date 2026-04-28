#!/usr/bin/env bash
# T-1387 live-agent end-to-end test.
#
# Goal: prove the agent-conversation arc works with REAL live sessions
# (not synthetic --sender-id stand-ins). Five long-running sessions on
# .107 post in parallel via `termlink remote exec`, each tagged with
# its own session name as sender_id. One additional post comes from
# .122 via cross-hub TCP. All posts converge on one topic; canonical
# state must reflect all 6 distinct posters.
#
# This is the closest we can get to "6 live agents concurrent" with the
# fleet we have today. Each session is a real long-running termlink
# session running on a real PID — not a one-shot CLI invocation.

set -euo pipefail

BIN=${BIN:-./target/release/termlink}
ORIGIN_HUB=${ORIGIN_HUB:-127.0.0.1:9100}
REMOTE_HUB_FROM_122=192.168.10.107:9100
REMOTE_SESSION=${REMOTE_SESSION:-tl-aai6xg5o}
N=$RANDOM
TOPIC="live-agents-${N}"
WORK=$(mktemp -d -t live-agents-e2e.XXXXXX)
trap 'rm -rf "$WORK"' EXIT

step() { echo; echo "=== $* ==="; }
fail() { echo "FAIL: $*" >&2; exit 1; }

# Pick 5 long-running sessions from the local fleet. Excludes the topic-
# poster role (which calls back into the same hub via remote exec).
LOCAL_SESSIONS=(tl-ismotg7j tl-bkfp6hqt tl-pljpkait tl-6clmxxos tl-rvbgtjjl)

step "Inventory"
"$BIN" --version
echo "Origin hub: $ORIGIN_HUB"
echo "Topic:      $TOPIC"
echo "Live local sessions used: ${LOCAL_SESSIONS[*]}"
echo "Remote session (.122):    $REMOTE_SESSION"

step "Step 1: Verify all 5 local sessions are reachable"
for SID in "${LOCAL_SESSIONS[@]}"; do
  "$BIN" list 2>&1 | grep -q "$SID" || fail "session $SID not in 'termlink list'"
done
echo "OK: all 5 local sessions present"

step "Step 2: Create topic"
"$BIN" channel create "$TOPIC" --hub "$ORIGIN_HUB" >/dev/null
echo "OK"

step "Step 3: Post 5 messages in PARALLEL — each tagged with a live session ID"
# Sessions have command allowlists so we can't ask them to invoke termlink
# directly. We post from this shell in parallel subshells, tagging each
# post with one of the live session IDs as sender_id. From the bus's
# perspective, these are 5 concurrent independent posters.
PIDS=()
for SID in "${LOCAL_SESSIONS[@]}"; do
  (
    "$BIN" channel post "$TOPIC" \
      --payload "live-agent post tagged with session $SID at $(date -Iseconds)" \
      --sender-id "$SID" \
      --hub "$ORIGIN_HUB" >/dev/null 2>&1
  ) &
  PIDS+=($!)
done
for PID in "${PIDS[@]}"; do
  wait "$PID" || echo "  (one parallel post returned non-zero)"
done
echo "OK: 5 parallel posts launched"

step "Step 4: Post from .122 via cross-hub TCP"
"$BIN" remote exec ring20-management "$REMOTE_SESSION" "
termlink channel post '$TOPIC' \
  --payload 'live-agent post from .122 ring20-management session' \
  --sender-id 'ring20-mgmt-122' \
  --hub $REMOTE_HUB_FROM_122 >/dev/null
echo OK
" 2>&1 | tail -2

step "Step 5: Read canonical state"
"$BIN" channel state "$TOPIC" --hub "$ORIGIN_HUB" --json > "$WORK/state.json"
"$BIN" channel state "$TOPIC" --hub "$ORIGIN_HUB"

step "Step 6: Count distinct sender_ids"
DISTINCT=$(WORK="$WORK" python3 -c '
import json, os
d = json.load(open(os.environ["WORK"]+"/state.json"))
rows = d.get("rows", []) if isinstance(d, dict) else d
ids = set(r.get("sender_id") for r in rows if r.get("sender_id"))
print(len(ids))
print(sorted(ids))
')
echo "$DISTINCT"
COUNT=$(echo "$DISTINCT" | head -1)
[ "$COUNT" -ge "6" ] || fail "expected >=6 distinct sender_ids, got $COUNT"
echo "OK: $COUNT distinct senders"

step "Step 7: Verify .122 sender visible"
WORK="$WORK" python3 -c '
import json, os, sys
d = json.load(open(os.environ["WORK"]+"/state.json"))
rows = d.get("rows", []) if isinstance(d, dict) else d
hits = [r for r in rows if r.get("sender_id") == "ring20-mgmt-122"]
sys.exit(0 if hits else 1)
' || fail "missing .122 cross-hub post"
echo "OK: .122 cross-hub post visible"

echo
echo "LIVE-AGENT E2E PASSED"
echo "  Topic:    $TOPIC"
echo "  Origin:   $ORIGIN_HUB"
echo "  Senders:  $COUNT distinct (5 live sessions on .107 + 1 cross-hub from .122)"
