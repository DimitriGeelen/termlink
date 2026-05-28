#!/usr/bin/env bash
# T-1833 — tests for agent-listeners.sh.
#
# Most tests use a fresh ephemeral topic per test so cross-contamination
# is impossible. Status-classification tests (T5/T7) use --interval=5 and
# wait the requisite seconds — slow but mechanical.
#
# Covers:
#   T1  --help → exit 0 with usage
#   T2  unknown arg → exit 2
#   T3  empty topic → ok=true total=0 (created topic, no heartbeats)
#   T4  one LIVE heartbeat → live=1
#   T5  STALE classification (interval=5, age > 10s)
#   T6  --filter-agent-id narrows correctly
#   T7  --include-offline surfaces OFFLINE (interval=5, age > 25s)
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
SCRIPT="${SCRIPT:-scripts/agent-listeners.sh}"
HEARTBEAT="${HEARTBEAT:-scripts/listener-heartbeat.sh}"

PASS=0; FAIL=0; SKIP=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

# Pre-flight: local hub up? (T3..T7)
if "$TERMLINK" hub status >/dev/null 2>&1; then hub_up=1; else hub_up=0; fi

# Helper: post one heartbeat with --once to a custom topic.
post_hb() {
    local topic="$1" aid="$2" interval="$3"; shift 3
    bash "$HEARTBEAT" --agent-id "$aid" --interval "$interval" --topic "$topic" --once "$@" >/dev/null 2>&1
}

# -------- T1 --------
echo "T1: --help → exit 0 with usage"
out="$(bash "$SCRIPT" --help 2>/dev/null)"
rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF "Usage:"; then
    pass "T1"
else
    fail "T1: rc=$rc"
fi

# -------- T2 --------
echo "T2: unknown arg → exit 2"
if bash "$SCRIPT" --bogus >/dev/null 2>&1; then
    fail "T2: should have failed"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T2"
    else fail "T2: expected 2, got $rc"; fi
fi

# -------- T3 --------
echo "T3: empty topic → ok=true total=0"
if [ "$hub_up" -ne 1 ]; then
    skip "T3: local hub not up"
else
    topic="agent-listeners-test-T3-$$-$(date +%s)"
    "$TERMLINK" channel create "$topic" --retention messages:10 >/dev/null 2>&1
    out="$(bash "$SCRIPT" --topic "$topic" --json 2>/dev/null)"
    rc=$?
    ok="$(printf '%s' "$out" | jq -r '.ok')"
    tot="$(printf '%s' "$out" | jq -r '.total_listeners')"
    if [ "$rc" -eq 0 ] && [ "$ok" = "true" ] && [ "$tot" = "0" ]; then
        pass "T3: empty topic ok=true total=0"
    else
        fail "T3: rc=$rc ok=$ok total=$tot"
    fi
fi

# -------- T4 --------
echo "T4: one LIVE heartbeat → live=1"
if [ "$hub_up" -ne 1 ]; then
    skip "T4: local hub not up"
else
    topic="agent-listeners-test-T4-$$-$(date +%s)"
    aid="agent-T4-$$-$(date +%s)"
    "$TERMLINK" channel create "$topic" --retention messages:10 >/dev/null 2>&1
    post_hb "$topic" "$aid" 30  # interval=30 → LIVE for next 60s
    out="$(bash "$SCRIPT" --topic "$topic" --json 2>/dev/null)"
    rc=$?
    live="$(printf '%s' "$out" | jq -r '.live')"
    total="$(printf '%s' "$out" | jq -r '.total_listeners')"
    found_status="$(printf '%s' "$out" | jq -r --arg aid "$aid" '.listeners[] | select(.agent_id==$aid) | .status')"
    if [ "$rc" -eq 0 ] && [ "$live" = "1" ] && [ "$total" = "1" ] && [ "$found_status" = "LIVE" ]; then
        pass "T4: live=1 total=1 status=LIVE"
    else
        fail "T4: rc=$rc live=$live total=$total status=$found_status"
    fi
fi

# -------- T5 --------
echo "T5: STALE classification (interval=5, age > 10s)"
if [ "$hub_up" -ne 1 ]; then
    skip "T5: local hub not up"
else
    topic="agent-listeners-test-T5-$$-$(date +%s)"
    aid="agent-T5-$$-$(date +%s)"
    "$TERMLINK" channel create "$topic" --retention messages:10 >/dev/null 2>&1
    post_hb "$topic" "$aid" 5  # interval=5 → STALE after 10s, OFFLINE after 25s
    sleep 11
    out="$(bash "$SCRIPT" --topic "$topic" --json 2>/dev/null)"
    rc=$?
    stale="$(printf '%s' "$out" | jq -r '.stale')"
    found_status="$(printf '%s' "$out" | jq -r --arg aid "$aid" '.listeners[] | select(.agent_id==$aid) | .status')"
    if [ "$rc" -eq 0 ] && [ "$stale" = "1" ] && [ "$found_status" = "STALE" ]; then
        pass "T5: stale=1 status=STALE"
    else
        fail "T5: rc=$rc stale=$stale status=$found_status"
    fi
fi

# -------- T6 --------
echo "T6: --filter-agent-id narrows correctly"
if [ "$hub_up" -ne 1 ]; then
    skip "T6: local hub not up"
else
    topic="agent-listeners-test-T6-$$-$(date +%s)"
    aid_a="agent-T6a-$$-$(date +%s)"
    aid_b="agent-T6b-$$-$(date +%s)"
    "$TERMLINK" channel create "$topic" --retention messages:10 >/dev/null 2>&1
    post_hb "$topic" "$aid_a" 30
    post_hb "$topic" "$aid_b" 30
    out="$(bash "$SCRIPT" --topic "$topic" --filter-agent-id "$aid_a" --json 2>/dev/null)"
    rc=$?
    total="$(printf '%s' "$out" | jq -r '.total_listeners')"
    only_aid="$(printf '%s' "$out" | jq -r '.listeners[0].agent_id // ""')"
    if [ "$rc" -eq 0 ] && [ "$total" = "1" ] && [ "$only_aid" = "$aid_a" ]; then
        pass "T6: filter narrowed to $aid_a"
    else
        fail "T6: rc=$rc total=$total only_aid=$only_aid"
    fi
fi

# -------- T7 --------
echo "T7: --include-offline surfaces OFFLINE (interval=5, age > 25s) — slow"
if [ "$hub_up" -ne 1 ]; then
    skip "T7: local hub not up"
else
    topic="agent-listeners-test-T7-$$-$(date +%s)"
    aid="agent-T7-$$-$(date +%s)"
    "$TERMLINK" channel create "$topic" --retention messages:10 >/dev/null 2>&1
    post_hb "$topic" "$aid" 5  # interval=5; needs > 25s for OFFLINE
    sleep 27
    # Default — should NOT include OFFLINE.
    out1="$(bash "$SCRIPT" --topic "$topic" --json 2>/dev/null)"
    tot_default="$(printf '%s' "$out1" | jq -r '.total_listeners')"
    # With --include-offline.
    out2="$(bash "$SCRIPT" --topic "$topic" --include-offline --json 2>/dev/null)"
    tot_inc="$(printf '%s' "$out2" | jq -r '.total_listeners')"
    off_inc="$(printf '%s' "$out2" | jq -r '.offline')"
    status_inc="$(printf '%s' "$out2" | jq -r --arg aid "$aid" '.listeners[] | select(.agent_id==$aid) | .status')"
    if [ "$tot_default" = "0" ] && [ "$tot_inc" = "1" ] && [ "$off_inc" = "1" ] && [ "$status_inc" = "OFFLINE" ]; then
        pass "T7: default hides OFFLINE; --include-offline surfaces it"
    else
        fail "T7: default_tot=$tot_default inc_tot=$tot_inc inc_off=$off_inc status=$status_inc"
    fi
fi

# -------- T8: pty_session round-trips through discovery (T-1834) --------
echo "T8: pty_session round-trips through discovery"
if [ "$hub_up" -ne 1 ]; then
    skip "T8: local hub not up"
else
    topic="agent-listeners-test-T8-$$-$(date +%s)"
    aid="agent-T8-$$-$(date +%s)"
    "$TERMLINK" channel create "$topic" --retention messages:10 >/dev/null 2>&1
    bash "$HEARTBEAT" --agent-id "$aid" --pty-session "soak-pty-T8" --topic "$topic" --once >/dev/null 2>&1
    out="$(bash "$SCRIPT" --topic "$topic" --json 2>/dev/null)"
    pty="$(printf '%s' "$out" | jq -r --arg aid "$aid" '.listeners[] | select(.agent_id==$aid) | .pty_session // ""')"
    if [ "$pty" = "soak-pty-T8" ]; then
        pass "T8: pty_session='soak-pty-T8' surfaced"
    else
        fail "T8: expected 'soak-pty-T8', got '$pty'"
    fi
fi

# -------- T9: G-060 graceful degradation — unknown topic on healthy hub → exit 0 empty (T-1842) --------
echo "T9: unknown topic on healthy hub → exit 0 with empty rollup"
if [ "$hub_up" -ne 1 ]; then
    skip "T9: local hub not up"
else
    # Use a topic name that's extremely unlikely to exist. Hub is healthy
    # (we passed pre-flight), so the failure must be -32013, not network.
    bogus_topic="agent-listeners-test-T9-NEVER-CREATED-$$-$(date +%s)"
    out="$(bash "$SCRIPT" --topic "$bogus_topic" --json 2>/dev/null)"
    rc=$?
    ok="$(printf '%s' "$out" | jq -r '.ok')"
    tot="$(printf '%s' "$out" | jq -r '.total_listeners')"
    listeners_len="$(printf '%s' "$out" | jq -r '.listeners | length')"
    if [ "$rc" -eq 0 ] && [ "$ok" = "true" ] && [ "$tot" = "0" ] && [ "$listeners_len" = "0" ]; then
        pass "T9: unknown topic → exit 0 ok=true total=0 listeners=[]"
    else
        fail "T9: rc=$rc ok=$ok total=$tot listeners_len=$listeners_len"
    fi
fi

# -------- T10: real subscribe failure still exits 3 (T-1842 — non-32013 path preserved) --------
echo "T10: subscribe failure other than -32013 still exits 3"
# Use an unreachable hub address. Should fail with network error, NOT -32013.
out="$(bash "$SCRIPT" --hub 192.0.2.99:9100 --json 2>/dev/null)"
rc=$?
if [ "$rc" -eq 3 ]; then
    pass "T10: unreachable hub → exit 3 preserved"
else
    fail "T10: expected 3, got $rc (out=$out)"
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
