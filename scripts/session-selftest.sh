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
#   PTY_SPAWN     (T-2695) a second, `--shell` session registers WITH a real PTY.
#                 STAGE 1's session is spawned `-- sleep <ttl>` and has `pty: null`,
#                 so it cannot serve the two PTY verbs below; the sleep-backed session
#                 is left untouched so the existing stages carry no regression risk.
#   OUTPUT        (T-2695) `termlink output --strip-ansi --json` returns ok + non-empty
#                 content — the charter's "stream output" claim. Ordered BEFORE inject
#                 deliberately: if the observation channel is broken, blaming inject
#                 for an unobservable effect would be a wrong diagnosis.
#   INJECT        (T-2695) the charter's "inject keystrokes" claim, proven BY EFFECT.
#                 Injecting `echo FOO` makes FOO appear twice in the PTY — once as the
#                 terminal's echo of the keystrokes, once as the command's output — so
#                 a naive grep would pass on echo alone, proving the bytes ARRIVED but
#                 not that the shell INTERPRETED them (the same weakness as the
#                 `command_inject_resolves_keys_no_pty` unit tests). The injected text
#                 therefore embeds shell quoting the shell strips: the typed line
#                 contains the quotes, the output line does not, so matching the
#                 UNQUOTED string can only match the interpreted result.
#   CLEANUP       best-effort teardown (signal TERM + clean) for BOTH sessions on every
#                 path including PTY-stage failure; NEVER fatal (a self-reaped session
#                 makes `signal` legitimately fail).
# Unlike doorbell-wake (needs a live peer), `termlink exec --json` makes this
# deterministic and local. This is an on-demand PROVER, NOT a cron canary — the
# affirmative complement to the detectors, and the 4th sibling of comms-selftest.
#
# T-2695 origin: the T-2694 review decomposed the charter's terminal-endpoints noun
# into the four capabilities it actually lists — stream output, inject keystrokes,
# exec, doorbell-wake — and found this prover exercised only `exec`, i.e. 1 of 4,
# while the header above quoted the acknowledged inject gap without closing it.
# Building the INJECT stage then surfaced T-2697: `termlink inject` reported
# `{"ok":true,"bytes_injected":N}` for a no-op on a session with no PTY.
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
#                   stages: spawn, exec, exec_exitcode, pty_spawn, output, inject, cleanup
#     --ttl <secs>  scratch-session lifetime (default 30; it self-reaps as a backstop)
#     --hub <addr>  target hub (default: local hub)
#
# TEST HOOKS (PL-213 — host-independent, no live hub):
#   TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC=<int>        canned spawn exit code
#   TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON=<json>      canned `exec --json` output
#                                                        (set .truncated:true to test F3)
#   TERMLINK_SESSION_SELFTEST_TEST_EXITCODE_JSON=<json>  canned negative-exec `--json`
#                                                        output for the EXEC_EXITCODE stage
#   TERMLINK_SESSION_SELFTEST_TEST_OUTPUT_STATUS=<PASS|FAIL>  canned OUTPUT verdict (T-2695)
#   TERMLINK_SESSION_SELFTEST_TEST_INJECT_STATUS=<PASS|FAIL>  canned INJECT verdict (T-2695)
#     Unset ⇒ the PTY stages report "skipped", which never blocks `proven` — so
#     pre-existing SPAWN/EXEC-focused harness cases stay green without modification.
#   SESSION_SELFTEST_PTY_ATTEMPTS / _PTY_SLEEP           PTY readiness retry budget
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
            sed -n '2,83p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
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

# --- STAGES 3a/3b: OUTPUT + INJECT (T-2695) ---------------------------------
# WHY these exist. The charter's terminal-endpoints noun makes FOUR claims — peers
# can "stream output, inject keystrokes, exec, and doorbell-wake" sessions. Until
# T-2695 this prover exercised exactly one of them (`exec`), while its own header
# quoted the acknowledged gap ("agent-conversation-selftest.sh says: What it does NOT
# validate: PTY inject") without closing it. `inject`'s unit tests are named
# `command_inject_resolves_keys_no_pty` — they prove key RESOLUTION with no PTY
# attached, which is precisely not the claim.
#
# WHY A SECOND SESSION. STAGE 1 spawns `-- sleep $TTL`, which has NO PTY
# (`status` reports `pty: null`); `output` refuses it with -32007 and `inject` can
# never reach a terminal through it. Reuse is structurally impossible, so the PTY
# stages spawn their own `--shell` session. The sleep-backed session is left exactly
# as it was, so the existing stages — and the T-2557 canary that runs them — carry
# zero regression risk from this change.
#
# WHY THE ODD SENTINEL. Injecting `echo FOO` makes FOO appear TWICE in the PTY: once
# as the terminal's echo of the keystrokes, once as the command's output. Grepping
# for FOO would therefore pass on echo alone — proving the bytes ARRIVED but not that
# the shell INTERPRETED them, which is the same weakness as the no_pty unit tests. So
# the injected text embeds shell quoting (`echo INJ'-'OK`) that the shell strips:
# the typed line contains the quotes, the output line does not. Matching the UNQUOTED
# string can only match the interpreted result.
pty_status="skipped"; output_status="skipped"; inject_status="skipped"
PTY_SESSION=""
PTY_ATTEMPTS="${SESSION_SELFTEST_PTY_ATTEMPTS:-10}"
PTY_SLEEP="${SESSION_SELFTEST_PTY_SLEEP:-0.5}"
# Split sentinel: INJ_HEAD'-'INJ_TAIL is typed; INJ_HEAD-INJ_TAIL is what the shell prints.
INJ_HEAD="INJECT-PROVEN"; INJ_TAIL="${NONCE}"
INJ_EXPECT="${INJ_HEAD}-${INJ_TAIL}"

if [ "$TEST_MODE" -eq 1 ]; then
    # Test seams (PL-213): canned verdicts so the canary translation stays verifiable
    # without a live PTY. Absent seams leave the stages "skipped", which never blocks
    # `proven` — pre-existing SPAWN/EXEC-focused cases stay green untouched.
    output_status="${TERMLINK_SESSION_SELFTEST_TEST_OUTPUT_STATUS:-skipped}"
    inject_status="${TERMLINK_SESSION_SELFTEST_TEST_INJECT_STATUS:-skipped}"
    [ "$output_status" = "FAIL" ] && broken="${broken:-OUTPUT}"
    [ "$inject_status" = "FAIL" ] && broken="${broken:-INJECT}"
elif [ "$spawn_status" = "PASS" ]; then
    PTY_SESSION="${SESSION}-pty"
    if "$TERMLINK" spawn --name "$PTY_SESSION" --shell --backend tmux \
        --wait --wait-timeout 10 "${HUB_ARGS[@]}" >/dev/null 2>&1; then
        pty_status="PASS"

        # --- STAGE 3a: OUTPUT -------------------------------------------------
        # Prove the streaming path works at all before blaming inject for a blank
        # buffer. A shell session always renders at least a prompt.
        attempt=0
        while : ; do
            attempt=$((attempt+1))
            out_json="$("$TERMLINK" output "$PTY_SESSION" --strip-ansi --json "${HUB_ARGS[@]}" 2>/dev/null || true)"
            o_ok="$(printf '%s' "$out_json" | jq -r '.ok // false' 2>/dev/null || echo false)"
            o_txt="$(printf '%s' "$out_json" | jq -r '.output // ""' 2>/dev/null || echo '')"
            if [ "$o_ok" = "true" ] && [ -n "$o_txt" ]; then
                output_status="PASS"; break
            fi
            if [ "$attempt" -ge "$PTY_ATTEMPTS" ]; then
                output_status="FAIL"; broken="${broken:-OUTPUT}"; break
            fi
            sleep "$PTY_SLEEP"
        done

        # --- STAGE 3b: INJECT (only if OUTPUT works) --------------------------
        # Ordered deliberately: if `output` is broken we cannot observe inject's
        # effect, so attributing the failure to INJECT would be a wrong diagnosis.
        if [ "$output_status" = "PASS" ]; then
            if "$TERMLINK" inject "$PTY_SESSION" "echo ${INJ_HEAD}'-'${INJ_TAIL}" \
                --enter --json "${HUB_ARGS[@]}" >/dev/null 2>&1; then
                attempt=0
                while : ; do
                    attempt=$((attempt+1))
                    inj_json="$("$TERMLINK" output "$PTY_SESSION" --strip-ansi --json "${HUB_ARGS[@]}" 2>/dev/null || true)"
                    inj_txt="$(printf '%s' "$inj_json" | jq -r '.output // ""' 2>/dev/null || echo '')"
                    if printf '%s' "$inj_txt" | grep -qF "$INJ_EXPECT"; then
                        inject_status="PASS"; break
                    fi
                    if [ "$attempt" -ge "$PTY_ATTEMPTS" ]; then
                        inject_status="FAIL"; broken="${broken:-INJECT}"; break
                    fi
                    sleep "$PTY_SLEEP"
                done
            else
                # T-2697 made a no-PTY inject exit non-zero, so a failure here is a
                # real refusal rather than the silent no-op it used to be.
                inject_status="FAIL"; broken="${broken:-INJECT}"
            fi
        fi
    else
        pty_status="FAIL"; broken="${broken:-PTY_SPAWN}"
    fi
fi

# --- STAGE 4: CLEANUP (best-effort, never fatal) ----------------------------
if [ "$TEST_MODE" -eq 0 ] && [ "$spawn_status" = "PASS" ]; then
    "$TERMLINK" signal "$SESSION" TERM "${HUB_ARGS[@]}" >/dev/null 2>&1 || true
    # Reap the PTY session on EVERY path including PTY-stage failure — a leaked tmux
    # session per canary run would be a worse defect than the gap being closed.
    if [ -n "$PTY_SESSION" ]; then
        "$TERMLINK" signal "$PTY_SESSION" TERM "${HUB_ARGS[@]}" >/dev/null 2>&1 || true
        # T-2780: `signal TERM` + `clean` are NOT sufficient, and this is the trap —
        # both return rc=0 (reporting success) while the backing tmux session keeps
        # running. `signal` targets the session's registered PROCESS; the tmux session
        # that process created outlives it, and `clean` reaps termlink's registry, not
        # tmux. Measured: 7 orphaned `tl-session-selftest-*-pty` sessions after 7 runs,
        # with 0 showing in `termlink list` — invisible through every termlink surface.
        # The T-2557 canary runs this prover daily, so that is one orphan per day
        # forever. T-2695 AC 5 asserted this was handled; it was not.
        # Scoped to our OWN generated name (nonce-suffixed), never a prefix/glob, so it
        # can never touch a `tl-*` session belonging to real work. Best-effort: tmux may
        # legitimately be absent, and a reap failure must never fail a passing prover.
        if command -v tmux >/dev/null 2>&1; then
            tmux kill-session -t "tl-${PTY_SESSION}" >/dev/null 2>&1 || true
        fi
    fi
    "$TERMLINK" clean >/dev/null 2>&1 || true
    cleanup_status="done"
fi

# --- render -----------------------------------------------------------------
# proven requires SPAWN + EXEC (happy path) + EXEC_EXITCODE (fidelity). The
# exitcode stage runs only when EXEC passed, so requiring PASS here also folds in
# the EXEC precondition — but we assert all three explicitly for clarity.
# T-2695: a PTY stage that ran and FAILED blocks `proven` — that is the whole point
# of adding them. A stage that was SKIPPED (test-hook mode without the seam set) does
# not, so pre-existing SPAWN/EXEC-focused cases stay green without modification.
pty_ok="true"
[ "$output_status" = "FAIL" ] && pty_ok="false"
[ "$inject_status" = "FAIL" ] && pty_ok="false"
[ "$pty_status" = "FAIL" ] && pty_ok="false"

proven="false"
[ "$spawn_status" = "PASS" ] && [ "$exec_status" = "PASS" ] && [ "$exitcode_status" = "PASS" ] \
    && [ "$pty_ok" = "true" ] && proven="true"

if [ "$JSON_MODE" -eq 1 ]; then
    jq -cn \
        --arg spawn "$spawn_status" --arg exec "$exec_status" --arg exitcode "$exitcode_status" --arg cleanup "$cleanup_status" \
        --arg pty "$pty_status" --arg output "$output_status" --arg inject "$inject_status" \
        --arg session "$SESSION" --arg sentinel "$SENTINEL" \
        --argjson proven "$proven" \
        --arg broken "$broken" \
        '{ok:$proven, proven:$proven,
          broken_stage:(if $broken=="" then null else $broken end),
          stages:{spawn:$spawn, exec:$exec, exec_exitcode:$exitcode,
                  pty_spawn:$pty, output:$output, inject:$inject, cleanup:$cleanup},
          session:$session, sentinel:$sentinel}'
else
    echo "session-selftest: control-terminal-sessions verb"
    printf '  STAGE 1  SPAWN         %s\n' "$spawn_status"
    printf '  STAGE 2  EXEC          %s%s\n' "$exec_status" "$( [ -n "$exec_detail" ] && echo "  ($exec_detail)" )"
    printf '  STAGE 2b EXEC_EXITCODE %s\n' "$exitcode_status"
    printf '  STAGE 3   PTY_SPAWN    %s\n' "$pty_status"
    printf '  STAGE 3a  OUTPUT       %s\n' "$output_status"
    printf '  STAGE 3b  INJECT       %s\n' "$inject_status"
    printf '  STAGE 4   CLEANUP      %s\n' "$cleanup_status"
    if [ "$proven" = "true" ]; then
        echo "  → PROVEN: registered a session, exec'd a command (sentinel echoed, not truncated),"
        echo "    a non-zero exit code propagated faithfully, PTY output streamed back, and injected"
        echo "    keystrokes were INTERPRETED by the shell (not merely echoed)."
    else
        echo "  → BROKEN at $broken. The terminal-session verb is not silently failing — the stage is named."
    fi
fi

[ "$proven" = "true" ] && exit 0
exit 1
