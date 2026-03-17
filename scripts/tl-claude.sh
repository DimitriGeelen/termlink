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

die() { echo "ERROR: $1" >&2; exit 1; }

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
  --help            Show this help

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

build_claude_cmd() {
    local cmd="claude"
    for arg in "${CLAUDE_ARGS[@]}"; do
        cmd="$cmd $(printf '%q' "$arg")"
    done
    echo "$cmd"
}

# --- Subcommands ---

cmd_oneshot() {
    echo "Starting Claude Code as TermLink session '$SESSION_NAME' (backend: $BACKEND)"
    echo "Remote access: termlink attach $SESSION_NAME"
    echo ""
    exec termlink spawn \
        --name "$SESSION_NAME" \
        --tags "$TAGS" \
        --backend "$BACKEND" \
        -- claude "${CLAUDE_ARGS[@]}"
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
}

cmd_stop() {
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

# --- Parse ---

# Check for subcommand first
SUBCOMMAND=""
case "${1:-}" in
    start)   SUBCOMMAND="start"; shift ;;
    restart) SUBCOMMAND="restart"; shift ;;
    status)  SUBCOMMAND="status"; shift ;;
    stop)    SUBCOMMAND="stop"; shift ;;
esac

# Parse remaining options
while [[ $# -gt 0 ]]; do
    case $1 in
        --name) SESSION_NAME="$2"; shift 2 ;;
        --backend) BACKEND="$2"; shift 2 ;;
        --tags) TAGS="$2"; shift 2 ;;
        --help|-h) show_help; exit 0 ;;
        --) shift; CLAUDE_ARGS=("$@"); break ;;
        *) CLAUDE_ARGS=("$@"); break ;;
    esac
done

# Preflight
command -v termlink >/dev/null 2>&1 || die "termlink not found on PATH"
command -v claude >/dev/null 2>&1 || die "claude not found on PATH"

# Dispatch
case "$SUBCOMMAND" in
    start)   cmd_start ;;
    restart) cmd_restart ;;
    status)  cmd_status ;;
    stop)    cmd_stop ;;
    "")      cmd_oneshot ;;
esac
