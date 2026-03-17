#!/bin/bash
# tl-claude.sh — Launch Claude Code inside a TermLink session
#
# Makes the Claude Code session discoverable, observable, and remotely
# controllable via TermLink primitives (attach, stream, pty inject).
#
# Usage:
#   tl-claude.sh [--name NAME] [--backend auto|terminal|tmux|background] [-- claude args...]
#   tl-claude.sh --help
#
# Examples:
#   tl-claude.sh                              # Launch with defaults (name: claude-master)
#   tl-claude.sh --name my-session            # Custom session name
#   tl-claude.sh --backend tmux               # Force tmux backend
#   tl-claude.sh -- --resume                  # Pass --resume to claude
#   tl-claude.sh -- -p "hello"                # Run claude in print mode
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

Usage:
  tl-claude.sh [OPTIONS] [-- CLAUDE_ARGS...]

Options:
  --name NAME       Session name (default: claude-master)
  --backend TYPE    Spawn backend: auto, terminal, tmux, background
                    (default: auto, or TL_CLAUDE_BACKEND env var)
  --tags TAGS       Comma-separated tags (default: master,claude)
  --help            Show this help

Everything after -- is passed to claude:
  tl-claude.sh -- --resume           # Resume last session
  tl-claude.sh -- -p "prompt"        # Print mode

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

# Parse args
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

echo "Starting Claude Code as TermLink session '$SESSION_NAME' (backend: $BACKEND)"
echo "Remote access: termlink attach $SESSION_NAME"
echo ""

# Launch Claude Code inside a TermLink-managed PTY session.
# termlink spawn handles: PTY allocation, session registration, cleanup on exit.
exec termlink spawn \
    --name "$SESSION_NAME" \
    --tags "$TAGS" \
    --backend "$BACKEND" \
    -- claude "${CLAUDE_ARGS[@]}"
