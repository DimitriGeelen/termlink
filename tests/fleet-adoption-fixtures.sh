#!/usr/bin/env bash
# T-2591 — load-bearing fixture for scripts/fleet-adoption-snapshot.sh
# unique_speakers.
#
# Drives the whole script through a mock `termlink` (TERMLINK_BIN) + a mock
# listeners verb (LISTENERS_VERB) + a fixture hubs.toml, and asserts that the
# unique_speakers metric counts note + chat + post posters (the content set)
# while excluding meta envelopes (reaction/etc.). FAILS against the old
# `select(.msg_type == "chat")` whitelist (temp-revert proven), PASSES against
# the fix. No live hub, no network.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/../scripts/fleet-adoption-snapshot.sh"
[ -f "$SCRIPT" ] || { echo "FAIL: $SCRIPT not found"; exit 1; }

WORK="$(mktemp -d -t fleet-adoption-fixtures.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

# --- fixture hubs.toml: one hub -----------------------------------------------
cat > "$WORK/hubs.toml" <<'TOML'
[hubs.testhub]
address = "127.0.0.1:9999"
TOML

# --- mock listeners verb: keep verdict "ok" -----------------------------------
LV="$WORK/listeners.sh"
cat > "$LV" <<'LVEOF'
#!/usr/bin/env bash
printf '%s\n' '{"live":1}'
LVEOF
chmod +x "$LV"

# --- mock termlink ------------------------------------------------------------
# agent-chat-arc window has FOUR content posters across three content types
# plus one meta envelope:
#   note by agent-alpha   (agent_post path — the one the old whitelist DROPPED)
#   chat by agent-beta    (/broadcast-chat path)
#   post by agent-gamma   (legacy content type — still in the content set)
#   reaction by agent-delta  (META — must NOT count as a speaker)
# Fixed content set {post,chat,note} → unique_speakers = 3 (alpha,beta,gamma).
# Old "chat"-only whitelist            → unique_speakers = 1 (beta).
MOCK="$WORK/termlink"
cat > "$MOCK" <<'MOCKEOF'
#!/usr/bin/env bash
args="$*"
case "$args" in
  "channel info "*"agent-chat-arc"*)
    printf '%s\n' '{"count":4}'
    ;;
  "channel subscribe "*"agent-chat-arc"*)
    printf '%s\n' '{"offset":0,"msg_type":"note","sender_id":"fp-a","metadata":{"agent_id":"agent-alpha"}}'
    printf '%s\n' '{"offset":1,"msg_type":"chat","sender_id":"fp-b","metadata":{"agent_id":"agent-beta"}}'
    printf '%s\n' '{"offset":2,"msg_type":"post","sender_id":"fp-c","metadata":{"agent_id":"agent-gamma"}}'
    printf '%s\n' '{"offset":3,"msg_type":"reaction","sender_id":"fp-d","metadata":{"agent_id":"agent-delta"}}'
    ;;
  "channel list "*)
    printf '%s\n' '{"topics":[]}'
    ;;
  *)
    : ;;
esac
MOCKEOF
chmod +x "$MOCK"

fail=0
pass() { echo "PASS: $1"; }
bad()  { echo "FAIL: $1"; fail=1; }

out="$(TERMLINK_BIN="$MOCK" LISTENERS_VERB="$LV" \
      bash "$SCRIPT" --hubs-file "$WORK/hubs.toml" --json 2>/dev/null)"
speakers="$(printf '%s' "$out" | jq -r '.summary.unique_speakers // "MISSING"' 2>/dev/null)"

if [ "$speakers" = "3" ]; then
  pass "unique_speakers=3 — note(alpha)+chat(beta)+post(gamma) all counted, reaction(delta) excluded"
else
  bad "expected unique_speakers=3, got '$speakers' (old chat-only whitelist would give 1)"
  printf '  raw: %s\n' "$out"
fi

echo
if [ "$fail" -eq 0 ]; then
  echo "fleet-adoption-fixtures: ALL PASS"; exit 0
else
  echo "fleet-adoption-fixtures: FAILURES"; exit 1
fi
