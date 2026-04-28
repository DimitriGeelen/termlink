#!/usr/bin/env bash
# T-1394 cross-hub DM end-to-end flow.
#
# Verifies the Matrix-style direct-message primitive (`channel dm`)
# works across the .107 + .122 hub boundary. Two agents (alice on .107
# + bob on .122) exchange messages on the canonical `dm:<a>:<b>` topic,
# the topic name resolved once via `--topic-only` so both sides agree.

set -euo pipefail

BIN=${BIN:-./target/release/termlink}
HUB_107=${HUB_107:-127.0.0.1:9100}
HUB_122=${HUB_122:-192.168.10.122:9100}
HUB_107_FROM_122=${HUB_107_FROM_122:-192.168.10.107:9100}
WORK=$(mktemp -d -t xhub-dm.XXXXXX)
trap 'rm -rf "$WORK"' EXIT

step() { echo; echo "=== $* ==="; }
fail() { echo "FAIL: $*" >&2; exit 1; }

REMOTE_SESSION=$($BIN remote list ring20-management 2>/dev/null | awk '/^tl-/ {print $1; exit}')
[ -n "$REMOTE_SESSION" ] || fail "no live session on ring20-management"

# Use a unique peer-id pair per run so we don't collide with prior runs
# of the same script — DM topics persist `forever` retention by default.
PEER_BOB="bob-122-$$"
PEER_ALICE="alice-107-$$"

step "Inventory"
"$BIN" --version
echo "Peer: $PEER_BOB (alice's view from .107)"
echo "Peer: $PEER_ALICE (bob's view from .122)"

step "Step 1: Resolve canonical DM topic from .107 (alice's view)"
DM_TOPIC=$("$BIN" channel dm "$PEER_BOB" --topic-only --hub "$HUB_107" --json \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["topic"])')
echo "DM topic: $DM_TOPIC"

step "Step 2: alice (.107) sends first message via channel dm --send"
"$BIN" channel dm "$PEER_BOB" --send "alice: hi bob, can we sync?" --hub "$HUB_107" --json >/dev/null

step "Step 3: bob (.122 cross-hub TCP) replies on the SAME topic"
# Bob's vantage: he doesn't use channel dm (which would derive a different
# topic from his fingerprint); he posts directly to the canonical topic
# alice already created. This is the cross-machine pattern: one side
# computes the canonical topic, both sides post to it.
"$BIN" remote exec ring20-management "$REMOTE_SESSION" \
  "termlink channel post '$DM_TOPIC' --payload 'bob: yes, in 5 min' --sender-id '$PEER_ALICE' --hub $HUB_107_FROM_122 --json" \
  >/dev/null 2>&1 || fail "bob's cross-hub DM reply failed"

step "Step 4: alice replies threaded to bob's message"
# Find bob's offset
"$BIN" channel state "$DM_TOPIC" --hub "$HUB_107" --json > "$WORK/dm_state.json"
BOB_OFFSET=$(python3 -c '
import json
rows = json.load(open("'"$WORK/dm_state.json"'"))
rows = rows if isinstance(rows, list) else rows.get("rows", [])
hits = [r for r in rows if r.get("sender_id") == "'"$PEER_ALICE"'"]
print(hits[0]["offset"] if hits else -1)
')
[ "$BOB_OFFSET" != "-1" ] || fail "could not locate bob's message"
"$BIN" channel post "$DM_TOPIC" --payload "alice: 👍 see you" --reply-to "$BOB_OFFSET" --hub "$HUB_107" --json >/dev/null
echo "alice's threaded reply linked to bob@offset=$BOB_OFFSET"

step "Step 5: Verify state from .107 native"
"$BIN" channel state "$DM_TOPIC" --hub "$HUB_107"
"$BIN" channel state "$DM_TOPIC" --hub "$HUB_107" --json > "$WORK/state_107.json"
SENDERS_107=$(python3 -c '
import json
rows = json.load(open("'"$WORK/state_107.json"'"))
rows = rows if isinstance(rows, list) else rows.get("rows", [])
ids = sorted(set(r.get("sender_id") for r in rows if r.get("sender_id")))
print(",".join(ids))
print(len(rows))
')
SENDER_LINE=$(echo "$SENDERS_107" | head -1)
ROW_COUNT=$(echo "$SENDERS_107" | tail -1)
echo "senders: $SENDER_LINE  rows: $ROW_COUNT"
echo "$SENDER_LINE" | grep -q "$PEER_ALICE" || fail "bob's reply missing on .107 read"
[ "$ROW_COUNT" -ge "3" ] || fail "expected >=3 rows in DM, got $ROW_COUNT"

step "Step 6: Verify state from .122 cross-hub (must converge)"
"$BIN" remote exec ring20-management "$REMOTE_SESSION" \
  "termlink channel state '$DM_TOPIC' --hub $HUB_107_FROM_122 --json" \
  > "$WORK/state_122_raw.txt" 2>/dev/null || fail "cross-hub state read failed"

python3 - <<PY > "$WORK/state_122.json"
import sys
raw = open("$WORK/state_122_raw.txt").read()
def extract(s, openc, closec):
    depth=0; start=-1
    for i,c in enumerate(s):
        if c==openc:
            if depth==0: start=i
            depth+=1
        elif c==closec:
            depth-=1
            if depth==0 and start>=0:
                return s[start:i+1]
    return None
out = extract(raw,'[',']') or extract(raw,'{','}')
if not out: sys.exit("no JSON")
sys.stdout.write(out)
PY

SENDERS_122=$(python3 -c '
import json
rows = json.load(open("'"$WORK/state_122.json"'"))
rows = rows if isinstance(rows, list) else rows.get("rows", [])
ids = sorted(set(r.get("sender_id") for r in rows if r.get("sender_id")))
print(",".join(ids))
')
echo "senders (.122 cross-hub): $SENDERS_122"
[ "$SENDER_LINE" = "$SENDERS_122" ] || fail "DM senders diverge across hubs: 107=$SENDER_LINE  122=$SENDERS_122"
echo "OK: cross-hub DM read convergence"

step "Step 7: Verify channel dm --list on .107 includes this DM"
"$BIN" channel dm --list --hub "$HUB_107" --json > "$WORK/dm_list.json" 2>/dev/null
HAS_TOPIC=$(python3 -c '
import json
d = json.load(open("'"$WORK/dm_list.json"'"))
items = d if isinstance(d, list) else (d.get("dms") or d.get("topics") or [])
names = [it.get("topic") if isinstance(it, dict) else it for it in items]
print("yes" if "'"$DM_TOPIC"'" in names else "no")
')
echo "DM list includes our topic: $HAS_TOPIC"
[ "$HAS_TOPIC" = "yes" ] || fail "channel dm --list missing $DM_TOPIC"

step "Step 8: Verify thread relationship — alice's last is a reply to bob's"
LAST_OFF=$(python3 -c '
import json
rows = json.load(open("'"$WORK/state_107.json"'"))
rows = rows if isinstance(rows, list) else rows.get("rows", [])
print(rows[-1].get("offset", -1))
')
"$BIN" channel quote "$DM_TOPIC" "$LAST_OFF" --hub "$HUB_107" --json > "$WORK/dm_quote.json"
PARENT_SENDER=$(python3 -c '
import json
d = json.load(open("'"$WORK/dm_quote.json"'"))
p = d.get("parent")
print(p.get("sender_id","none") if p else "no-parent")
')
echo "last message parent sender: $PARENT_SENDER (should be $PEER_ALICE)"
[ "$PARENT_SENDER" = "$PEER_ALICE" ] || fail "thread linkage broken"

echo
echo "DM-FLOW E2E PASSED"
echo "  DM topic: $DM_TOPIC"
echo "  Rows:     $ROW_COUNT (3 messages: alice → bob → alice-thread-reply)"
echo "  Cross-hub read converged: yes"
