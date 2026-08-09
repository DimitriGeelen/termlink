#!/usr/bin/env bash
# session-selftest.sh (T-2485) — affirmative prover for the "control terminal
# sessions" charter verb.
#
# WHY: TermLink's charter names four core verbs — discover, exchange durable
# messages, claim work, and control terminal sessions. Affirmative on-demand provers
# already exist for three: comms-selftest.sh (T-2482) proves discover+exchange,
# substrate-smoke.sh (T-2151) proves claim work. The fourth — the FOUNDING verb
# ("TermLink began as a cross-terminal session-control tool", docs/CHARTER.md) — had
# NO affirmative prover. The gap was acknowledged in-code: agent-conversation-selftest.sh
# says "What it does NOT validate: PTY inject", and comms-selftest only checks the
# `pty_session` presence FLAG, never that a command actually injects and runs. So
# nothing proved, on demand, that you can register a terminal session and inject/exec
# into it right now (the G-069 shipped≠live class, applied to the PTY verb).
#
# WHAT IT PROVES (staged PASS/FAIL checks against a real session):
#   SPAWN         a tmux-backed scratch session registers on this host.
#   EXEC          `termlink exec <session> 'echo <sentinel>'` returns ok + exit_code 0
#                 with the unique sentinel in stdout AND truncated!=true — the
#                 inject → run → capture round-trip (T-2563: truncated check closes the
#                 executor.rs cap-band where a truncated result reads as exit_code 0).
#   EXEC_EXITCODE a negative exec (`sh -c 'exit 7'`) whose real non-zero code must
#                 propagate as exit_code==7 (T-2563 F2). The happy-path EXEC only ever
#                 exits 0, so without this stage a regression pinning exit_code:0 for
#                 ALL commands would still report PROVEN — a peer's `exec 'deploy.sh'`
#                 would then read success on failure. This proves exit-code FIDELITY.
#   CLEANUP       best-effort teardown (signal TERM + clean); NEVER fatal (a self-reaped
#                 session makes `signal` legitimately fail).
# Unlike doorbell-wake (needs a live peer), `termlink exec --json` makes this
# deterministic and local. This is an on-demand PROVER, NOT a cron canary — the
# affirmative complement to the detectors, and the 4th sibling of comms-selftest.
#
# EXIT CODES:
#   0  proven   -- SPAWN and EXEC both PASSed: the terminal-session verb works now.
#   1  broken   -- SPAWN or EXEC FAILed; the broken stage is named (not silent).
#   2  tooling  -- missing termlink/jq, or the local hub is down (fail-closed — an
#                  unprovable environment is NEVER reported "proven").
#
# USAGE:
#   session-selftest.sh [--json] [--ttl <secs>] [--hub <addr>] [--help]
#     --json        emit {ok, proven, broken_stage, stages, session, sentinel}
#     --ttl <secs>  scratch-session lifetime (default 30; it self-reaps as a backstop)
#     --hub <addr>  target hub (default: local hub)
#
# TEST HOOKS (PL-213 — host-independent, no live hub):
#   TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC=<int>        canned spawn exit code
#   TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON=<json>      canned `exec --json` output
#                                                        (set .truncated:true to test F3)
#   TERMLINK_SESSION_SELFTEST_TEST_EXITCODE_JSON=<json>  canned negative-exec `--json`
#                                                        output for the EXEC_EXITCODE stage
#
# See docs/operations/session-selftest.md. Sibling: scripts/comms-selftest.sh.
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
JSON_MODE=0
TTL=30
HUB=""

tooling() { # <msg> -- exit 2, fail-closed
    if [ "$JSON_MODE" -eq 1 ]; then
        printf '{"ok":false,"proven":false,"broken_stage":"tooling","error":%s}\n' "$(printf '%s' "$1" | jq -R . 2>/dev/null || echo "\"$1\"")"
    else
        echo "session-selftest: tooling error — $1 (fail-closed: cannot prove, so NOT reporting proven)" >&2
    fi
    exit 2
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON_MODE=1; shift ;;
        --ttl) TTL="${2:-30}"; shift 2 ;;
        --hub) HUB="${2:-}"; shift 2 ;;
        -h|--help)
            sed -n '2,52p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "session-selftest: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v jq >/dev/null 2>&1 || tooling "jq not found (required)"

HUB_ARGS=()
[ -n "$HUB" ] && HUB_ARGS=(--hub "$HUB")

# In test-hook mode we never touch a live hub. Otherwise confirm the local hub is
# up first — a hub-down is a tooling condition (exit 2), NOT "the PTY verb is broken".
TEST_MODE=0
if [ -n "${TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC:-}" ] || [ -n "${TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON:-}" ]; then
    TEST_MODE=1
fi
if [ "$TEST_MODE" -eq 0 ]; then
    command -v "$TERMLINK" >/dev/null 2>&1 || tooling "termlink not found on PATH"
    if [ -z "$HUB" ] && ! "$TERMLINK" hub status >/dev/null 2>&1; then
        tooling "local hub is not running (start it or pass --hub)"
    fi
fi

# Unique per-run identifiers. $RANDOM keeps concurrent selftests from colliding.
NONCE="$$-${RANDOM}${RANDOM}"
SESSION="session-selftest-${NONCE}"
# SESSION_SELFTEST_SENTINEL override is a test seam (PL-213): lets the test harness fix
# the sentinel so a canned TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON can carry it.
SENTINEL="${SESSION_SELFTEST_SENTINEL:-SESSION-SELFTEST-OK-${NONCE}}"

spawn_status="FAIL"; exec_status="FAIL"; cleanup_status="skipped"
broken=""; exec_detail=""

# --- STAGE 1: SPAWN ---------------------------------------------------------
if [ -n "${TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC:-}" ]; then
    spawn_rc="$TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC"
else
    "$TERMLINK" spawn --name "$SESSION" "${HUB_ARGS[@]}" -- sleep "$TTL" >/dev/null 2>&1
    spawn_rc=$?
fi
if [ "$spawn_rc" -eq 0 ]; then
    spawn_status="PASS"
else
    broken="SPAWN"
fi

# --- STAGE 2: EXEC (only if SPAWN passed) -----------------------------------
# `spawn` returns rc 0 before the tmux-backed shell is guaranteed exec-ready, so on
# the live path we retry the inject up to EXEC_READY_ATTEMPTS × EXEC_READY_SLEEP to
# absorb the occasional slow start. A genuinely broken verb still FAILs after the
# whole budget is spent (~5s); a ready session passes on attempt 1 at no extra cost.
# Test-hook mode is single-shot (the canned JSON is deterministic — no readiness race).
EXEC_READY_ATTEMPTS="${SESSION_SELFTEST_EXEC_ATTEMPTS:-10}"
EXEC_READY_SLEEP="${SESSION_SELFTEST_EXEC_SLEEP:-0.5}"
if [ "$spawn_status" = "PASS" ]; then
    attempt=0
    while : ; do
        attempt=$((attempt+1))
        if [ -n "${TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON:-}" ]; then
            exec_json="$TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON"
        else
            exec_json="$("$TERMLINK" exec "$SESSION" "echo $SENTINEL" --json "${HUB_ARGS[@]}" 2>/dev/null || true)"
        fi
        e_ok="$(printf '%s' "$exec_json" | jq -r '.ok // false' 2>/dev/null || echo false)"
        e_code="$(printf '%s' "$exec_json" | jq -r '.exit_code // "null"' 2>/dev/null || echo null)"
        e_out="$(printf '%s' "$exec_json" | jq -r '.stdout // ""' 2>/dev/null || echo '')"
        # F3 (T-2563): also require truncated != true. executor.rs:24-25 admits a
        # truncated result can read as exit_code:0 in the ~64 KiB band around the cap;
        # if the prover ignored .truncated, a silently-truncated round-trip that still
        # carries the sentinel and exits 0 would PASS — data loss reported as proven.
        e_trunc="$(printf '%s' "$exec_json" | jq -r '.truncated // false' 2>/dev/null || echo false)"
        if [ "$e_ok" = "true" ] && [ "$e_code" = "0" ] && [ "$e_trunc" != "true" ] && printf '%s' "$e_out" | grep -qF "$SENTINEL"; then
            exec_status="PASS"
            break
        fi
        # Give up only after the readiness budget is spent, OR immediately in test-hook
        # mode (deterministic canned output — retrying can never change the verdict).
        if [ "$TEST_MODE" -eq 1 ] || [ "$attempt" -ge "$EXEC_READY_ATTEMPTS" ]; then
            exec_status="FAIL"; broken="EXEC"
            exec_detail="ok=$e_ok exit_code=$e_code truncated=$e_trunc sentinel_found=$(printf '%s' "$e_out" | grep -qF "$SENTINEL" && echo yes || echo no) attempts=$attempt"
            break
        fi
        sleep "$EXEC_READY_SLEEP"
    done
fi

# --- STAGE 2b: EXEC_EXITCODE (F2, T-2563; only if EXEC passed) --------------
# The happy-path EXEC runs `echo` which always exits 0, so it cannot catch a
# regression that pins exit_code:0 for ALL commands (or drops the child's real
# status — executor.rs:229 `status.code().unwrap_or(-1)` is correct today, but the
# prover exists to catch a FUTURE regression of that line). This negative stage
# execs a command whose real exit code is non-zero and asserts the exact code
# propagates — proving exit-code fidelity, not just "success exits 0".
EXITCODE_EXPECT=7
exitcode_status="skipped"
if [ "$exec_status" = "PASS" ]; then
    if [ -n "${TERMLINK_SESSION_SELFTEST_TEST_EXITCODE_JSON:-}" ]; then
        ec_json="$TERMLINK_SESSION_SELFTEST_TEST_EXITCODE_JSON"
    elif [ "$TEST_MODE" -eq 1 ]; then
        # Test-hook mode with no explicit exitcode fixture: default to a faithful
        # non-zero propagation (ok=false is the real semantic for a failing command)
        # so pre-existing SPAWN/EXEC-focused cases stay proven without touching a
        # live hub. Cases that exercise this stage set the fixture explicitly.
        ec_json="{\"ok\":false,\"exit_code\":$EXITCODE_EXPECT}"
    else
        ec_json="$("$TERMLINK" exec "$SESSION" "sh -c 'exit $EXITCODE_EXPECT'" --json "${HUB_ARGS[@]}" 2>/dev/null || true)"
    fi
    ec_ok="$(printf '%s' "$ec_json" | jq -r '.ok // false' 2>/dev/null || echo false)"
    ec_code="$(printf '%s' "$ec_json" | jq -r '.exit_code // "null"' 2>/dev/null || echo null)"
    # The discriminator is `exit_code == EXITCODE_EXPECT` (7), NOT `ok`. Verified on
    # the live hub: `termlink exec --json` sets `ok` to whether the COMMAND succeeded
    # (exit 0), so a deliberately-failing command legitimately returns ok=false. A
    # total round-trip failure yields exit_code null/-1/empty (never exactly 7), and a
    # pinned-0 regression yields 0 — both ≠ 7, so both correctly FAIL this stage. The
    # exact non-zero code being echoed back IS the proof of exit-code fidelity.
    if [ "$ec_code" = "$EXITCODE_EXPECT" ]; then
        exitcode_status="PASS"
    else
        exitcode_status="FAIL"; broken="EXEC_EXITCODE"
        exec_detail="${exec_detail}${exec_detail:+ }exitcode_stage: ok=$ec_ok exit_code=$ec_code expected=$EXITCODE_EXPECT"
    fi
fi

# --- STAGE 3: CLEANUP (best-effort, never fatal) ----------------------------
if [ "$TEST_MODE" -eq 0 ] && [ "$spawn_status" = "PASS" ]; then
    "$TERMLINK" signal "$SESSION" TERM "${HUB_ARGS[@]}" >/dev/null 2>&1 || true
    "$TERMLINK" clean >/dev/null 2>&1 || true
    cleanup_status="done"
fi

# --- render -----------------------------------------------------------------
# proven requires SPAWN + EXEC (happy path) + EXEC_EXITCODE (fidelity). The
# exitcode stage runs only when EXEC passed, so requiring PASS here also folds in
# the EXEC precondition — but we assert all three explicitly for clarity.
proven="false"
[ "$spawn_status" = "PASS" ] && [ "$exec_status" = "PASS" ] && [ "$exitcode_status" = "PASS" ] && proven="true"

if [ "$JSON_MODE" -eq 1 ]; then
    jq -cn \
        --arg spawn "$spawn_status" --arg exec "$exec_status" --arg exitcode "$exitcode_status" --arg cleanup "$cleanup_status" \
        --arg session "$SESSION" --arg sentinel "$SENTINEL" \
        --argjson proven "$proven" \
        --arg broken "$broken" \
        '{ok:$proven, proven:$proven,
          broken_stage:(if $broken=="" then null else $broken end),
          stages:{spawn:$spawn, exec:$exec, exec_exitcode:$exitcode, cleanup:$cleanup},
          session:$session, sentinel:$sentinel}'
else
    echo "session-selftest: control-terminal-sessions verb"
    printf '  STAGE 1  SPAWN         %s\n' "$spawn_status"
    printf '  STAGE 2  EXEC          %s%s\n' "$exec_status" "$( [ -n "$exec_detail" ] && echo "  ($exec_detail)" )"
    printf '  STAGE 2b EXEC_EXITCODE %s\n' "$exitcode_status"
    printf '  STAGE 3  CLEANUP       %s\n' "$cleanup_status"
    if [ "$proven" = "true" ]; then
        echo "  → PROVEN: registered a session, exec'd a command (sentinel echoed, not truncated), and a non-zero exit code propagated faithfully."
    else
        echo "  → BROKEN at $broken. The terminal-session verb is not silently failing — the stage is named."
    fi
fi

[ "$proven" = "true" ] && exit 0
exit 1
