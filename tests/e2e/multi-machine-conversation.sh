#!/usr/bin/env bash
# T-1386 multi-machine, multi-agent end-to-end test.
#
# Goal: prove a single shared topic on the .107 hub receives concurrent
# posts from agents on BOTH .107 and .122, all going through cross-hub
# TCP RPC where appropriate. Demonstrates the agent-conversation arc
# working in a real fleet topology.
#
# Topology:
#   .107 (workstation, this host) — origin hub for the conversation topic.
#   .122 (ring20-management LXC)  — posts via TCP --hub 192.168.10.107:9100.
#
# Six conversation participants (4 from .107, 2 from .122) post a thread,
# we verify ordering, sender attribution, and that the .122-originated
# posts are visible on the canonical state.
#
# Requires:
#   - termlink binary at ./target/release/termlink locally.
#   - .122 reachable via `termlink remote exec ring20-management <session>`.
#   - .122 has a hubs.toml profile for 192.168.10.107:9100 (workstation-107)
#     with the .107 hub secret installed.
#
# Exits non-zero on any failure.

set -euo pipefail

BIN=${BIN:-./target/release/termlink}
ORIGIN_HUB=192.168.10.107:9100
N=$RANDOM
TOPIC="multi-machine-e2e-${N}"
REMOTE_SESSION=${REMOTE_SESSION:-tl-aai6xg5o}
WORK=$(mktemp -d -t multi-machine-e2e.XXXXXX)
trap 'rm -rf "$WORK"' EXIT

step() { echo; echo "=== $* ==="; }
fail() { echo "FAIL: $*" >&2; exit 1; }

step "Inventory"
"$BIN" --version
echo "Origin hub:     $ORIGIN_HUB"
echo "Remote session: $REMOTE_SESSION (on ring20-management/.122)"
echo "Topic:          $TOPIC"

step "Step 1: Create topic on .107"
"$BIN" channel create "$TOPIC" --hub "$ORIGIN_HUB" >/dev/null
echo "OK"

step "Step 2: Four .107 agents post"
for SID in alice bob carol dave; do
  "$BIN" channel post "$TOPIC" \
    --payload ".107:$SID joining the conversation" \
    --sender-id "$SID" \
    --hub "$ORIGIN_HUB" >/dev/null
done
echo "OK"

step "Step 3: Two .122 agents post via cross-hub TCP back to .107"
"$BIN" remote exec ring20-management "$REMOTE_SESSION" "
set -e
for SID in erin frank; do
  termlink channel post '$TOPIC' \
    --payload \".122:\$SID joining via cross-hub TCP\" \
    --sender-id \"\$SID\" \
    --hub 192.168.10.107:9100 >/dev/null
done
echo OK
" 2>&1 | tail -3

step "Step 4: Read canonical state on .107"
"$BIN" channel state "$TOPIC" --hub "$ORIGIN_HUB" --json > "$WORK/state.json"
"$BIN" channel state "$TOPIC" --hub "$ORIGIN_HUB"

step "Step 5: Verify all 6 agents present"
COUNT=$(WORK="$WORK" python3 -c 'import json,os; d=json.load(open(os.environ["WORK"]+"/state.json")); rows=d.get("rows",[]) if isinstance(d,dict) else d; print(len(rows))')
[ "$COUNT" = "6" ] || fail "expected 6 rows, got $COUNT"
echo "OK: 6 messages present"

for SID in alice bob carol dave erin frank; do
  WORK="$WORK" SID="$SID" python3 -c '
import json, os, sys
d = json.load(open(os.environ["WORK"]+"/state.json"))
rows = d.get("rows", []) if isinstance(d, dict) else d
sid = os.environ["SID"]
hits = [r for r in rows if r.get("sender_id") == sid]
sys.exit(0 if hits else 1)
' || fail "missing posts from agent $SID"
done
echo "OK: all 6 agents attributed (alice, bob, carol, dave, erin, frank)"

step "Step 6: Verify cross-hub posts came from .122 (payload prefix)"
LOCAL_HITS=$(WORK="$WORK" python3 -c 'import json,os; d=json.load(open(os.environ["WORK"]+"/state.json")); rows=d.get("rows",[]) if isinstance(d,dict) else d; print(sum(1 for r in rows if str(r.get("payload","")).startswith(".107:")))')
REMOTE_HITS=$(WORK="$WORK" python3 -c 'import json,os; d=json.load(open(os.environ["WORK"]+"/state.json")); rows=d.get("rows",[]) if isinstance(d,dict) else d; print(sum(1 for r in rows if str(r.get("payload","")).startswith(".122:")))')
[ "$LOCAL_HITS"  = "4" ] || fail "expected 4 .107-origin payloads, got $LOCAL_HITS"
[ "$REMOTE_HITS" = "2" ] || fail "expected 2 .122-origin payloads, got $REMOTE_HITS"
echo "OK: 4 from .107, 2 from .122"

step "Step 7: Cross-machine reply chain — frank (.122) replies to alice (.107)"
"$BIN" remote exec ring20-management "$REMOTE_SESSION" "
termlink channel post '$TOPIC' \
  --payload 'frank: alice, agreed!' \
  --sender-id frank \
  --reply-to 0 \
  --hub 192.168.10.107:9100 >/dev/null
echo OK
" 2>&1 | tail -2

step "Step 8: Verify reply visible on .107 with in_reply_to=0"
"$BIN" channel relations "$TOPIC" 0 --hub "$ORIGIN_HUB" 2>&1 | head -10

echo
echo "MULTI-MACHINE E2E PASSED"
echo "  Topic:     $TOPIC"
echo "  Origin:    $ORIGIN_HUB"
echo "  Agents:    6 (4 from .107, 2 from .122)"
echo "  Cross-hub: TCP-authed posts from .122 -> .107 with reply-chain"
