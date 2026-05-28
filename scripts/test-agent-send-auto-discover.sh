#!/usr/bin/env bash
# T-1834 — tests for agent-send.sh --to <agent-id> auto-discover.
#
# Uses --dry-run to test the resolution layer without actually posting/
# injecting. Five tests cover the success path + four error paths.
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
SCRIPT="${SCRIPT:-scripts/agent-send.sh}"
HEARTBEAT="${HEARTBEAT:-scripts/listener-heartbeat.sh}"

PASS=0; FAIL=0; SKIP=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

if "$TERMLINK" hub status >/dev/null 2>&1; then hub_up=1; else hub_up=0; fi

run_tag="t1834-$$-$(date +%s)"

# -------- T1: --to + --to-session both → exit 2 (mutex) --------
echo "T1: --to + --to-session both given → exit 2"
out="$(bash "$SCRIPT" --to "$run_tag-a" --to-session "X" --message "m" --dry-run 2>&1)"
rc=$?
if [ "$rc" -eq 2 ] && printf '%s' "$out" | grep -qF "mutex"; then
    pass "T1: mutex enforced (exit=$rc)"
else
    fail "T1: rc=$rc out=$out"
fi

# -------- T2: --to <unknown> → exit 2 with not-found --------
echo "T2: --to <unknown> → exit 2 with not-found"
if [ "$hub_up" -ne 1 ]; then
    skip "T2: local hub not up"
else
    out="$(bash "$SCRIPT" --to "definitely-not-listening-$run_tag" --message "m" --dry-run 2>&1)"
    rc=$?
    if [ "$rc" -eq 2 ] && printf '%s' "$out" | grep -qiF "no listener"; then
        pass "T2: not-found surfaced"
    else
        fail "T2: rc=$rc out=$out"
    fi
fi

# -------- T3: --to <agent> with full metadata + --dry-run → RESOLVED line --------
echo "T3: --to <agent> with pty_session + dm:* listen_topic + --dry-run"
if [ "$hub_up" -ne 1 ]; then
    skip "T3: local hub not up"
else
    aid="t3-good-$run_tag"
    bash "$HEARTBEAT" --agent-id "$aid" --pty-session "pty-t3-$run_tag" --listen-topic "dm:alice:bob" --listen-topic "agent-chat-arc" --once >/dev/null 2>&1
    sleep 1
    out="$(bash "$SCRIPT" --to "$aid" --message "m" --dry-run 2>&1)"
    rc=$?
    if [ "$rc" -eq 0 ] \
        && printf '%s' "$out" | grep -qF "RESOLVED:" \
        && printf '%s' "$out" | grep -qF "to_session=pty-t3-$run_tag" \
        && printf '%s' "$out" | grep -qF "topic=dm:alice:bob" \
        && printf '%s' "$out" | grep -qF "status=LIVE"; then
        pass "T3: RESOLVED with expected to_session+topic+status"
    else
        fail "T3: rc=$rc out=$out"
    fi
fi

# -------- T4: --to <agent> with no pty_session → exit 2 --------
echo "T4: --to <agent> heartbeat lacks pty_session → exit 2"
if [ "$hub_up" -ne 1 ]; then
    skip "T4: local hub not up"
else
    aid="t4-nopty-$run_tag"
    bash "$HEARTBEAT" --agent-id "$aid" --listen-topic "dm:alice:bob" --once >/dev/null 2>&1
    sleep 1
    out="$(bash "$SCRIPT" --to "$aid" --message "m" --dry-run 2>&1)"
    rc=$?
    if [ "$rc" -eq 2 ] && printf '%s' "$out" | grep -qiF "pty_session"; then
        pass "T4: pty_session-missing surfaced"
    else
        fail "T4: rc=$rc out=$out"
    fi
fi

# -------- T5: --to <agent> with no dm:* listen_topic → exit 2 --------
echo "T5: --to <agent> no dm:* listen_topic → exit 2"
if [ "$hub_up" -ne 1 ]; then
    skip "T5: local hub not up"
else
    aid="t5-nodm-$run_tag"
    bash "$HEARTBEAT" --agent-id "$aid" --pty-session "pty-t5" --listen-topic "agent-chat-arc" --once >/dev/null 2>&1
    sleep 1
    out="$(bash "$SCRIPT" --to "$aid" --message "m" --dry-run 2>&1)"
    rc=$?
    if [ "$rc" -eq 2 ] && printf '%s' "$out" | grep -qiF "dm:"; then
        pass "T5: no-dm-topic surfaced"
    else
        fail "T5: rc=$rc out=$out"
    fi
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
