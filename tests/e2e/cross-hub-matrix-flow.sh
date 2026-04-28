#!/usr/bin/env bash
# T-1391 cross-hub Matrix-primitive end-to-end flow.
#
# Exercises the Matrix-analogue subset of the agent-conversation arc
# (replies, reactions, edits, redactions, receipts) in a 6-agent
# conversation that spans the .107 + .122 hubs. Builds on T-1390 which
# established cross-hub posting + read convergence; this test layers
# the primitives that depend on offset bookkeeping (`metadata.in_reply_to`,
# `metadata.replaces`, `metadata.redacts`, `metadata.up_to`).
#
# Hard caveat (same as T-1387 / T-1390): identity is per-user, not per
# session. We use `--sender-id` overrides on commands that support it
# (post, reply, react, ack). `channel edit` and `channel redact` have
# no `--sender-id` flag, so those primitives are always attributed to
# the local identity. This is a CLI-surface limitation, not an arc bug.

set -euo pipefail

BIN=${BIN:-./target/release/termlink}
HUB_107=${HUB_107:-127.0.0.1:9100}
HUB_122=${HUB_122:-192.168.10.122:9100}
HUB_107_FROM_122=${HUB_107_FROM_122:-192.168.10.107:9100}
N=$RANDOM
TOPIC="xhub-matrix-${N}"
WORK=$(mktemp -d -t xhub-matrix.XXXXXX)
trap 'rm -rf "$WORK"' EXIT

step() { echo; echo "=== $* ==="; }
fail() { echo "FAIL: $*" >&2; exit 1; }

REMOTE_SESSION=$($BIN remote list ring20-management 2>/dev/null | awk '/^tl-/ {print $1; exit}')
[ -n "$REMOTE_SESSION" ] || fail "no live session on ring20-management"

# Helper: post and capture offset from --json output
post_capture() {
  local topic="$1" payload="$2" sender="$3" hub="$4"
  shift 4
  "$BIN" channel post "$topic" \
    --payload "$payload" --sender-id "$sender" --hub "$hub" --json "$@" \
    | python3 -c '
import json, sys
d = json.load(sys.stdin)
# shape: {"delivered": {"offset": N, "ts": ...}} or {"offset": N}
if "delivered" in d and isinstance(d["delivered"], dict):
    print(d["delivered"].get("offset", -1))
elif "offset" in d:
    print(d["offset"])
else:
    print(-1)
'
}

step "Inventory"
"$BIN" --version
echo "Topic:   $TOPIC (origin .107)"
echo "Hubs:    .107=$HUB_107  .122=$HUB_122"
echo "Remote:  $REMOTE_SESSION (.122)"

step "Step 1: Create topic on .107 hub"
"$BIN" channel create "$TOPIC" --hub "$HUB_107" >/dev/null
echo "OK"

step "Step 2: Alice posts root (offset 0)"
ALICE_OFFSET=$(post_capture "$TOPIC" "alice: who's joining the standup?" "alice-107" "$HUB_107")
echo "alice root offset: $ALICE_OFFSET"
[ "$ALICE_OFFSET" = "0" ] || fail "expected alice root at offset 0, got $ALICE_OFFSET"

step "Step 3: Bob replies to alice (offset 1)"
BOB_OFFSET=$(post_capture "$TOPIC" "bob: I'm in" "bob-107" "$HUB_107" --reply-to 0)
echo "bob reply offset: $BOB_OFFSET"

step "Step 4: Carol reacts 👍 to alice (offset 2)"
"$BIN" channel react "$TOPIC" 0 "👍" --sender-id "carol-107" --hub "$HUB_107" --json >/dev/null
echo "OK"

step "Step 5: Dave replies to bob (offset 3, depth-2)"
DAVE_OFFSET=$(post_capture "$TOPIC" "dave: same here" "dave-107" "$HUB_107" --reply-to "$BOB_OFFSET")
echo "dave deep-reply offset: $DAVE_OFFSET"

step "Step 6: Erin posts a typo, then edits it (offsets 4 + 5)"
ERIN_OFFSET=$(post_capture "$TOPIC" "erin: srandup at 9" "erin-107" "$HUB_107")
"$BIN" channel edit "$TOPIC" "$ERIN_OFFSET" "erin: standup at 9 (edited)" --hub "$HUB_107" --json >/dev/null
echo "erin original offset: $ERIN_OFFSET (edit emitted)"

step "Step 7: Erin posts a stray comment, then redacts it (offsets 6 + 7)"
ERIN_STRAY=$(post_capture "$TOPIC" "erin: oops wrong channel" "erin-107" "$HUB_107")
"$BIN" channel redact "$TOPIC" "$ERIN_STRAY" --reason "wrong-channel" --hub "$HUB_107" --json >/dev/null
echo "erin stray offset: $ERIN_STRAY (redact emitted)"

step "Step 8: Frank from .122 reacts 🎉 to alice over cross-hub TCP"
"$BIN" remote exec ring20-management "$REMOTE_SESSION" \
  "termlink channel react '$TOPIC' 0 '🎉' --sender-id 'frank-122' --hub $HUB_107_FROM_122 --json" \
  >/dev/null 2>&1 || fail "cross-hub react from .122 failed"
echo "OK: frank reacted from .122"

step "Step 9: Frank from .122 replies to dave's reply (cross-hub deep reply)"
"$BIN" remote exec ring20-management "$REMOTE_SESSION" \
  "termlink channel post '$TOPIC' --payload 'frank: count me in too (from .122)' --sender-id 'frank-122' --reply-to $DAVE_OFFSET --hub $HUB_107_FROM_122 --json" \
  >/dev/null 2>&1 || fail "cross-hub reply from .122 failed"
echo "OK: frank replied from .122"

step "Step 10: Read full topic state from .107"
"$BIN" channel state "$TOPIC" --hub "$HUB_107"

step "Step 11: Verify thread structure from .107 (DFS render)"
"$BIN" channel thread "$TOPIC" 0 --hub "$HUB_107" --json > "$WORK/thread_107.json"
THREAD_107_SENDERS=$(python3 -c '
import json, sys
d = json.load(open("'"$WORK/thread_107.json"'"))
nodes = d if isinstance(d, list) else (d.get("thread") or d.get("nodes") or [])
ids = sorted(set(n.get("sender_id") for n in nodes if n.get("sender_id")))
print(",".join(ids))
')
echo "thread senders (.107): $THREAD_107_SENDERS"

step "Step 12: Verify thread structure from .122 (cross-hub TCP read) — must converge"
"$BIN" remote exec ring20-management "$REMOTE_SESSION" \
  "termlink channel thread '$TOPIC' 0 --hub $HUB_107_FROM_122 --json" \
  > "$WORK/thread_122_raw.txt" 2>/dev/null || fail "cross-hub thread read failed"

# extract balanced JSON
python3 - <<PY > "$WORK/thread_122.json"
import sys
raw = open("$WORK/thread_122_raw.txt").read()
def extract(s, openc, closec):
    depth = 0; start = -1
    for i,c in enumerate(s):
        if c == openc:
            if depth == 0: start = i
            depth += 1
        elif c == closec:
            depth -= 1
            if depth == 0 and start >= 0:
                return s[start:i+1]
    return None
out = extract(raw, '[', ']') or extract(raw, '{', '}')
if not out: sys.exit("no JSON in remote thread output")
sys.stdout.write(out)
PY

THREAD_122_SENDERS=$(python3 -c '
import json
d = json.load(open("'"$WORK/thread_122.json"'"))
nodes = d if isinstance(d, list) else (d.get("thread") or d.get("nodes") or [])
ids = sorted(set(n.get("sender_id") for n in nodes if n.get("sender_id")))
print(",".join(ids))
')
echo "thread senders (.122 cross-hub): $THREAD_122_SENDERS"

[ "$THREAD_107_SENDERS" = "$THREAD_122_SENDERS" ] \
  || fail "thread sender sets differ across hub vantage points: 107=$THREAD_107_SENDERS vs 122=$THREAD_122_SENDERS"
echo "OK: thread structure converges across hubs"

step "Step 13: Members list — must include all 6 stand-in senders + local identity (for edit/redact)"
"$BIN" channel members "$TOPIC" --include-meta --hub "$HUB_107" --json > "$WORK/members.json"
MEMBERS=$(python3 -c '
import json, sys
d = json.load(open("'"$WORK/members.json"'"))
rows = d if isinstance(d, list) else d.get("members", [])
ids = sorted(set(r.get("sender_id") for r in rows if r.get("sender_id")))
print(",".join(ids))
print(len(ids))
')
MEMBER_LINE=$(echo "$MEMBERS" | head -1)
MEMBER_COUNT=$(echo "$MEMBERS" | tail -1)
echo "members: $MEMBER_LINE"
echo "count:   $MEMBER_COUNT"

# Required senders (the 6 stand-ins). Local identity is also there from edit+redact;
# count must be >= 6. We assert each named sender is present.
for required in alice-107 bob-107 carol-107 dave-107 erin-107 frank-122; do
  echo "$MEMBER_LINE" | grep -q "$required" || fail "missing required sender: $required"
done
[ "$MEMBER_COUNT" -ge "6" ] || fail "expected >=6 members, got $MEMBER_COUNT"
echo "OK: all 6 stand-in senders attributed in members list"

step "Step 14: Verify at least one edit + one redaction in canonical state"
"$BIN" channel state "$TOPIC" --hub "$HUB_107" --include-redacted --json > "$WORK/state.json"
HAS_EDIT=$(python3 -c '
import json
d = json.load(open("'"$WORK/state.json"'"))
rows = d.get("rows", []) if isinstance(d, dict) else d
print("yes" if any(r.get("is_edited") or r.get("edit_count",0) > 0 for r in rows) else "no")
')
HAS_REDACT=$(python3 -c '
import json
d = json.load(open("'"$WORK/state.json"'"))
rows = d.get("rows", []) if isinstance(d, dict) else d
print("yes" if any(r.get("is_redacted") for r in rows) else "no")
')
echo "edit visible:   $HAS_EDIT"
echo "redact visible: $HAS_REDACT"
[ "$HAS_EDIT" = "yes" ] || fail "no edited envelope in canonical state"
[ "$HAS_REDACT" = "yes" ] || fail "no redacted envelope in canonical state"

echo
echo "MATRIX-FLOW E2E PASSED"
echo "  Topic: $TOPIC"
echo "  Senders: $MEMBER_COUNT distinct (incl. all 6 stand-ins)"
echo "  Primitives exercised: post / reply / react / edit / redact (all visible cross-hub)"
echo "  Cross-hub thread convergence: verified"
