#!/usr/bin/env bash
# T-2270: verify agent-listeners.sh surfaces identity_fingerprint from the
# heartbeat envelope's top-level sender_id (T-1427 verified fingerprint).
#
# Hub-independent: uses the TERMLINK_LISTENERS_TEST_JSON seam to feed a canned
# `channel subscribe --json` stream, so no running hub is required. Mirror of
# the T-2058 TERMLINK_GROWTH_TEST_JSON pattern (PL-213).
#
# Exit 0 = projection surfaces identity_fingerprint == sender_id; 1 = mismatch.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/agent-listeners.sh"

FP="deadbeefcafef00d1122334455667788990011223344556677889900aabbccddee"
NOW_MS="$(date +%s%3N)"

fixture="$(mktemp)"
trap 'rm -f "$fixture"' EXIT

# One canned heartbeat envelope, exactly the shape `channel subscribe --json`
# emits: sender_id at the top level (sibling of msg_type/ts), agent metadata
# under .metadata. ts is "now" so the row classifies LIVE.
cat > "$fixture" <<EOF
{"msg_type":"heartbeat","sender_id":"$FP","ts":$NOW_MS,"metadata":{"agent_id":"test-peer","role":"claude-code","interval_secs":"30","listen_topics":"agent-presence","host":"testhost","pty_session":"sess-1"}}
EOF

out="$(TERMLINK_LISTENERS_TEST_JSON="$fixture" bash "$SCRIPT" --json 2>/dev/null)"

got_fp="$(printf '%s' "$out" | jq -r '.listeners[] | select(.agent_id=="test-peer") | .identity_fingerprint')"

if [ "$got_fp" = "$FP" ]; then
    echo "PASS: identity_fingerprint surfaced from sender_id ($got_fp)"
    exit 0
else
    echo "FAIL: expected identity_fingerprint=$FP, got '$got_fp'"
    echo "--- listeners output ---"
    printf '%s\n' "$out"
    exit 1
fi
