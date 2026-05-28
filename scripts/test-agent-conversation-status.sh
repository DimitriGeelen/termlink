#!/usr/bin/env bash
# T-1826 — tests for agent-conversation-status.sh.
#
# Covers:
#   T1 usage error on missing args → exit 2
#   T2 unknown args → exit 2
#   T3 --json on empty cid yields ok=true, turn_count=0
#   T4 populated cid (deterministic ephemeral topic) yields correct counts
#   T5 pending detection: a turn with no receipt up_to >= its offset is pending
#
# T4/T5 use an ephemeral local topic to keep the test self-contained — no
# dependency on cross-host state. The local hub is assumed up (any agent
# session that ran termlink before will have one); pre-flight skips T4/T5
# cleanly if not.
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
SCRIPT="${SCRIPT:-scripts/agent-conversation-status.sh}"

# Counters
PASS=0
FAIL=0
SKIP=0

pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

# -------- T1: usage error on missing --topic --------
echo "T1: missing --topic → exit 2"
if "$SCRIPT" --conversation-id cid-x >/dev/null 2>&1; then
    fail "T1: should have failed without --topic"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T1: exit=$rc"
    else fail "T1: expected 2, got $rc"; fi
fi

# -------- T2: unknown arg → exit 2 --------
echo "T2: unknown arg --bogus → exit 2"
if "$SCRIPT" --bogus >/dev/null 2>&1; then
    fail "T2: should have failed on --bogus"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T2: exit=$rc"
    else fail "T2: expected 2, got $rc"; fi
fi

# -------- T3: empty / unknown cid → ok=true, turn_count=0 --------
echo "T3: unknown cid in chat-arc → ok=true with turn_count=0"
# Use a high-entropy cid that won't match anything.
nonce="cid-t1826-test-$$-$(date +%s)"
out="$("$SCRIPT" --topic agent-chat-arc --conversation-id "$nonce" --json 2>/dev/null || true)"
if [ -z "$out" ]; then
    skip "T3: no output (local hub may be down)"
else
    ok="$(printf '%s' "$out" | jq -r '.ok // false' 2>/dev/null || echo "")"
    tc="$(printf '%s' "$out" | jq -r '.summary.turn_count // -1' 2>/dev/null || echo "-1")"
    if [ "$ok" = "true" ] && [ "$tc" = "0" ]; then
        pass "T3: ok=true turn_count=0"
    else
        fail "T3: ok=$ok turn_count=$tc out=$out"
    fi
fi

# -------- T4/T5: populated cid + pending detection --------
# Deterministic scenario: create an ephemeral topic, post 2 turns with the
# same cid, post 1 receipt with up_to = first turn's offset. Expect:
#   turn_count=2, receipt_count=1, pending_count=1 (the second turn).
echo "T4/T5: populated cid + pending detection (ephemeral topic)"
test_topic="agent-conv-status-test-$$-$(date +%s)"
test_cid="cid-t1826-pending-$$"

# Create topic + post turns + receipt; if any step errors, skip the populated tests.
if ! "$TERMLINK" channel create "$test_topic" --retention messages:50 >/dev/null 2>&1; then
    skip "T4/T5: cannot create ephemeral topic (local hub probably down)"
else
    # Post turn #1. channel post --json emits {"delivered":{"offset":N,...},...}.
    post1="$("$TERMLINK" channel post "$test_topic" --msg-type turn --payload "turn-1" \
        --metadata conversation_id="$test_cid" --json 2>&1)" || post1=""
    offset1="$(printf '%s' "$post1" | jq -r '.delivered.offset // empty' 2>/dev/null || echo "")"

    # Post turn #2.
    post2="$("$TERMLINK" channel post "$test_topic" --msg-type turn --payload "turn-2" \
        --metadata conversation_id="$test_cid" --json 2>&1)" || post2=""
    offset2="$(printf '%s' "$post2" | jq -r '.delivered.offset // empty' 2>/dev/null || echo "")"

    # Post receipt acking up to offset1 only.
    "$TERMLINK" channel post "$test_topic" --msg-type receipt --payload "ack" \
        --metadata conversation_id="$test_cid" \
        --metadata up_to="$offset1" --json >/dev/null 2>&1 || true

    out="$("$SCRIPT" --topic "$test_topic" --conversation-id "$test_cid" --json 2>/dev/null || true)"

    if [ -z "$out" ] || [ -z "$offset1" ] || [ -z "$offset2" ]; then
        skip "T4/T5: setup incomplete (offset1=$offset1 offset2=$offset2)"
    else
        tc="$(printf '%s' "$out" | jq -r '.summary.turn_count')"
        rc="$(printf '%s' "$out" | jq -r '.summary.receipt_count')"
        pc="$(printf '%s' "$out" | jq -r '.summary.pending_count')"
        pending="$(printf '%s' "$out" | jq -r '.pending_turn_offsets | join(",")')"

        # T4 — counts
        if [ "$tc" = "2" ] && [ "$rc" = "1" ]; then
            pass "T4: turn_count=2 receipt_count=1"
        else
            fail "T4: expected turn=2 receipt=1, got turn=$tc receipt=$rc; out=$out"
        fi

        # T5 — pending detection: offset2 should be pending (offset1 acked).
        if [ "$pc" = "1" ] && [ "$pending" = "$offset2" ]; then
            pass "T5: pending offset=$offset2"
        else
            fail "T5: expected pending=[$offset2], got pending=[$pending] count=$pc"
        fi
    fi

    # No cleanup: termlink has no `channel delete` verb. The test topic
    # uses $$/timestamp suffix so it's uniquely-named per run; the hub's
    # retention setting (messages:50) bounds growth.
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
