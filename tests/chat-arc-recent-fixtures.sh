#!/usr/bin/env bash
# T-2592 — load-bearing fixture for scripts/agent-chat-arc-recent.sh msg_type
# default. Drives the whole engine through a mock `termlink` (TERMLINK_BIN) on a
# single --hub (bypasses the hubs.toml walk) and asserts the three-state filter:
#
#   default (no flags)   → content set {post,chat,note}, meta excluded
#   --filter-msg-type X   → single-type narrowing
#   --all-msg-types       → everything, incl meta (reaction)
#
# The default-view test FAILS against the old `FILTER_MSG_TYPE="chat"` +
# `.msg_type == $mtype` default (temp-revert proven) and PASSES against the fix —
# making "a note post appears in the default 'what's been said?' view"
# load-bearing. No live hub, no network.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/../scripts/agent-chat-arc-recent.sh"
[ -f "$SCRIPT" ] || { echo "FAIL: $SCRIPT not found"; exit 1; }

WORK="$(mktemp -d -t chat-arc-recent-fixtures.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

# --- mock termlink ------------------------------------------------------------
# agent-chat-arc window has THREE content posters across three content types
# plus one meta envelope:
#   note by agent-alpha      (termlink_agent_post / agent_reply path — the type
#                             the OLD chat-only default silently DROPPED)
#   chat by agent-beta       (/broadcast-chat path)
#   post by agent-gamma      (legacy content type — still in the content set)
#   reaction by agent-delta  (META — must NOT appear except under --all-msg-types)
# ts is stamped at run time so every envelope is inside the 24h window.
MOCK="$WORK/termlink"
cat > "$MOCK" <<'MOCKEOF'
#!/usr/bin/env bash
args="$*"
now_ms="$(date +%s)000"
case "$args" in
  *"channel info"*"agent-chat-arc"*)
    printf '%s\n' '{"count":4}'
    ;;
  *"channel subscribe"*"agent-chat-arc"*)
    printf '{"offset":0,"ts":%s,"msg_type":"note","sender_id":"fp-a","metadata":{"agent_id":"agent-alpha"},"payload":"hello-note"}\n' "$now_ms"
    printf '{"offset":1,"ts":%s,"msg_type":"chat","sender_id":"fp-b","metadata":{"agent_id":"agent-beta"},"payload":"hello-chat"}\n' "$now_ms"
    printf '{"offset":2,"ts":%s,"msg_type":"post","sender_id":"fp-c","metadata":{"agent_id":"agent-gamma"},"payload":"hello-post"}\n' "$now_ms"
    printf '{"offset":3,"ts":%s,"msg_type":"reaction","sender_id":"fp-d","metadata":{"agent_id":"agent-delta"},"payload":"hello-reaction"}\n' "$now_ms"
    ;;
  *)
    : ;;
esac
MOCKEOF
chmod +x "$MOCK"

HUB="127.0.0.1:9999"
run() { TERMLINK_BIN="$MOCK" bash "$SCRIPT" --hub "$HUB" --json "$@" 2>/dev/null; }
types() { jq -r '.posts // [] | map(.msg_type) | sort | join(",")' 2>/dev/null; }

fail=0
pass() { echo "PASS: $1"; }
bad()  { echo "FAIL: $1"; fail=1; }

# ---- Test 1: default view = content set {post,chat,note}, meta excluded -------
out="$(run)"
got="$(printf '%s' "$out" | types)"
if [ "$got" = "chat,note,post" ]; then
  pass "default → content set {chat,note,post}, reaction excluded (note is VISIBLE)"
else
  bad "default → expected 'chat,note,post', got '$got' (old chat-only default gives just 'chat')"
  printf '  raw: %s\n' "$out"
fi

# ---- Test 2: the load-bearing assertion — a note post appears by default ------
if printf '%s' "$out" | jq -e '.posts // [] | any(.msg_type == "note")' >/dev/null 2>&1; then
  pass "default → a note post is present (the T-2592 silent-drop is closed)"
else
  bad "default → NO note post present (regression: note content hidden by default)"
fi

# ---- Test 3: --filter-msg-type note narrows to a single type ------------------
got="$(run --filter-msg-type note | types)"
if [ "$got" = "note" ]; then
  pass "--filter-msg-type note → only note (single-type narrowing preserved)"
else
  bad "--filter-msg-type note → expected 'note', got '$got'"
fi

# ---- Test 4: --all-msg-types includes meta (reaction) ------------------------
got="$(run --all-msg-types | types)"
if [ "$got" = "chat,note,post,reaction" ]; then
  pass "--all-msg-types → everything incl reaction (meta) still works"
else
  bad "--all-msg-types → expected 'chat,note,post,reaction', got '$got'"
fi

echo
if [ "$fail" -eq 0 ]; then
  echo "chat-arc-recent-fixtures: ALL PASS"; exit 0
else
  echo "chat-arc-recent-fixtures: FAILURES"; exit 1
fi
