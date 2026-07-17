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

# (b) T-2412/T-2413: the REPLY class — pinned explicitly per msg_type. Dropping any
# one of these must fail the suite rather than silently narrow the rail: T-2413 was
# exactly that failure (turn omitted, every fixture modelled on the one observed
# note-shaped concierge reply).
check consumed 2 '[{"msg_type":"note","offset":3,"metadata":{"conversation_id":"cid-x","in_reply_to":"2"}}]' \
    "note reply in_reply_to=2 (concierge case)"
# chat reply also counts
check consumed 2 '[{"msg_type":"chat","offset":4,"metadata":{"in_reply_to":2}}]' \
    "chat reply in_reply_to=2 (numeric)"
# T-2413: turn reply — the CANONICAL reply type (agent-send --await-reply's own
# definition). Omitted by T-2412 → false woken-but-silent on a real answered doorbell.
check consumed 2 '[{"msg_type":"turn","offset":3,"metadata":{"conversation_id":"cid-x","in_reply_to":"2"}}]' \
    "turn reply in_reply_to=2 (T-2413 canonical reply type)"
check consumed 2 '[{"msg_type":"turn","offset":5,"metadata":{"in_reply_to":2}}]' \
    "turn reply in_reply_to=2 (numeric)"
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
check not 2 '[{"msg_type":"turn","offset":5,"metadata":{"in_reply_to":"99"}}]' \
    "turn reply to different turn (in_reply_to=99) — unrelated"
# empty topic traffic
check not 2 '[]' "no traffic — NOT CONSUMED"

# --- T-2413: REAL-RAIL regression. The captured 2026-07-17 incident that fooled the
# T-2412 matcher: peer `aef` answered our turn (offset 1) at offset 2 with
# msg_type=turn + in_reply_to=1, signed with its own agent fp — and agent-send still
# reported "receiver never acked". Replaying the real envelope shape on every run
# means this exact false-silent can never be un-fixed.
FIXTURE="$ROOT/tests/fixtures/aef-turn-reply.json"
if [ -f "$FIXTURE" ]; then
    out="$(TERMLINK_WAKECONFIRM_TEST_JSON="$FIXTURE" bash "$WC" \
             --topic dm:real --cid cid-1784285903-5798 --since-offset 1 --json 2>/dev/null)"
    rc=$?
    if [ "$rc" -eq 0 ] && printf '%s' "$out" | jq -e '.consumed==true' >/dev/null 2>&1; then
        pass "real aef rail fixture (turn reply in_reply_to=1) -> CONSUMED"
    else
        fail "real aef rail fixture expected CONSUMED, got rc=$rc out=$out"
    fi
    # the reply must be reported as a reply, not mislabelled a receipt
    if printf '%s' "$out" | jq -e '.kind=="reply"' >/dev/null 2>&1; then
        pass "real aef rail fixture reports kind=reply"
    else
        fail "real aef rail fixture expected kind=reply, got $(printf '%s' "$out" | jq -c '.kind' 2>/dev/null)"
    fi
    # our own turn at offset 1 (host-key-signed, no in_reply_to) must NOT self-match
    # even though sender and recipient can share a key on a shared host
    TERMLINK_WAKECONFIRM_TEST_JSON="$FIXTURE" bash "$WC" \
        --topic dm:real --cid c --since-offset 99 --json >/dev/null 2>&1
    [ $? -eq 3 ] && pass "real aef rail fixture: no false match at unrelated offset" \
                 || fail "real aef rail fixture false-matched at unrelated offset 99"
else
    fail "missing real-rail fixture $FIXTURE"
fi

# kind reporting: a receipt must still report kind=receipt (not mislabelled reply)
out="$(printf '%s' '[{"msg_type":"receipt","offset":3,"metadata":{"up_to":"2"}}]' > "$TMP/r.json"; \
       TERMLINK_WAKECONFIRM_TEST_JSON="$TMP/r.json" bash "$WC" --topic t --cid c --since-offset 2 --json 2>/dev/null)"
printf '%s' "$out" | jq -e '.kind=="receipt"' >/dev/null 2>&1 \
    && pass "receipt reports kind=receipt" \
    || fail "receipt expected kind=receipt, got $(printf '%s' "$out" | jq -c '.kind' 2>/dev/null)"

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
