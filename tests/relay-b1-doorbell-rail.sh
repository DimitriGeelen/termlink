#!/usr/bin/env bash
# T-2394 (relay-loop B1): verify agent-send.sh stamps the reply-rail into the
# doorbell text so a woken agent replies on the EXACT rail that rang it.
#
# The relay-loop return leg (T-2393 Gap 1) requires the woken recipient to reply
# on the same dm:<a>:<b> rail + conversation_id that woke them — otherwise the
# reply lands on a thread topic that rings nobody and the return leg never fires.
# B1 closes this by defaulting the doorbell text to
#   /check-arc respond --rail <topic> --cid <cid>
# whenever the caller did NOT override --doorbell-text. An explicit
# --doorbell-text must be preserved verbatim (operator override wins).
#
# Hub-independent: uses --dry-run + TERMLINK=/bin/true so no running hub is
# required (explicit --to-session/--topic short-circuits presence resolution).
# The RESOLVED line echoes doorbell_text=[...] (T-2394 seam) for assertion.
#
# Exit 0 = both branches correct; 1 = a mismatch.
set -euo pipefail

cd "$(dirname "$0")/.."

fail() { echo "FAIL: $*" >&2; exit 1; }

# --- Case 1: default doorbell text is rail-augmented -------------------------
out_default="$(TERMLINK=/bin/true bash scripts/agent-send.sh \
    --to-session sess-b --topic "dm:aaa:bbb" --conversation-id "cid-test" \
    --message "hello" --dry-run 2>&1 | grep RESOLVED)"

echo "$out_default" | grep -q -- "--rail dm:aaa:bbb" \
    || fail "default doorbell missing '--rail dm:aaa:bbb' (got: $out_default)"
echo "$out_default" | grep -q -- "--cid cid-test" \
    || fail "default doorbell missing '--cid cid-test' (got: $out_default)"

# --- Case 2: explicit --doorbell-text is preserved verbatim -------------------
out_custom="$(TERMLINK=/bin/true bash scripts/agent-send.sh \
    --to-session sess-b --topic "dm:aaa:bbb" --conversation-id "cid-test" \
    --message "hello" --doorbell-text "custom text" --dry-run 2>&1 | grep RESOLVED)"

echo "$out_custom" | grep -q "doorbell_text=\[custom text\]" \
    || fail "custom doorbell-text not preserved (got: $out_custom)"
echo "$out_custom" | grep -q -- "--rail" \
    && fail "custom doorbell-text was wrongly rail-augmented (got: $out_custom)"

echo "PASS: relay-b1 doorbell-rail — default stamps --rail/--cid; custom preserved"
