#!/usr/bin/env bash
# test-comms-selftest.sh (T-2482) -- host-independent unit tests for the staged
# comms round-trip prover. Feeds a canned presence fixture (inherited by
# diagnose-unconsumed.sh via TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON) for the
# DISCOVER stage, and a canned agent-send exit code (COMMS_SELFTEST_TEST_SEND_RC)
# for the SEND+CONSUME stages — no live hub needed.
#
# Prints one line per case + a final "PASS"/"FAIL" summary (P-011 greps "PASS").
set -uo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT="$SELF_DIR/comms-selftest.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fails=0
check() { # <label> <expected-exit> <actual-exit>
    if [ "$2" -eq "$3" ]; then echo "  ok   $1 (exit $3)"; else echo "  FAIL $1 (expected $2, got $3)"; fails=$((fails+1)); fi
}

# --- presence fixtures ------------------------------------------------------
cat > "$TMP/live_armed.json" <<'JSON'
{"listeners":[{"agent_id":"peerX","status":"LIVE","age_secs":10,"pty_session":"pts-7"}]}
JSON
cat > "$TMP/live_unarmed.json" <<'JSON'
{"listeners":[{"agent_id":"peerX","status":"LIVE","age_secs":10,"pty_session":null}]}
JSON
cat > "$TMP/dead.json" <<'JSON'
{"listeners":[{"agent_id":"someone-else","status":"LIVE","age_secs":5,"pty_session":"pts-1"}]}
JSON

echo "test-comms-selftest:"

# 1. all-green: armed peer + send rc 0 -> round-trip proven (0)
check "all-green (armed + send=0) -> proven" 0 "$(
    TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$TMP/live_armed.json" COMMS_SELFTEST_TEST_SEND_RC=0 \
    bash "$SCRIPT" --peer peerX >/dev/null 2>&1; echo $?)"

# 2. CONSUME fail: armed peer + send rc 3 (written, not acked) -> broken (1)
check "consume-fail (armed + send=3) -> broken" 1 "$(
    TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$TMP/live_armed.json" COMMS_SELFTEST_TEST_SEND_RC=3 \
    bash "$SCRIPT" --peer peerX >/dev/null 2>&1; echo $?)"

# 3. SEND fail: armed peer + send rc 2 (precondition) -> broken (1)
check "send-fail (armed + send=2) -> broken" 1 "$(
    TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$TMP/live_armed.json" COMMS_SELFTEST_TEST_SEND_RC=2 \
    bash "$SCRIPT" --peer peerX >/dev/null 2>&1; echo $?)"

# 3b. CONSUME timeout: armed peer + send rc 124 (bounded timeout tripped) -> broken (1)
check "consume-timeout (armed + send=124) -> broken" 1 "$(
    TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$TMP/live_armed.json" COMMS_SELFTEST_TEST_SEND_RC=124 \
    bash "$SCRIPT" --peer peerX >/dev/null 2>&1; echo $?)"

# 4. DISCOVER fail: unarmed peer -> broken at DISCOVER (1), no send attempted
check "discover-fail (unwakeable) -> broken" 1 "$(
    TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$TMP/live_unarmed.json" \
    bash "$SCRIPT" --peer peerX >/dev/null 2>&1; echo $?)"

# 5. DISCOVER fail: absent/dead peer -> broken at DISCOVER (1)
check "discover-fail (dead) -> broken" 1 "$(
    TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$TMP/dead.json" \
    bash "$SCRIPT" --peer peerX >/dev/null 2>&1; echo $?)"

# 6. --discover-only on armed peer -> pass (0), never touches send
check "discover-only (armed) -> pass" 0 "$(
    TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$TMP/live_armed.json" \
    bash "$SCRIPT" --peer peerX --discover-only >/dev/null 2>&1; echo $?)"

# 7. --discover-only on unarmed peer -> broken (1)
check "discover-only (unarmed) -> broken" 1 "$(
    TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$TMP/live_unarmed.json" \
    bash "$SCRIPT" --peer peerX --discover-only >/dev/null 2>&1; echo $?)"

# 8. discover-only must NOT fire a send even if a send-rc hook is (wrongly) set:
#    an armed peer with send hook 3 should still exit 0 because send is skipped.
check "discover-only ignores send hook" 0 "$(
    TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$TMP/live_armed.json" COMMS_SELFTEST_TEST_SEND_RC=3 \
    bash "$SCRIPT" --peer peerX --discover-only >/dev/null 2>&1; echo $?)"

# 9. tooling error: no presence readable -> exit 2 (fail-closed, not a false proof)
check "no presence -> tooling(2)" 2 "$(
    TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$TMP/nonexistent.json" \
    bash "$SCRIPT" --peer peerX >/dev/null 2>&1; echo $?)"

# 10. missing --peer -> tooling error (2)
bash "$SCRIPT" --json >/dev/null 2>&1; check "missing --peer -> tooling(2)" 2 "$?"

# 11. JSON mode emits per-stage breakdown with broken_stage on a CONSUME fail
json_out="$(TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$TMP/live_armed.json" COMMS_SELFTEST_TEST_SEND_RC=3 \
    bash "$SCRIPT" --peer peerX --json 2>/dev/null)"
if printf '%s' "$json_out" | jq -e '.broken_stage=="CONSUME" and (.stages|length==3) and (.stages[0].stage=="DISCOVER")' >/dev/null 2>&1; then
    echo "  ok   json breakdown (broken_stage=CONSUME, 3 stages)"
else
    echo "  FAIL json breakdown"; fails=$((fails+1))
fi

# 12. --help documents the three stages
help_out="$(bash "$SCRIPT" --help 2>&1)"
if printf '%s' "$help_out" | grep -q "DISCOVER" && printf '%s' "$help_out" | grep -q "CONSUME"; then
    echo "  ok   --help documents stages"
else
    echo "  FAIL --help missing stage names"; fails=$((fails+1))
fi

echo
if [ "$fails" -eq 0 ]; then echo "test-comms-selftest: PASS"; exit 0
else echo "test-comms-selftest: FAIL ($fails failing)"; exit 1; fi
