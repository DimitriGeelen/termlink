#!/usr/bin/env bash
# T-1827 — tests for agent-conversation-list.sh.
#
# Covers:
#   T1 usage missing --topic → exit 2
#   T2 unknown arg → exit 2
#   T3 invalid --sort → exit 2
#   T4 empty topic → ok=true conversation_count=0
#   T5 populated topic with 2 distinct cids → conversation_count=2 + per-cid counts
#   T6 --include-no-cid adds a sentinel entry
#   T7 --sort turn_count orders descending by turn count
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
SCRIPT="${SCRIPT:-scripts/agent-conversation-list.sh}"

PASS=0
FAIL=0
SKIP=0

pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

# -------- T1: usage error on missing --topic --------
echo "T1: missing --topic → exit 2"
if "$SCRIPT" >/dev/null 2>&1; then
    fail "T1: should have failed without --topic"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T1: exit=$rc"
    else fail "T1: expected 2, got $rc"; fi
fi

# -------- T2: unknown arg → exit 2 --------
echo "T2: unknown arg → exit 2"
if "$SCRIPT" --bogus >/dev/null 2>&1; then
    fail "T2: should have failed on --bogus"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T2: exit=$rc"
    else fail "T2: expected 2, got $rc"; fi
fi

# -------- T3: invalid --sort → exit 2 --------
echo "T3: invalid --sort → exit 2"
if "$SCRIPT" --topic any --sort wat >/dev/null 2>&1; then
    fail "T3: should have rejected --sort wat"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T3: exit=$rc"
    else fail "T3: expected 2, got $rc"; fi
fi

# -------- T4: empty topic (exists, no envelopes) → ok=true, conversation_count=0 --------
echo "T4: empty topic (created but unpopulated) → ok=true conversation_count=0"
empty_topic="agent-conv-list-empty-$$-$(date +%s)"
if ! "$TERMLINK" channel create "$empty_topic" --retention messages:5 >/dev/null 2>&1; then
    skip "T4: cannot create ephemeral topic (local hub down?)"
else
    out="$("$SCRIPT" --topic "$empty_topic" --json 2>/dev/null || true)"
    ok="$(printf '%s' "$out" | jq -r '.ok // false' 2>/dev/null || echo "")"
    cc="$(printf '%s' "$out" | jq -r '.conversation_count // -1' 2>/dev/null || echo "-1")"
    if [ "$ok" = "true" ] && [ "$cc" = "0" ]; then
        pass "T4: ok=true conversation_count=0 (empty topic)"
    else
        fail "T4: ok=$ok cc=$cc out=$out"
    fi
fi

# -------- T5/T6/T7: populated topic with 2 distinct cids + 1 no-cid envelope --------
echo "T5/T6/T7: populated topic — counts, no-cid sentinel, sort by turn_count"
test_topic="agent-conv-list-test-$$-$(date +%s)"
if ! "$TERMLINK" channel create "$test_topic" --retention messages:50 >/dev/null 2>&1; then
    skip "T5/T6/T7: cannot create ephemeral topic (local hub down?)"
else
    # cid-A: 2 turns + 1 receipt
    "$TERMLINK" channel post "$test_topic" --msg-type turn --payload "A1" \
        --metadata conversation_id="cid-A" --json >/dev/null 2>&1 || true
    "$TERMLINK" channel post "$test_topic" --msg-type turn --payload "A2" \
        --metadata conversation_id="cid-A" --json >/dev/null 2>&1 || true
    "$TERMLINK" channel post "$test_topic" --msg-type receipt --payload "ack" \
        --metadata conversation_id="cid-A" --metadata up_to=0 --json >/dev/null 2>&1 || true
    # cid-B: 1 turn
    "$TERMLINK" channel post "$test_topic" --msg-type turn --payload "B1" \
        --metadata conversation_id="cid-B" --json >/dev/null 2>&1 || true
    # no-cid: 1 chat
    "$TERMLINK" channel post "$test_topic" --msg-type chat --payload "no-cid-msg" \
        --json >/dev/null 2>&1 || true

    # T5: default (no-cid skipped) → conversation_count=2
    out="$("$SCRIPT" --topic "$test_topic" --json 2>/dev/null || true)"
    cc="$(printf '%s' "$out" | jq -r '.conversation_count // -1')"
    if [ "$cc" = "2" ]; then
        # Find cid-A entry and verify turn_count=2, receipt_count=1
        a_turns="$(printf '%s' "$out" | jq -r '.conversations[] | select(.conversation_id == "cid-A") | .turn_count')"
        a_receipts="$(printf '%s' "$out" | jq -r '.conversations[] | select(.conversation_id == "cid-A") | .receipt_count')"
        b_turns="$(printf '%s' "$out" | jq -r '.conversations[] | select(.conversation_id == "cid-B") | .turn_count')"
        if [ "$a_turns" = "2" ] && [ "$a_receipts" = "1" ] && [ "$b_turns" = "1" ]; then
            pass "T5: conversation_count=2 cid-A(turns=2,rec=1) cid-B(turns=1)"
        else
            fail "T5: counts wrong (A_turns=$a_turns A_rec=$a_receipts B_turns=$b_turns)"
        fi
    else
        fail "T5: expected conversation_count=2, got $cc; out=$out"
    fi

    # T6: --include-no-cid → conversation_count=3 with "(no-cid)" sentinel
    out2="$("$SCRIPT" --topic "$test_topic" --include-no-cid --json 2>/dev/null || true)"
    cc2="$(printf '%s' "$out2" | jq -r '.conversation_count // -1')"
    has_nocid="$(printf '%s' "$out2" | jq -r '.conversations | map(.conversation_id) | contains(["(no-cid)"])')"
    if [ "$cc2" = "3" ] && [ "$has_nocid" = "true" ]; then
        pass "T6: --include-no-cid → conversation_count=3 with (no-cid) sentinel"
    else
        fail "T6: cc2=$cc2 has_nocid=$has_nocid"
    fi

    # T7: --sort turn_count → cid-A (2 turns) first
    first_cid="$("$SCRIPT" --topic "$test_topic" --sort turn_count --json 2>/dev/null | jq -r '.conversations[0].conversation_id')"
    if [ "$first_cid" = "cid-A" ]; then
        pass "T7: --sort turn_count → cid-A first (highest turn count)"
    else
        fail "T7: expected cid-A first, got '$first_cid'"
    fi
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
