#!/usr/bin/env bash
# T-1832 — tests for listener-heartbeat.sh.
#
# Covers:
#   T1 --help → exit 0 with usage
#   T2 unknown arg → exit 2
#   T3 missing --agent-id → exit 2
#   T4 --once --json against local hub → posts heartbeat, JSON parseable,
#      envelope visible via channel subscribe with expected metadata
#   T5 --once with multiple --listen-topic → metadata.listen_topics csv
#   T6 --interval too small → exit 2
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
SCRIPT="${SCRIPT:-scripts/listener-heartbeat.sh}"

PASS=0; FAIL=0; SKIP=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

# Pre-flight: local hub up? (affects T4/T5 only)
if "$TERMLINK" hub status >/dev/null 2>&1; then hub_up=1; else hub_up=0; fi

# Unique per-run agent_id so subscribe filter doesn't collide with prior runs.
run_id="t-1832-$$-$(date +%s)"

# -------- T1: --help → exit 0 with usage --------
echo "T1: --help → exit 0 with usage"
out="$(bash "$SCRIPT" --help 2>/dev/null)"
rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF "Usage:"; then
    pass "T1: --help exit=0 with usage"
else
    fail "T1: exit=$rc out=$out"
fi

# -------- T2: unknown arg → exit 2 --------
echo "T2: unknown arg → exit 2"
if bash "$SCRIPT" --bogus >/dev/null 2>&1; then
    fail "T2: should have failed on --bogus"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T2: exit=$rc"
    else fail "T2: expected 2, got $rc"; fi
fi

# -------- T3: missing --agent-id → exit 2 --------
echo "T3: missing --agent-id → exit 2"
if bash "$SCRIPT" --once >/dev/null 2>&1; then
    fail "T3: should have failed without --agent-id"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T3: exit=$rc"
    else fail "T3: expected 2, got $rc"; fi
fi

# -------- T4: --once --json → posts heartbeat, envelope visible --------
echo "T4: --once --json against local hub"
if [ "$hub_up" -ne 1 ]; then
    skip "T4: local hub not up"
else
    aid="$run_id-T4"
    out="$(bash "$SCRIPT" --agent-id "$aid" --once --json 2>/dev/null)"
    rc=$?
    offset="$(printf '%s' "$out" | jq -r '.delivered.offset // empty' 2>/dev/null)"
    if [ "$rc" -eq 0 ] && [ -n "$offset" ]; then
        # Now read back from the topic and confirm the envelope.
        # Limit/scan recent envelopes and filter by metadata.agent_id.
        found_envelope="$(termlink channel subscribe agent-presence --limit 50 --json 2>/dev/null | jq -c "select(.metadata.agent_id == \"$aid\")" | tail -1)"
        if [ -n "$found_envelope" ]; then
            mt="$(printf '%s' "$found_envelope" | jq -r '.msg_type')"
            mid="$(printf '%s' "$found_envelope" | jq -r '.metadata.agent_id')"
            if [ "$mt" = "heartbeat" ] && [ "$mid" = "$aid" ]; then
                pass "T4: heartbeat posted (offset=$offset) + readable + msg_type=heartbeat"
            else
                fail "T4: envelope mismatch — msg_type=$mt agent_id=$mid"
            fi
        else
            fail "T4: envelope not found via subscribe filter"
        fi
    else
        fail "T4: rc=$rc out=$out"
    fi
fi

# -------- T5: multiple --listen-topic → csv in metadata.listen_topics --------
echo "T5: multiple --listen-topic → comma-joined"
if [ "$hub_up" -ne 1 ]; then
    skip "T5: local hub not up"
else
    aid="$run_id-T5"
    out="$(bash "$SCRIPT" --agent-id "$aid" --listen-topic foo --listen-topic bar --once --json 2>/dev/null)"
    rc=$?
    if [ "$rc" -eq 0 ]; then
        found_envelope="$(termlink channel subscribe agent-presence --limit 50 --json 2>/dev/null | jq -c "select(.metadata.agent_id == \"$aid\")" | tail -1)"
        listen_topics_field="$(printf '%s' "$found_envelope" | jq -r '.metadata.listen_topics // ""')"
        # Order is preserved as passed: foo,bar
        if [ "$listen_topics_field" = "foo,bar" ]; then
            pass "T5: listen_topics='foo,bar'"
        else
            fail "T5: expected 'foo,bar', got '$listen_topics_field'"
        fi
    else
        fail "T5: rc=$rc out=$out"
    fi
fi

# -------- T6: --interval too small → exit 2 --------
echo "T6: --interval 1 → exit 2 (below minimum 5)"
if bash "$SCRIPT" --agent-id x --interval 1 --once >/dev/null 2>&1; then
    fail "T6: should have failed on --interval 1"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T6: exit=$rc"
    else fail "T6: expected 2, got $rc"; fi
fi

# -------- T7: --pty-session round-trips to metadata.pty_session (T-1834) --------
echo "T7: --pty-session round-trips; omitted means field absent"
if [ "$hub_up" -ne 1 ]; then
    skip "T7: local hub not up"
else
    aid_with="$run_id-T7-with"
    aid_without="$run_id-T7-without"
    bash "$SCRIPT" --agent-id "$aid_with" --pty-session "pty-soak-foo" --once >/dev/null 2>&1
    bash "$SCRIPT" --agent-id "$aid_without" --once >/dev/null 2>&1

    env_with="$(termlink channel subscribe agent-presence --limit 50 --json 2>/dev/null | jq -c "select(.metadata.agent_id == \"$aid_with\")" | tail -1)"
    env_without="$(termlink channel subscribe agent-presence --limit 50 --json 2>/dev/null | jq -c "select(.metadata.agent_id == \"$aid_without\")" | tail -1)"

    pty_with="$(printf '%s' "$env_with" | jq -r '.metadata.pty_session // "ABSENT"')"
    pty_without="$(printf '%s' "$env_without" | jq -r '.metadata.pty_session // "ABSENT"')"

    if [ "$pty_with" = "pty-soak-foo" ] && [ "$pty_without" = "ABSENT" ]; then
        pass "T7: with --pty-session='pty-soak-foo' present; omitted -> absent"
    else
        fail "T7: with=$pty_with (want 'pty-soak-foo'), without=$pty_without (want ABSENT)"
    fi
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
