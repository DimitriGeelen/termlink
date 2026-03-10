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

    # Kill orchestrator if set
    if [ -n "${ORCH_PID:-}" ]; then
        kill "$ORCH_PID" 2>/dev/null || true
    fi

    # Close tracked Terminal.app windows (only ours)
    # Note: `close window id N` silently fails in Terminal.app. Must iterate windows
    # and close by reference. Kill child processes first so windows aren't busy.
    if [ -f "$WINDOW_IDS_FILE" ] && [ -s "$WINDOW_IDS_FILE" ]; then
        # Phase 1: for each tracked window, get TTY and kill child processes
        while IFS= read -r wid; do
            if [ -n "$wid" ]; then
                local tty_name
                tty_name=$(osascript -e "
                    tell application \"Terminal\"
                        try
                            return tty of tab 1 of window id $wid
                        end try
                    end tell
                " 2>/dev/null || true)
                if [ -n "$tty_name" ]; then
                    local tty_short="${tty_name#/dev/}"
                    # Kill child processes but spare login and login shell (-zsh/-bash)
                    local pids
                    pids=$(ps -t "$tty_short" -o pid=,comm= 2>/dev/null \
                        | grep -v -E '(login|-zsh|-bash)' \
                        | awk '{print $1}' || true)
                    if [ -n "$pids" ]; then
                        echo "$pids" | xargs kill -9 2>/dev/null || true
                    fi
                fi
            fi
        done < "$WINDOW_IDS_FILE"

        sleep 2

        # Phase 2: send "exit" to each window — shell exits cleanly, window auto-closes
        while IFS= read -r wid; do
            if [ -n "$wid" ]; then
                osascript -e "
                    tell application \"Terminal\"
                        try
                            do script \"exit\" in window id $wid
                        end try
                    end tell
                " 2>/dev/null || true
            fi
        done < "$WINDOW_IDS_FILE"

        sleep 2

        # Phase 3: close any remaining windows by reference (fallback)
        local id_list=""
        while IFS= read -r wid; do
            [ -n "$wid" ] && id_list="${id_list:+$id_list, }$wid"
        done < "$WINDOW_IDS_FILE"

        if [ -n "$id_list" ]; then
            osascript -e "
                tell application \"Terminal\"
                    set targetIds to {$id_list}
                    repeat with w in (reverse of (windows as list))
                        try
                            if (id of w) is in targetIds then
                                close w
                            end if
                        end try
                    end repeat
                end tell
            " 2>/dev/null || true
        fi
        echo "  Closed tracked Terminal window(s)"
    fi

    # Clean stale sessions
    TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" clean 2>/dev/null || true

    # Remove runtime dir
    rm -rf "$RUNTIME_DIR"
    echo "Done."
}
