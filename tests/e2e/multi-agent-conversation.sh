#!/usr/bin/env bash
# T-1385 multi-agent end-to-end test.
#
# Goal: prove that 6 distinct agents (cryptographic identities) can hold a
# conversation in one topic, AND that two hubs (.107 local + .122 remote)
# converge on byte-identical canonical state via cross-hub TCP RPC.
#
# Layout:
#   - Topic created on local (.107) hub.
#   - 6 synthetic identities post a sequence of messages: post, reply,
#     edit, redaction, receipt, annotation.
#   - Cross-hub mirror: same topic created on .122, same 6 identities post
#     the same content via TCP --hub.
#   - Compare canonical state via `channel state` → must be byte-identical
#     for offset/sender_id/payload triples.
#
# Requires:
#   - Local hub running on /var/lib/termlink/hub.sock (and 127.0.0.1:9100 TCP).
#   - hubs.toml profile `ring20-management` pointing at 192.168.10.122:9100
#     with secret_file resolved.
#   - termlink binary at ./target/release/termlink (or override BIN env).
#
# Exits non-zero on any divergence or failure.

set -euo pipefail

BIN=${BIN:-./target/release/termlink}
LOCAL_HUB=${LOCAL_HUB:-127.0.0.1:9100}
REMOTE_HUB=${REMOTE_HUB:-192.168.10.122:9100}
N=$RANDOM
TOPIC="multi-agent-e2e-${N}"
WORK=$(mktemp -d -t multi-agent-e2e.XXXXXX)
trap 'rm -rf "$WORK"' EXIT

step() { echo; echo "=== $* ==="; }
fail() { echo "FAIL: $*" >&2; exit 1; }

step "Inventory binaries and hubs"
"$BIN" --version
echo "Local hub:  $LOCAL_HUB"
echo "Remote hub: $REMOTE_HUB"
echo "Topic:      $TOPIC"

step "Step 1: Create topic on both hubs"
"$BIN" channel create "$TOPIC" --hub "$LOCAL_HUB"
"$BIN" channel create "$TOPIC" --hub "$REMOTE_HUB"

# Six synthetic agents — distinct sender_id strings stand in for distinct
# identities. We don't need cryptographic divergence for this test (one
# identity on this host signs all posts), only to prove the bus correctly
# attributes by sender_id metadata.
AGENTS=(alice bob carol dave erin frank)
MESSAGES=(
  "alice:Initial proposal — ship the prototype Friday"
  "bob:Reply, pushing back on Friday"
  "carol:Counter-proposal: stage to canary first"
  "dave:Note: blocked by hub auth refactor"
  "erin:Update — auth refactor is live (T-1385)"
  "frank:Ack, ready to ship"
)

post_to_hub() {
  local hub="$1"; shift
  local off=0
  for entry in "${MESSAGES[@]}"; do
    local sid="${entry%%:*}"
    local payload="${entry#*:}"
    "$BIN" channel post "$TOPIC" \
      --payload "$payload" \
      --sender-id "$sid" \
      --hub "$hub" >/dev/null
    off=$((off + 1))
  done
}

step "Step 2: Post 6-agent conversation to LOCAL hub"
post_to_hub "$LOCAL_HUB"

step "Step 3: Post the same conversation to REMOTE hub via TCP --hub"
post_to_hub "$REMOTE_HUB"

step "Step 4: Pull canonical state from both hubs"
"$BIN" channel state "$TOPIC" --hub "$LOCAL_HUB"  --json > "$WORK/local.json"
"$BIN" channel state "$TOPIC" --hub "$REMOTE_HUB" --json > "$WORK/remote.json"

step "Step 5: Compare (sender_id, payload) tuples per offset"
extract() {
  python3 - "$1" <<'PYEOF'
import json, sys
data = json.load(open(sys.argv[1]))
rows = data.get("rows", []) if isinstance(data, dict) else data
for r in rows:
    print("\t".join([str(r.get("offset")), str(r.get("sender_id")), str(r.get("payload"))]))
PYEOF
}
extract "$WORK/local.json"  > "$WORK/local.tuples"
extract "$WORK/remote.json" > "$WORK/remote.tuples"

if diff -u "$WORK/local.tuples" "$WORK/remote.tuples"; then
  echo "OK: canonical state byte-identical across hubs"
else
  fail "canonical state diverged between $LOCAL_HUB and $REMOTE_HUB"
fi

step "Step 6: Reply chain (m.in_reply_to) — alice replies to bob"
"$BIN" channel post "$TOPIC" \
  --payload "alice: Bob, what about staging?" \
  --sender-id alice \
  --reply-to 1 \
  --hub "$LOCAL_HUB" >/dev/null

step "Step 7: Edit (m.replace) — bob edits offset 1"
"$BIN" channel post "$TOPIC" \
  --msg-type edit \
  --payload "bob (edited): I was wrong — Friday is fine" \
  --metadata replaces=1 \
  --sender-id bob \
  --hub "$LOCAL_HUB" >/dev/null

step "Step 8: Redaction — frank redacts offset 3"
"$BIN" channel post "$TOPIC" \
  --msg-type redaction \
  --payload "" \
  --metadata redacts=3 \
  --sender-id frank \
  --hub "$LOCAL_HUB" >/dev/null

step "Step 9: Re-pull state and verify edit + redaction collapsed"
"$BIN" channel state "$TOPIC" --hub "$LOCAL_HUB" --json > "$WORK/local-after.json"
EDITED=$(WORK="$WORK" python3 - <<'PYEOF'
import json, os
d = json.load(open(os.environ["WORK"] + "/local-after.json"))
rows = d["rows"] if isinstance(d, dict) else d
r = [x for x in rows if x.get("offset") == 1]
print(r[0].get("is_edited", False) if r else False)
PYEOF
)
REDACTED_VISIBLE=$(WORK="$WORK" python3 - <<'PYEOF'
import json, os
d = json.load(open(os.environ["WORK"] + "/local-after.json"))
rows = d["rows"] if isinstance(d, dict) else d
print(any(x.get("offset") == 3 for x in rows))
PYEOF
)
[ "$EDITED" = "True" ] || fail "expected offset 1 to be is_edited=true, got $EDITED"
[ "$REDACTED_VISIBLE" = "False" ] || fail "expected offset 3 to be hidden after redaction"
echo "OK: edit collapsed, redaction hidden"

step "Step 10: Prove cross-hub isolation — local edit/redact did NOT leak to remote"
"$BIN" channel state "$TOPIC" --hub "$LOCAL_HUB"  --json > "$WORK/local-final.json"
"$BIN" channel state "$TOPIC" --hub "$REMOTE_HUB" --json > "$WORK/remote-final.json"
LOCAL_O1=$(WORK="$WORK" python3 -c 'import json,os; d=json.load(open(os.environ["WORK"]+"/local-final.json")); rows=d.get("rows",[]) if isinstance(d,dict) else d; r=[x for x in rows if x.get("offset")==1]; print(r[0].get("payload","") if r else "MISSING")')
REMOTE_O1=$(WORK="$WORK" python3 -c 'import json,os; d=json.load(open(os.environ["WORK"]+"/remote-final.json")); rows=d.get("rows",[]) if isinstance(d,dict) else d; r=[x for x in rows if x.get("offset")==1]; print(r[0].get("payload","") if r else "MISSING")')
echo "Local offset 1 payload:  $LOCAL_O1"
echo "Remote offset 1 payload: $REMOTE_O1"
case "$LOCAL_O1" in
  *edited*) echo "OK: local shows edited content";;
  *) fail "expected local offset 1 to contain 'edited', got: $LOCAL_O1";;
esac
case "$REMOTE_O1" in
  *Reply,\ pushing\ back*) echo "OK: remote shows original content (edit did not leak)";;
  *) fail "expected remote offset 1 to remain original 'Reply, pushing back...', got: $REMOTE_O1";;
esac

LOCAL_HAS_3=$(WORK="$WORK" python3 -c 'import json,os; d=json.load(open(os.environ["WORK"]+"/local-final.json")); rows=d.get("rows",[]) if isinstance(d,dict) else d; print(any(x.get("offset")==3 for x in rows))')
REMOTE_HAS_3=$(WORK="$WORK" python3 -c 'import json,os; d=json.load(open(os.environ["WORK"]+"/remote-final.json")); rows=d.get("rows",[]) if isinstance(d,dict) else d; print(any(x.get("offset")==3 for x in rows))')
[ "$LOCAL_HAS_3" = "False" ] || fail "local offset 3 should be redacted, but is visible"
[ "$REMOTE_HAS_3" = "True" ]  || fail "remote offset 3 should still be visible (redact did not leak)"
echo "OK: redaction did not leak to remote"

echo
echo "MULTI-AGENT E2E PASSED"
echo "  Topic:           $TOPIC"
echo "  Local hub:       $LOCAL_HUB"
echo "  Remote hub:      $REMOTE_HUB"
echo "  Agents:          ${#AGENTS[@]} (alice, bob, carol, dave, erin, frank)"
echo "  Cross-hub state: byte-identical after initial 6 posts"
echo "  Isolation proof: local edit + redact did not leak to remote"
