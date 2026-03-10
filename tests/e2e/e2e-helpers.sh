#!/usr/bin/env bash
# =============================================================================
# Shared helpers for e2e tests
# =============================================================================
# Source this file in e2e tests:
#   source "$(dirname "$0")/e2e-helpers.sh"
#
# Required vars (set before sourcing):
#   RUNTIME_DIR  — temp directory for this test run
#   TERMLINK     — path to termlink binary
#
# Provides:
#   spawn_tracked  — spawn a session and track its Terminal.app window ID
#   cleanup_all    — kill processes, close tracked windows, clean sessions
# =============================================================================

WINDOW_IDS_FILE="${RUNTIME_DIR}/window-ids.txt"
PIDS_FILE="${RUNTIME_DIR}/pids.txt"
touch "$WINDOW_IDS_FILE" "$PIDS_FILE"

# spawn_tracked: wrapper around `termlink spawn` that captures and stores window IDs
# Usage: spawn_tracked [termlink spawn args...]
# Example: spawn_tracked --name reviewer --roles reviewer --wait --wait-timeout 15 -- bash watcher.sh
spawn_tracked() {
    local output
    output=$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" spawn "$@" 2>&1)
    echo "$output"

    # Extract window ID from output like "tab 1 of window id 7340"
    local wid
    wid=$(echo "$output" | sed -n 's/.*window id \([0-9]*\).*/\1/p' | head -1)
    if [ -n "$wid" ]; then
        echo "$wid" >> "$WINDOW_IDS_FILE"
    fi
}

# cleanup_all: kill tracked processes, close tracked Terminal windows, clean sessions
# Call this in a trap: trap cleanup_all EXIT
cleanup_all() {
    echo ""
    echo "=== Cleanup ==="

    # Kill tracked processes
    if [ -f "$PIDS_FILE" ]; then
        while IFS= read -r pid; do
            kill "$pid" 2>/dev/null || true
        done < "$PIDS_FILE"
    fi

    # Kill orchestrator if set
    if [ -n "${ORCH_PID:-}" ]; then
        kill "$ORCH_PID" 2>/dev/null || true
    fi

    # Wait for processes to die before closing windows
    sleep 1

    # Close tracked Terminal.app windows (only ours, no confirmation dialog)
    if [ -f "$WINDOW_IDS_FILE" ] && [ -s "$WINDOW_IDS_FILE" ]; then
        local closed=0
        while IFS= read -r wid; do
            if [ -n "$wid" ]; then
                osascript -e "
                    tell application \"Terminal\"
                        try
                            close window id $wid saving no
                        end try
                    end tell
                " 2>/dev/null || true
                closed=$((closed + 1))
            fi
        done < "$WINDOW_IDS_FILE"
        echo "  Closed $closed Terminal window(s)"
    fi

    # Clean stale sessions
    TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" clean 2>/dev/null || true

    # Remove runtime dir
    rm -rf "$RUNTIME_DIR"
    echo "Done."
}
