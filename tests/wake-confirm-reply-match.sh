#!/usr/bin/env bash
# tests/wake-confirm-reply-match.sh (T-2412) — hermetic test for the broadened
# doorbell consumption-confirmation matcher. Feeds canned `channel subscribe`
# JSON via the TERMLINK_WAKECONFIRM_TEST_JSON seam (PL-213 — no live hub) and
# asserts CONSUMED (exit 0) vs NOT-CONSUMED (exit 3) across the matrix:
#   - a cid-matched REPLY (in_reply_to == since_offset) → CONSUMED  [T-2412 new]
#   - a receipt (up_to >= since_offset)                 → CONSUMED  [unchanged]
#   - our own original post (no in_reply_to)            → NOT self-matched
#   - a stale receipt (up_to < since_offset)            → NOT CONSUMED (T-1808)
#   - unrelated traffic                                 → NOT CONSUMED

set -u
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SELF_DIR/.."
WC="$ROOT/scripts/wake-confirm.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
fails=0
pass() { echo "  PASS: $1"; }
fail() { echo "  FAIL: $1"; fails=$((fails+1)); }

# check <expected: consumed|not> <since_offset> <json> <label>
check() {
    local expect="$1" so="$2" json="$3" label="$4" f rc
    f="$TMP/fixture.json"; printf '%s' "$json" > "$f"
    TERMLINK_WAKECONFIRM_TEST_JSON="$f" bash "$WC" \
        --topic dm:test --cid cid-x --since-offset "$so" --json >/dev/null 2>&1
    rc=$?
    if [ "$expect" = "consumed" ]; then
        [ "$rc" -eq 0 ] && pass "$label -> CONSUMED" || fail "$label expected CONSUMED (exit0) got exit=$rc"
    else
        [ "$rc" -eq 3 ] && pass "$label -> NOT CONSUMED" || fail "$label expected NOT-CONSUMED (exit3) got exit=$rc"
    fi
}

# (b) T-2412: a note reply referencing our posted offset (2) — the concierge case
check consumed 2 '[{"msg_type":"note","offset":3,"metadata":{"conversation_id":"cid-x","in_reply_to":"2"}}]' \
    "note reply in_reply_to=2 (concierge case)"
# chat reply also counts
check consumed 2 '[{"msg_type":"chat","offset":4,"metadata":{"in_reply_to":2}}]' \
    "chat reply in_reply_to=2 (numeric)"
# (a) canonical receipt still works — unchanged path
check consumed 2 '[{"msg_type":"receipt","offset":3,"metadata":{"up_to":"2"}}]' \
    "receipt up_to=2 (unchanged path)"
check consumed 2 '[{"msg_type":"receipt","offset":9,"metadata":{"up_to":"5"}}]' \
    "receipt up_to=5 >= so (unchanged path)"
# our OWN original post carries no in_reply_to → must NOT self-match
check not 2 '[{"msg_type":"note","offset":2,"metadata":{"conversation_id":"cid-x"}}]' \
    "own original post (no in_reply_to) — no self-match"
check not 2 '[{"msg_type":"turn","offset":2,"metadata":{"conversation_id":"cid-x"}}]' \
    "own turn (no in_reply_to) — no self-match"
# stale receipt (up_to < since_offset) — T-1808 guard preserved
check not 2 '[{"msg_type":"receipt","offset":1,"metadata":{"up_to":"1"}}]' \
    "stale receipt up_to=1 < so=2 (T-1808 guard)"
# a reply to a DIFFERENT turn (in_reply_to != so) — unrelated
check not 2 '[{"msg_type":"note","offset":5,"metadata":{"in_reply_to":"99"}}]' \
    "reply to different turn (in_reply_to=99) — unrelated"
# empty topic traffic
check not 2 '[]' "no traffic — NOT CONSUMED"

# --- both edited scripts parse cleanly ---
bash -n "$ROOT/scripts/wake-confirm.sh" 2>/dev/null \
    && pass "bash -n scripts/wake-confirm.sh clean" \
    || fail "bash -n scripts/wake-confirm.sh FAILED"
bash -n "$ROOT/scripts/agent-send.sh" 2>/dev/null \
    && pass "bash -n scripts/agent-send.sh clean" \
    || fail "bash -n scripts/agent-send.sh FAILED"
# grace poll wired into agent-send
grep -q 'AGENT_SEND_GRACE_SECS' "$ROOT/scripts/agent-send.sh" \
    && pass "agent-send has post-ring grace poll" \
    || fail "agent-send missing grace poll"

echo ""
if [ "$fails" -eq 0 ]; then echo "wake-confirm-reply-match: ALL PASS"; exit 0
else echo "wake-confirm-reply-match: $fails FAIL"; exit 1; fi
