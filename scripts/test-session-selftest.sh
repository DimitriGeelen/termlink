#!/usr/bin/env bash
# test-session-selftest.sh (T-2485) -- host-independent unit tests for the
# control-terminal-sessions prover. Drives every path via the PL-213 hooks
# (TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC / _EXEC_JSON + the SESSION_SELFTEST_SENTINEL
# test seam), so no live hub / no real tmux session is touched.
#
# Prints one line per case + a final "PASS"/"FAIL" summary (P-011 greps "PASS").
set -uo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT="$SELF_DIR/session-selftest.sh"

fails=0
check() { # <label> <expected-exit> <actual-exit>
    if [ "$2" -eq "$3" ]; then echo "  ok   $1 (exit $3)"; else echo "  FAIL $1 (expected $2, got $3)"; fails=$((fails+1)); fi
}

FIX="FIXED-SENTINEL-TOKEN"
# Canned exec --json outputs.
GOOD_JSON="{\"ok\":true,\"exit_code\":0,\"stderr\":\"\",\"stdout\":\"${FIX}\\n\"}"
BADCODE_JSON="{\"ok\":true,\"exit_code\":1,\"stderr\":\"boom\",\"stdout\":\"\"}"
NOSENT_JSON="{\"ok\":true,\"exit_code\":0,\"stderr\":\"\",\"stdout\":\"something else\\n\"}"
NOPTY_JSON="{\"ok\":false,\"exit_code\":null,\"stdout\":\"\",\"error\":\"Session has no PTY\"}"

# run <spawn_rc> <exec_json> -> exit code (single-shot: TEST_MODE short-circuits retry)
run() {
    SESSION_SELFTEST_SENTINEL="$FIX" \
    TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC="$1" \
    TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON="$2" \
        bash "$SCRIPT" --json >/dev/null 2>&1; echo $?
}

echo "test-session-selftest:"

# 1. proven — spawn ok + exec ok+0+sentinel. Expect 0.
check "proven (spawn ok, exec ok+0+sentinel) -> 0" 0 "$(run 0 "$GOOD_JSON")"

# 2. broken at SPAWN — spawn rc 1 (exec canned but never reached). Expect 1.
check "broken at SPAWN (spawn rc=1) -> 1" 1 "$(run 1 "$GOOD_JSON")"

# 3. broken at EXEC — non-zero exit_code. Expect 1.
check "broken at EXEC (exit_code=1) -> 1" 1 "$(run 0 "$BADCODE_JSON")"

# 4. broken at EXEC — sentinel absent from stdout. Expect 1.
check "broken at EXEC (sentinel missing) -> 1" 1 "$(run 0 "$NOSENT_JSON")"

# 5. broken at EXEC — ok:false (no-PTY class). Expect 1.
check "broken at EXEC (ok:false / no-PTY) -> 1" 1 "$(run 0 "$NOPTY_JSON")"

# 6. tooling — unknown arg. Expect 2 (fail-closed exit path).
bash "$SCRIPT" --bogus-flag >/dev/null 2>&1; check "unknown arg -> tooling(2)" 2 "$?"

# 7. JSON envelope on proven names the stages + sentinel.
jp="$(SESSION_SELFTEST_SENTINEL="$FIX" TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC=0 \
      TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON="$GOOD_JSON" bash "$SCRIPT" --json 2>/dev/null)"
if printf '%s' "$jp" | jq -e '.proven==true and .broken_stage==null and .stages.spawn=="PASS" and .stages.exec=="PASS" and (.sentinel=="'"$FIX"'")' >/dev/null 2>&1; then
    echo "  ok   json proven envelope well-formed"
else
    echo "  FAIL json proven envelope"; fails=$((fails+1))
fi

# 8. JSON envelope on broken-at-EXEC names the stage.
jb="$(SESSION_SELFTEST_SENTINEL="$FIX" TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC=0 \
      TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON="$BADCODE_JSON" bash "$SCRIPT" --json 2>/dev/null)"
if printf '%s' "$jb" | jq -e '.proven==false and .broken_stage=="EXEC"' >/dev/null 2>&1; then
    echo "  ok   json broken envelope names EXEC"
else
    echo "  FAIL json broken envelope"; fails=$((fails+1))
fi

# 9. --help documents the charter verb.
if bash "$SCRIPT" --help 2>&1 | grep -q "control terminal sessions"; then
    echo "  ok   --help documents the charter verb"
else
    echo "  FAIL --help missing charter verb"; fails=$((fails+1))
fi

echo
if [ "$fails" -eq 0 ]; then echo "test-session-selftest: PASS"; exit 0
else echo "test-session-selftest: FAIL ($fails failing)"; exit 1; fi
