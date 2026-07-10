#!/usr/bin/env bash
# T-2390: verify agent-listeners.sh reads presence via the cv_index
# (`channel subscribe --include-current-value`) rather than the T-1844
# count-based seek-to-tail, which is blind to live heartbeats under
# `latest_per_cv_key` retention (count decoupled from the monotonic tail
# offset — see T-2390 RCA).
#
# Hub-independent: uses the TERMLINK_LISTENERS_CV_TEST_JSON seam to feed canned
# `channel subscribe --include-current-value` output, so no running hub is
# required. Mirror of the TERMLINK_LISTENERS_TEST_JSON / TERMLINK_GROWTH_TEST_JSON
# pattern (PL-213).
#
# The fixture deliberately reproduces the bug shape: 4 LIVE agents at HIGH
# offsets (33409-33412) alongside dead-agent cv_keys pinned at LOW offsets
# (30810/30816). The legacy count-based seek would clamp to the low offsets and
# miss the live tail; the cv path returns the current value per key regardless.
#
# Exit 0 = all 4 live agents parse LIVE from current_values; 1 = miss.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/agent-listeners.sh"

NOW_MS="$(date +%s%3N)"
OLD_MS=$((NOW_MS - 700000))  # ~11.7 min → OFFLINE (> 5*interval)

fixture="$(mktemp)"
trap 'rm -f "$fixture"' EXIT

# `channel subscribe --include-current-value --json` emits ONE object carrying a
# `current_values` array (each entry: {cv_key, msg, offset}) followed by any
# streamed envelopes. The script's jq filters to the current_values object and
# explodes .current_values[].msg. We append a trailing stream line to prove it
# is ignored (only the current_values object is authoritative).
mk_msg() { # agent_id offset ts
  printf '{"cv_key":"%s","offset":%s,"msg":{"msg_type":"heartbeat","sender_id":"fp-%s","ts":%s,"offset":%s,"metadata":{"agent_id":"%s","role":"claude-code","interval_secs":"30","listen_topics":"dm:%s:*,agent-chat-arc","host":"testhost","pty_session":"%s"}}}' \
    "$1" "$2" "$1" "$3" "$2" "$1" "$1" "$1"
}

{
  printf '{"current_values":['
  mk_msg aef              33409 "$NOW_MS"; printf ','
  mk_msg sonnenstall      33410 "$NOW_MS"; printf ','
  mk_msg workshop-designer 33411 "$NOW_MS"; printf ','
  mk_msg workflow-designer 33412 "$NOW_MS"; printf ','
  mk_msg arc004-probe     30810 "$OLD_MS"; printf ','
  mk_msg demo-c7          30816 "$OLD_MS"
  printf ']}\n'
  # Trailing stream line (must be ignored by the current_values filter):
  printf '{"msg_type":"heartbeat","sender_id":"fp-stream","ts":%s,"offset":30797,"metadata":{"agent_id":"stream-noise","role":"listener","interval_secs":"30"}}\n' "$OLD_MS"
} > "$fixture"

out="$(TERMLINK_LISTENERS_CV_TEST_JSON="$fixture" bash "$SCRIPT" --no-cache --json 2>/dev/null)"

live="$(printf '%s' "$out" | jq -r '.live')"
live_ids="$(printf '%s' "$out" | jq -r '[.listeners[] | select(.status=="LIVE") | .agent_id] | sort | join(",")')"
expected="aef,sonnenstall,workflow-designer,workshop-designer"

if [ "$live" = "4" ] && [ "$live_ids" = "$expected" ]; then
    echo "PASS: cv_index read surfaced 4 LIVE agents ($live_ids) despite low-offset dead keys"
    exit 0
else
    echo "FAIL: expected live=4 [$expected], got live=$live [$live_ids]"
    echo "--- listeners output ---"
    printf '%s\n' "$out"
    exit 1
fi
