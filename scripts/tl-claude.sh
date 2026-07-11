#!/bin/bash
# tl-claude.sh — Launch Claude Code inside a TermLink session
#
# Makes the Claude Code session discoverable, observable, and remotely
# controllable via TermLink primitives (attach, stream, pty inject).
#
# Supports two modes:
#   - Default: Claude runs inside a TermLink session (session dies with claude)
#   - Persistent: Shell session stays alive across claude restarts
#
# Usage:
#   tl-claude.sh [OPTIONS] [-- CLAUDE_ARGS...]     # Default mode (one-shot)
#   tl-claude.sh start [OPTIONS] [-- CLAUDE_ARGS...]  # Persistent mode
#   tl-claude.sh restart [--name NAME] [-- CLAUDE_ARGS...]  # Re-inject claude
#   tl-claude.sh status [--name NAME]               # Show session state
#   tl-claude.sh stop [--name NAME]                 # Stop persistent session
#   tl-claude.sh --help
#
# Remote access (from another terminal or machine):
#   termlink list                             # See the session
#   termlink attach claude-master             # Mirror the TUI (bidirectional)
#   termlink stream claude-master             # Low-latency binary stream
#   termlink pty output claude-master --strip-ansi  # Read recent output as text
#   termlink pty inject claude-master "text" --enter  # Send input

set -e

SESSION_NAME="claude-master"
BACKEND="${TL_CLAUDE_BACKEND:-auto}"
TAGS="master,claude"
CLAUDE_ARGS=()
# T-2388 (T-2380 C7+C5): --reachable arms be-reachable (heartbeat + push-waker)
# against the spawned PTY, closing the arc-004 dormancy gap (PL-237: the waker
# can only ring termlink-owned PTYs; this launcher creates exactly those).
REACHABLE=0
AGENT_ID=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

die() { echo "ERROR: $1" >&2; exit 1; }

# T-2388: per-agent be-reachable state file so multiple armed agents on one
# host don't clobber the singleton ~/.termlink/be-reachable.state.
reachable_agent_id() { echo "${AGENT_ID:-$SESSION_NAME}"; }
reachable_state_file() { echo "${HOME}/.termlink/be-reachable-$(reachable_agent_id).state"; }

# Arm be-reachable for the spawned session. Loud on failure but NEVER kills the
# launched session (the durable poll floor still works un-armed).
arm_reachable() {
    local agent_id state br
    agent_id="$(reachable_agent_id)"
    state="$(reachable_state_file)"
    br="$SCRIPT_DIR/be-reachable.sh"
    if [ ! -x "$br" ] && [ ! -f "$br" ]; then
        echo "WARN: be-reachable.sh not found at $br — session is up but NOT push-reachable." >&2
        return 1
    fi
    if BE_REACHABLE_STATE="$state" bash "$br" start \
        --agent-id "$agent_id" --pty-session "$SESSION_NAME"; then
        echo "reachable: armed (agent-id=$agent_id pty=$SESSION_NAME state=$state)"
    else
        echo "WARN: be-reachable arm FAILED — session '$SESSION_NAME' is up but NOT push-reachable." >&2
        echo "  Retry manually: BE_REACHABLE_STATE=$state bash $br start --agent-id $agent_id --pty-session $SESSION_NAME" >&2
        return 1
    fi
}

# One-shot mode execs `termlink spawn`, so arming must happen out-of-band:
# detach a retry loop that waits for the session to register, then arms.
arm_reachable_async() {
    local agent_id state br log
    agent_id="$(reachable_agent_id)"
    state="$(reachable_state_file)"
    br="$SCRIPT_DIR/be-reachable.sh"
    log="${HOME}/.termlink/tl-claude-arm-${agent_id}.log"
    echo "reachable: arming in background once session '$SESSION_NAME' registers (log: $log)"
    nohup setsid bash -c '
        for i in $(seq 1 15); do
            sleep 2
            if termlink list 2>/dev/null | grep -q "'"$SESSION_NAME"'"; then
                BE_REACHABLE_STATE="'"$state"'" bash "'"$br"'" start \
                    --agent-id "'"$agent_id"'" --pty-session "'"$SESSION_NAME"'" && exit 0
                exit 1
            fi
        done
        echo "arm: session never registered — NOT armed" >&2
        exit 1
    ' >"$log" 2>&1 &
    disown 2>/dev/null || true
}

show_help() {
    cat <<'HELP'
tl-claude.sh — Launch Claude Code inside a TermLink session

Subcommands:
  (none)            One-shot mode — session dies when claude exits
  start             Persistent mode — shell session survives claude exit
  restart           Re-inject claude into existing persistent session
  status            Show TermLink session state
  stop              Kill the persistent session

Options:
  --name NAME       Session name (default: claude-master)
  --backend TYPE    Spawn backend: auto, terminal, tmux, background
                    (default: auto, or TL_CLAUDE_BACKEND env var)
  --tags TAGS       Comma-separated tags (default: master,claude)
  --reachable       T-2388 (arc-004 C7): arm be-reachable against the spawned
                    PTY — heartbeat + push-waker (inbox + dm rails), so a DM
                    push-wakes this agent sub-second instead of the poll floor.
                    Per-agent state: ~/.termlink/be-reachable-<agent-id>.state
  --agent-id ID     Agent identity for --reachable (default: session name)
  --help            Show this help

Subcommand install-boot (T-2388, arc-004 C5):
  tl-claude.sh install-boot --name N --agent-id ID [-- CLAUDE_ARGS]
                    Write /etc/cron.d/termlink-agent-<ID> (@reboot) so the
                    armed agent re-launches after a reboot — wakers survive
                    without human memory.

THE one-liner that makes an agent push-reachable (no tmux needed):
  bash scripts/tl-claude.sh start --reachable --agent-id <name> -- --resume

Everything after -- is passed to claude:
  tl-claude.sh -- --resume           # Resume last session
  tl-claude.sh start -- -p "prompt"  # Persistent + print mode

Environment:
  TL_CLAUDE_BACKEND   Override default backend (same as --backend)

Remote access (from another terminal):
  termlink list                              # Discover the session
  termlink attach <name>                     # Full TUI mirror (bidirectional)
  termlink stream <name>                     # Low-latency binary stream
  termlink pty output <name> --strip-ansi    # Read recent output as text
  termlink pty inject <name> "text" --enter  # Send input
HELP
}

session_exists() {
    termlink list 2>/dev/null | grep -q "$SESSION_NAME"
}

# T-2400: a --reachable agent must launch with auto-accept so that when a peer's
# doorbell wakes it, it can post its reply through channel_post WITHOUT a human
# at the PTY to approve the "Do you want to proceed?" prompt. Without this a
# reachable agent is discoverable + wakeable but MUTE (the comms loop dies after
# one hop). IS_SANDBOX=1 + --dangerously-skip-permissions is the root
# auto-accept combo (see reference_is_sandbox_root_bypass). No-op when not
# reachable; opt out with TL_NO_AUTO_ACCEPT=1; idempotent if caller already
# passed the flag.
build_claude_cmd() {
    local cmd="${TL_CLAUDE_CMD:-claude}"
    local env_prefix="" has_skip=0 arg
    # T-2403: sanitize a leaked PROJECT_ROOT so a cwd-scoped agent resolves its
    # OWN project. A stale value inherited from the launcher env (observed:
    # /opt/023 on workflow-designer while PWD=/opt/832) misroutes framework
    # project resolution and gates ALL of the agent's fw/Bash/Edit in the wrong
    # project. `env -u PROJECT_ROOT` drops it so the framework falls back to the
    # cwd's .framework.yaml / git-toplevel (the normal unset path). Unconditional
    # (independent of REACHABLE); opt out with TL_KEEP_PROJECT_ROOT=1.
    [ "${TL_KEEP_PROJECT_ROOT:-0}" != "1" ] && env_prefix="env -u PROJECT_ROOT "
    for arg in "${CLAUDE_ARGS[@]}"; do
        [ "$arg" = "--dangerously-skip-permissions" ] && has_skip=1
    done
    if [ "$REACHABLE" -eq 1 ] && [ "${TL_NO_AUTO_ACCEPT:-0}" != "1" ] && [ "$has_skip" -eq 0 ]; then
        # Compose with the sanitizer: `env -u PROJECT_ROOT IS_SANDBOX=1 claude …`
        # (env accepts -u NAME then VAR=val assignments then the command).
        env_prefix="${env_prefix}IS_SANDBOX=1 "
        CLAUDE_ARGS+=("--dangerously-skip-permissions")
    fi
    for arg in "${CLAUDE_ARGS[@]}"; do
        cmd="$cmd $(printf '%q' "$arg")"
    done
    echo "${env_prefix}${cmd}"
}

# --- Subcommands ---

cmd_oneshot() {
    echo "Starting Claude Code as TermLink session '$SESSION_NAME' (backend: $BACKEND)"
    echo "Remote access: termlink attach $SESSION_NAME"
    echo ""
    # T-2388: exec replaces this process, so arm out-of-band via retry loop.
    [ "$REACHABLE" -eq 1 ] && arm_reachable_async
    # T-2403: sanitize a leaked PROJECT_ROOT before the exec so the spawned
    # register/claude resolves its OWN project (see build_claude_cmd note).
    # This path uses exec (inherited env), so unset in-shell rather than prefix.
    [ "${TL_KEEP_PROJECT_ROOT:-0}" != "1" ] && unset PROJECT_ROOT
    # T-2400: same auto-accept guarantee as build_claude_cmd, but this path uses
    # exec (not PTY-string injection), so export IS_SANDBOX into the inherited
    # env and append the flag to the argv rather than prefixing a shell string.
    if [ "$REACHABLE" -eq 1 ] && [ "${TL_NO_AUTO_ACCEPT:-0}" != "1" ]; then
        local has_skip=0 arg
        for arg in "${CLAUDE_ARGS[@]}"; do
            [ "$arg" = "--dangerously-skip-permissions" ] && has_skip=1
        done
        if [ "$has_skip" -eq 0 ]; then
            export IS_SANDBOX=1
            CLAUDE_ARGS+=("--dangerously-skip-permissions")
        fi
    fi
    exec termlink spawn \
        --name "$SESSION_NAME" \
        --tags "$TAGS" \
        --backend "$BACKEND" \
        -- ${TL_CLAUDE_CMD:-claude} "${CLAUDE_ARGS[@]}"
}

cmd_start() {
    if session_exists; then
        echo "Session '$SESSION_NAME' already exists. Use 'restart' to re-inject claude."
        echo "Or 'stop' first to kill the existing session."
        exit 1
    fi

    echo "Starting persistent TermLink session '$SESSION_NAME' (backend: $BACKEND)"
    echo "Session survives claude exit. Use 'restart' to re-inject."
    echo "Remote access: termlink attach $SESSION_NAME"
    echo ""

    # Spawn a persistent shell session (stays alive after claude exits)
    termlink spawn \
        --name "$SESSION_NAME" \
        --tags "$TAGS" \
        --backend "$BACKEND" \
        --shell \
        --wait \
        --wait-timeout 15 \
        || die "Failed to spawn persistent session"

    # Inject claude command into the session
    sleep 1
    local claude_cmd
    claude_cmd=$(build_claude_cmd)
    termlink pty inject "$SESSION_NAME" "$claude_cmd" --enter >/dev/null 2>&1 \
        || die "Failed to inject claude command"

    echo "Claude injected into session '$SESSION_NAME'"

    # T-2388 (C7): arm push-reachability against the freshly spawned PTY.
    # Non-fatal — an un-armed session still works on the poll floor.
    if [ "$REACHABLE" -eq 1 ]; then
        arm_reachable || true
    fi
}

cmd_restart() {
    if ! session_exists; then
        die "Session '$SESSION_NAME' not found. Use 'start' to create one."
    fi

    local claude_cmd
    claude_cmd=$(build_claude_cmd)

    echo "Re-injecting claude into session '$SESSION_NAME'..."
    termlink pty inject "$SESSION_NAME" "$claude_cmd" --enter >/dev/null 2>&1 \
        || die "Failed to inject claude command"

    echo "Claude restarted in session '$SESSION_NAME'"
    echo "Attach: termlink attach $SESSION_NAME"
}

cmd_status() {
    if session_exists; then
        echo "Session '$SESSION_NAME' is active"
        termlink status "$SESSION_NAME" 2>/dev/null || true
        echo ""
        echo "Recent output (last 5 lines):"
        termlink pty output "$SESSION_NAME" --lines 5 --strip-ansi 2>/dev/null || echo "(no output)"
    else
        echo "Session '$SESSION_NAME' not found"
        exit 1
    fi
    # T-2388: show the paired reachability state (armed + waker vs dormant).
    local state
    state="$(reachable_state_file)"
    echo ""
    if [ -f "$state" ]; then
        echo "Reachability (state: $state):"
        BE_REACHABLE_STATE="$state" bash "$SCRIPT_DIR/be-reachable.sh" status 2>/dev/null \
            || echo "  (be-reachable status unavailable)"
    else
        echo "Reachability: NOT armed (no state at $state) — launch with --reachable"
    fi
}

cmd_stop() {
    # T-2388: stop the paired be-reachable FIRST so no orphan heartbeat/waker
    # outlives the session (heartbeat is nohup-setsid-detached by design).
    local state
    state="$(reachable_state_file)"
    if [ -f "$state" ]; then
        echo "Stopping paired be-reachable (state: $state)..."
        BE_REACHABLE_STATE="$state" bash "$SCRIPT_DIR/be-reachable.sh" stop 2>/dev/null || true
    fi

    if ! session_exists; then
        echo "Session '$SESSION_NAME' not found (already stopped?)"
        return 0
    fi

    # Send exit to shell, then clean up
    termlink pty inject "$SESSION_NAME" "exit" --enter >/dev/null 2>&1 || true
    sleep 2
    termlink clean 2>/dev/null || true
    echo "Session '$SESSION_NAME' stopped"
}

# T-2388 (C5): write an @reboot cron so the armed agent survives reboots.
# /etc/cron.d USER-field syntax (same convention as the canary crontabs). The
# 45s sleep lets the hub come up first; the arm retries inside be-reachable
# are the second line of defense.
cmd_install_boot() {
    local agent_id cron_file cron_user launch_args
    agent_id="$(reachable_agent_id)"
    cron_file="/etc/cron.d/termlink-agent-${agent_id}"
    cron_user="$(id -un)"
    launch_args="start --reachable --name $(printf '%q' "$SESSION_NAME") --agent-id $(printf '%q' "$agent_id") --backend background"
    if [ ${#CLAUDE_ARGS[@]} -gt 0 ]; then
        local a; launch_args="$launch_args --"
        for a in "${CLAUDE_ARGS[@]}"; do launch_args="$launch_args $(printf '%q' "$a")"; done
    fi
    # T-2389: resume from the agent's OWN project dir, not the termlink repo.
    # `start` spawns the shell (and thus claude) in $PWD, and claude keys
    # --continue/--resume on cwd — so a project agent MUST boot back into its
    # project dir. Capture $PWD at install time and invoke tl-claude.sh by its
    # absolute path (the earlier `cd $(dirname SCRIPT_DIR) && bash scripts/…`
    # hardcoded /opt/termlink and would resume every agent in the wrong cwd).
    local line="@reboot ${cron_user} sleep 45 && cd $(printf '%q' "$PWD") && bash $(printf '%q' "${SCRIPT_DIR}/tl-claude.sh") ${launch_args} >> ${HOME}/.termlink/tl-claude-boot-${agent_id}.log 2>&1"
    local content="# T-2388 (T-2380 C5): re-arm push-reachable agent '${agent_id}' after reboot.
# Managed by scripts/tl-claude.sh install-boot — edit/remove via that verb.
SHELL=/bin/bash
PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:${HOME}/.cargo/bin
${line}"
    if printf '%s\n' "$content" > "$cron_file" 2>/dev/null; then
        chmod 644 "$cron_file"
        echo "Boot re-arm installed: $cron_file"
        echo "  $line"
    else
        echo "Cannot write $cron_file (need root). Install manually:" >&2
        echo "----------------------------------------" >&2
        printf '%s\n' "$content" >&2
        echo "----------------------------------------" >&2
        exit 1
    fi
}

# --- Parse ---

# Check for subcommand first
SUBCOMMAND=""
case "${1:-}" in
    start)   SUBCOMMAND="start"; shift ;;
    restart) SUBCOMMAND="restart"; shift ;;
    status)  SUBCOMMAND="status"; shift ;;
    stop)    SUBCOMMAND="stop"; shift ;;
    install-boot) SUBCOMMAND="install-boot"; shift ;;
esac

# Parse remaining options
while [[ $# -gt 0 ]]; do
    case $1 in
        --name) SESSION_NAME="$2"; shift 2 ;;
        --backend) BACKEND="$2"; shift 2 ;;
        --tags) TAGS="$2"; shift 2 ;;
        --reachable) REACHABLE=1; shift ;;          # T-2388 (C7)
        --agent-id) AGENT_ID="$2"; shift 2 ;;       # T-2388 (C7)
        --help|-h) show_help; exit 0 ;;
        --) shift; CLAUDE_ARGS=("$@"); break ;;
        *) CLAUDE_ARGS=("$@"); break ;;
    esac
done

# T-2403: lib mode — sourcing with TL_CLAUDE_LIB=1 exposes the pure helpers
# (build_claude_cmd, …) WITHOUT running preflight/dispatch, so unit tests can
# exercise the command-builder in isolation.
if [ "${TL_CLAUDE_LIB:-0}" != "1" ]; then
    # Preflight
    command -v termlink >/dev/null 2>&1 || die "termlink not found on PATH"
    if [ "$SUBCOMMAND" != "stop" ] && [ "$SUBCOMMAND" != "status" ] && [ "$SUBCOMMAND" != "install-boot" ]; then
        command -v "${TL_CLAUDE_CMD:-claude}" >/dev/null 2>&1 || die "${TL_CLAUDE_CMD:-claude} not found on PATH"
    fi

    # Dispatch
    case "$SUBCOMMAND" in
        start)   cmd_start ;;
        restart) cmd_restart ;;
        status)  cmd_status ;;
        stop)    cmd_stop ;;
        install-boot) cmd_install_boot ;;
        "")      cmd_oneshot ;;
    esac
fi
