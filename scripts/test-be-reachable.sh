#!/usr/bin/env bash
# T-1841 — unit tests for scripts/be-reachable.sh.
#
# Uses BE_REACHABLE_LH_SCRIPT to swap listener-heartbeat.sh for a sleep
# stub so tests don't depend on a live hub. Uses BE_REACHABLE_STATE_DIR
# to isolate test state under a temp dir.
set -u

SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
BE_REACHABLE="${SELF_DIR}/be-reachable.sh"

PASS=0
FAIL=0
FAILED_TESTS=()

note() { printf '\n=== %s ===\n' "$1"; }
ok() { PASS=$((PASS+1)); echo "  ✓ $1"; }
bad() { FAIL=$((FAIL+1)); FAILED_TESTS+=("$1"); echo "  ✗ $1" >&2; }

assert_eq() {
    # $1 expected, $2 actual, $3 message
    if [ "$1" = "$2" ]; then
        ok "$3 ($2)"
    else
        bad "$3: expected '$1' got '$2'"
    fi
}

assert_contains() {
    # $1 needle, $2 haystack, $3 message
    if printf '%s' "$2" | grep -qF -- "$1"; then
        ok "$3"
    else
        bad "$3: expected to contain '$1' in: $2"
    fi
}

# ---- setup ---------------------------------------------------------------

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"; pkill -P $$ 2>/dev/null || true' EXIT INT TERM

STUB="${TMP}/lh-stub.sh"
cat >"$STUB" <<'EOF'
#!/usr/bin/env bash
# Mock listener-heartbeat.sh — accepts the same flags but just sleeps.
# Exit on TERM/INT cleanly.
trap 'exit 0' TERM INT
# Accept and discard args; just print them to a side log if requested.
if [ -n "${LH_STUB_LOG:-}" ]; then
    printf 'stub-args: %s\n' "$*" > "$LH_STUB_LOG"
fi
sleep 600 &
wait $!
EOF
chmod +x "$STUB"

export BE_REACHABLE_LH_SCRIPT="$STUB"
export BE_REACHABLE_STATE_DIR="$TMP/state"
export BE_REACHABLE_STATE="$TMP/state/be-reachable.state"

# ---- test 1: --help works -----------------------------------------------
note "test 1: --help works"
help_out="$(bash "$BE_REACHABLE" --help 2>&1)"
assert_contains "Usage: be-reachable.sh" "$help_out" "help prints Usage line"
assert_contains "start" "$help_out" "help lists start subcommand"
assert_contains "stop" "$help_out" "help lists stop subcommand"
assert_contains "status" "$help_out" "help lists status subcommand"

# ---- test 2: unknown subcommand exits 2 ---------------------------------
note "test 2: unknown subcommand exits 2"
bash "$BE_REACHABLE" frobnicate >/dev/null 2>&1
assert_eq 2 $? "unknown subcommand exits 2"

# ---- test 3: status when not running exits 1 ----------------------------
note "test 3: status when not running"
status_out="$(bash "$BE_REACHABLE" status 2>&1)"
status_rc=$?
assert_eq 1 "$status_rc" "status with no state exits 1"
assert_contains "not running" "$status_out" "status reports not running"

status_json="$(bash "$BE_REACHABLE" status --json 2>&1)"
assert_contains '"running": false' "$status_json" "status --json says running=false"

# ---- test 4: start writes state + spawns child --------------------------
note "test 4: start writes state and spawns child"
start_out="$(bash "$BE_REACHABLE" start --agent-id test-agent-1 --interval 5 2>&1)"
start_rc=$?
assert_eq 0 "$start_rc" "start exits 0"
assert_contains "agent_id:      test-agent-1" "$start_out" "start echoes agent_id"

# Give the state file a moment to settle.
sleep 1

if [ -f "$BE_REACHABLE_STATE" ]; then
    ok "state file exists"
    if command -v jq >/dev/null 2>&1; then
        agent_id="$(jq -r .agent_id "$BE_REACHABLE_STATE")"
        pid="$(jq -r .pid "$BE_REACHABLE_STATE")"
        assert_eq "test-agent-1" "$agent_id" "state.agent_id"
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            ok "spawned PID $pid is alive"
        else
            bad "spawned PID $pid is NOT alive"
        fi
    else
        echo "  (jq missing — skipping JSON field checks)"
    fi
else
    bad "state file was not created at $BE_REACHABLE_STATE"
fi

# ---- test 5: start is idempotent ----------------------------------------
note "test 5: start is idempotent when already running"
start2_out="$(bash "$BE_REACHABLE" start 2>&1)"
start2_rc=$?
assert_eq 0 "$start2_rc" "second start exits 0"
assert_contains "already running" "$start2_out" "second start says 'already running'"

# ---- test 6: status when running ----------------------------------------
note "test 6: status when running"
status_out="$(bash "$BE_REACHABLE" status 2>&1)"
status_rc=$?
assert_eq 0 "$status_rc" "status when running exits 0"
assert_contains "running" "$status_out" "status reports running"
assert_contains "test-agent-1" "$status_out" "status names agent_id"

status_json="$(bash "$BE_REACHABLE" status --json 2>&1)"
assert_contains '"running": true' "$status_json" "status --json says running=true"

# ---- test 7: stop kills + clears ----------------------------------------
note "test 7: stop kills and clears state"
if command -v jq >/dev/null 2>&1; then
    stopped_pid="$(jq -r .pid "$BE_REACHABLE_STATE")"
else
    stopped_pid=""
fi
stop_out="$(bash "$BE_REACHABLE" stop 2>&1)"
stop_rc=$?
assert_eq 0 "$stop_rc" "stop exits 0"
assert_contains "stopped" "$stop_out" "stop reports 'stopped'"
if [ -f "$BE_REACHABLE_STATE" ]; then
    bad "state file should be gone after stop"
else
    ok "state file removed after stop"
fi
if [ -n "$stopped_pid" ] && kill -0 "$stopped_pid" 2>/dev/null; then
    bad "child PID $stopped_pid still alive after stop"
else
    ok "child PID gone after stop"
fi

# ---- test 8: stop is idempotent -----------------------------------------
note "test 8: stop when not running"
stop2_out="$(bash "$BE_REACHABLE" stop 2>&1)"
stop2_rc=$?
assert_eq 0 "$stop2_rc" "second stop exits 0"
assert_contains "not running" "$stop2_out" "second stop says 'not running'"

# ---- test 9: default agent_id derivation --------------------------------
note "test 9: default agent_id derivation"
# Run start without --agent-id and inspect the state.
bash "$BE_REACHABLE" start >/dev/null 2>&1
if command -v jq >/dev/null 2>&1; then
    derived_id="$(jq -r .agent_id "$BE_REACHABLE_STATE")"
    if [ -n "$derived_id" ] && [ "$derived_id" != "null" ]; then
        # Must contain "claude" infix
        if printf '%s' "$derived_id" | grep -q claude; then
            ok "default agent_id contains 'claude' ($derived_id)"
        else
            bad "default agent_id missing 'claude' infix ($derived_id)"
        fi
        # Must be lowercase + [-a-z0-9] only
        if printf '%s' "$derived_id" | grep -Eq '^[a-z0-9][a-z0-9-]*[a-z0-9]$'; then
            ok "default agent_id is normalized"
        else
            bad "default agent_id has unexpected chars ($derived_id)"
        fi
    else
        bad "default agent_id was empty"
    fi
fi
bash "$BE_REACHABLE" stop >/dev/null 2>&1 || true

# ---- test 10: default listen_topics = dm:<id>:* + agent-chat-arc -------
note "test 10: default listen_topics"
bash "$BE_REACHABLE" start --agent-id test-topics-id >/dev/null 2>&1
if command -v jq >/dev/null 2>&1; then
    lt="$(jq -c '.listen_topics' "$BE_REACHABLE_STATE")"
    assert_contains "dm:test-topics-id:*" "$lt" "default listen_topics contains dm:<id>:*"
    assert_contains "agent-chat-arc" "$lt" "default listen_topics contains agent-chat-arc"
fi
bash "$BE_REACHABLE" stop >/dev/null 2>&1 || true

# ---- test 11: stale state recovery (PID dead, state present) -----------
note "test 11: stale state recovery"
# Write a fake state with a dead PID.
mkdir -p "$BE_REACHABLE_STATE_DIR"
cat >"$BE_REACHABLE_STATE" <<EOF
{
  "agent_id": "stale-agent",
  "pid": 999999,
  "started_at": "2020-01-01T00:00:00Z",
  "role": "claude-code",
  "interval": 30,
  "topic": "agent-presence",
  "listen_topics": ["dm:stale-agent:*"],
  "pty_session": "",
  "hub": ""
}
EOF
# status should report stale + exit 1
stale_status="$(bash "$BE_REACHABLE" status 2>&1)"
stale_rc=$?
assert_eq 1 "$stale_rc" "status on stale state exits 1"
assert_contains "stale state" "$stale_status" "status says 'stale state'"
# start should clear stale state and start fresh
bash "$BE_REACHABLE" start --agent-id fresh-agent >/dev/null 2>&1
if command -v jq >/dev/null 2>&1; then
    fresh_id="$(jq -r .agent_id "$BE_REACHABLE_STATE")"
    assert_eq "fresh-agent" "$fresh_id" "start cleared stale state and used new agent_id"
fi
bash "$BE_REACHABLE" stop >/dev/null 2>&1 || true

# ---- test 12: missing LH script → exit 3 --------------------------------
note "test 12: missing listener-heartbeat.sh fails clean"
BE_REACHABLE_LH_SCRIPT_BACKUP="$BE_REACHABLE_LH_SCRIPT"
export BE_REACHABLE_LH_SCRIPT="/nonexistent/listener-heartbeat.sh"
bash "$BE_REACHABLE" start 2>/dev/null
missing_rc=$?
assert_eq 3 "$missing_rc" "missing LH script exits 3"
export BE_REACHABLE_LH_SCRIPT="$BE_REACHABLE_LH_SCRIPT_BACKUP"

# ---- summary -------------------------------------------------------------
echo
echo "==================================="
echo "  PASS: $PASS"
echo "  FAIL: $FAIL"
if [ "$FAIL" -gt 0 ]; then
    echo "  Failed:"
    for t in "${FAILED_TESTS[@]}"; do
        echo "    - $t"
    done
fi
echo "==================================="

[ "$FAIL" -eq 0 ]
