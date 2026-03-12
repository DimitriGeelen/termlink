#!/usr/bin/env bash
# =============================================================================
# Shared E2E Setup — portable binary resolution, build, orchestrator helpers
# =============================================================================
# Source this file at the top of e2e tests:
#   source "$(dirname "$0")/setup.sh"
#
# Provides:
#   SCRIPT_DIR, PROJECT_ROOT, TERMLINK, CLAUDE, RUNTIME_DIR (all exported)
#   build_termlink  — build the termlink binary
#   tl              — shorthand for TERMLINK_RUNTIME_DIR=$RUNTIME_DIR $TERMLINK
#   register_orchestrator — register + health-check orchestrator session
#
# Also sources e2e-helpers.sh and sets trap cleanup_all EXIT.
#
# Override paths via env vars:
#   CARGO_BIN      — path to cargo (default: $HOME/.cargo/bin/cargo or command -v cargo)
#   CLAUDE_BIN     — path to claude (default: command -v claude)
#   TERMLINK_BIN   — path to termlink (default: $PROJECT_ROOT/target/debug/termlink)
# =============================================================================

# --- Path resolution ---
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Resolve cargo
if [ -n "${CARGO_BIN:-}" ]; then
    CARGO="$CARGO_BIN"
elif command -v cargo >/dev/null 2>&1; then
    CARGO="$(command -v cargo)"
elif [ -x "$HOME/.cargo/bin/cargo" ]; then
    CARGO="$HOME/.cargo/bin/cargo"
else
    echo "ERROR: cargo not found. Set CARGO_BIN or install Rust." >&2
    exit 1
fi

# Resolve claude
if [ -n "${CLAUDE_BIN:-}" ]; then
    CLAUDE="$CLAUDE_BIN"
elif command -v claude >/dev/null 2>&1; then
    CLAUDE="$(command -v claude)"
elif [ -x "$HOME/.local/bin/claude" ]; then
    CLAUDE="$HOME/.local/bin/claude"
else
    echo "WARNING: claude not found. Set CLAUDE_BIN for tests that need it." >&2
    CLAUDE=""
fi

# Resolve termlink
if [ -n "${TERMLINK_BIN:-}" ]; then
    TERMLINK="$TERMLINK_BIN"
else
    TERMLINK="$PROJECT_ROOT/target/debug/termlink"
fi

# Runtime dir (per-test isolation)
RUNTIME_DIR=$(mktemp -d)

export SCRIPT_DIR PROJECT_ROOT TERMLINK CLAUDE CARGO RUNTIME_DIR

# --- Source helpers and set trap ---
source "$SCRIPT_DIR/e2e-helpers.sh"
trap cleanup_all EXIT

# --- Helper functions ---

build_termlink() {
    echo "--- Build ---"
    (cd "$PROJECT_ROOT" && "$CARGO" build -p termlink 2>&1 | tail -1)
    echo ""
}

tl() {
    TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" "$@"
}

register_orchestrator() {
    local name="${1:-orchestrator}"
    local roles="${2:-orchestrator}"
    echo "--- Register $name ---"
    TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" register \
        --name "$name" --roles "$roles" &
    ORCH_PID=$!

    for i in $(seq 1 10); do
        if ls "$RUNTIME_DIR/sessions/"*.sock >/dev/null 2>&1; then break; fi
        sleep 1
    done

    if TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" ping "$name" 2>/dev/null; then
        echo "$name OK"
    else
        echo "FAIL: $name not registered"; exit 1
    fi
    echo ""
}
