#!/bin/bash
# tl-dispatch.sh — Spawn claude workers in real terminals via TermLink
#
# Usage:
#   tl-dispatch.sh --name worker-1 --prompt "Explore the codebase" [--project /path] [--timeout 300]
#   tl-dispatch.sh --name worker-1 --prompt-file /tmp/prompt.md [--project /path] [--timeout 300]
#   tl-dispatch.sh status                    # List active workers
#   tl-dispatch.sh wait --name worker-1      # Block until worker completes
#   tl-dispatch.sh wait --all                # Block until all workers complete
#   tl-dispatch.sh result --name worker-1    # Read worker's result
#   tl-dispatch.sh cleanup                   # Kill all workers, clean up
#
# Each worker runs in its own terminal with its own claude instance (full 200K context).
# Results are written to /tmp/tl-dispatch/<worker-name>/result.md
# Completion is signaled via TermLink events (topic: worker.done)

set -e

DISPATCH_DIR="/tmp/tl-dispatch"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# --- Helpers ---

die() { echo "ERROR: $1" >&2; exit 1; }

ensure_termlink() {
    command -v termlink >/dev/null 2>&1 || die "termlink not found on PATH"
}

ensure_claude() {
    command -v claude >/dev/null 2>&1 || die "claude CLI not found on PATH"
}

worker_dir() {
    echo "$DISPATCH_DIR/$1"
}

# --- Commands ---

cmd_spawn() {
    local name="" prompt="" prompt_file="" project_dir="" timeout=600

    while [[ $# -gt 0 ]]; do
        case $1 in
            --name) name="$2"; shift 2 ;;
            --prompt) prompt="$2"; shift 2 ;;
            --prompt-file) prompt_file="$2"; shift 2 ;;
            --project) project_dir="$2"; shift 2 ;;
            --timeout) timeout="$2"; shift 2 ;;
            *) die "Unknown option: $1" ;;
        esac
    done

    [ -z "$name" ] && die "Missing --name"
    [ -z "$prompt" ] && [ -z "$prompt_file" ] && die "Missing --prompt or --prompt-file"

    # Resolve prompt
    if [ -n "$prompt_file" ]; then
        [ -f "$prompt_file" ] || die "Prompt file not found: $prompt_file"
        prompt=$(cat "$prompt_file")
    fi

    # Default project to current directory
    project_dir="${project_dir:-$(pwd)}"

    # Create worker directory
    local wdir
    wdir=$(worker_dir "$name")
    mkdir -p "$wdir"

    # Write prompt to file (avoids shell escaping issues)
    echo "$prompt" > "$wdir/prompt.md"

    # Record metadata
    cat > "$wdir/meta.json" <<METAEOF
{
  "name": "$name",
  "project": "$project_dir",
  "timeout": $timeout,
  "started": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "status": "running"
}
METAEOF

    # Write the worker script that runs inside the terminal
    cat > "$wdir/run.sh" <<'RUNEOF'
#!/bin/bash
WORKER_NAME="$1"
PROJECT_DIR="$2"
WDIR="$3"
TIMEOUT="$4"

cd "$PROJECT_DIR"

# Run claude with the prompt, capture output
# Use background process + kill for timeout (macOS has no `timeout` command)
claude -p "$(cat "$WDIR/prompt.md")" --output-format text > "$WDIR/result.md" 2>"$WDIR/stderr.log" &
CLAUDE_PID=$!

# Watchdog: kill claude if it exceeds timeout
(sleep "$TIMEOUT" && kill "$CLAUDE_PID" 2>/dev/null && echo "TIMEOUT" > "$WDIR/stderr.log") &
WATCHDOG_PID=$!

# Wait for claude to finish
wait "$CLAUDE_PID" 2>/dev/null
EXIT_CODE=$?

# Kill watchdog if claude finished before timeout
kill "$WATCHDOG_PID" 2>/dev/null || true

# Record completion
echo "$EXIT_CODE" > "$WDIR/exit_code"
date -u +%Y-%m-%dT%H:%M:%SZ > "$WDIR/finished_at"

# Signal completion via TermLink event
termlink event emit "$WORKER_NAME" worker.done \
    -p "{\"exit_code\":$EXIT_CODE,\"result\":\"$WDIR/result.md\"}" 2>/dev/null || true

echo ""
echo "=== Worker $WORKER_NAME finished (exit: $EXIT_CODE) ==="
echo "Result: $WDIR/result.md"
RUNEOF
    chmod +x "$wdir/run.sh"

    # Spawn terminal session and track window ID for cleanup
    local spawn_output
    spawn_output=$(osascript -e "tell application \"Terminal\" to do script \"termlink register --name $name --shell\"" 2>&1)

    # Extract and store window ID (output is like "tab 1 of window id 7340")
    local wid
    wid=$(echo "$spawn_output" | sed -n 's/.*window id \([0-9]*\).*/\1/p' | head -1)
    if [ -n "$wid" ]; then
        echo "$wid" > "$wdir/window_id"
    fi

    # Wait for session to appear
    local found=false
    for i in $(seq 1 15); do
        if termlink list 2>/dev/null | grep -q "$name"; then
            found=true
            break
        fi
        sleep 1
    done
    [ "$found" = true ] || die "Session $name did not register within 15s"

    # Let shell settle, then inject the worker script (fire-and-forget, don't wait)
    sleep 1
    termlink pty inject "$name" "bash $wdir/run.sh '$name' '$project_dir' '$wdir' '$timeout'" --enter >/dev/null 2>&1

    echo "Worker spawned: $name"
    echo "  Project: $project_dir"
    echo "  Result:  $wdir/result.md"
    echo "  Timeout: ${timeout}s"
}

cmd_status() {
    ensure_termlink
    echo "=== Active Workers ==="
    echo ""

    if [ ! -d "$DISPATCH_DIR" ]; then
        echo "No workers dispatched."
        return
    fi

    local count=0
    for wdir in "$DISPATCH_DIR"/*/; do
        [ -d "$wdir" ] || continue
        local name
        name=$(basename "$wdir")
        local status="running"

        if [ -f "$wdir/exit_code" ]; then
            local ec
            ec=$(cat "$wdir/exit_code")
            if [ "$ec" = "0" ]; then
                status="complete"
            else
                status="failed (exit: $ec)"
            fi
        fi

        local started=""
        if [ -f "$wdir/meta.json" ]; then
            started=$(python3 -c "import json; print(json.load(open('$wdir/meta.json'))['started'])" 2>/dev/null || echo "unknown")
        fi

        local finished=""
        if [ -f "$wdir/finished_at" ]; then
            finished=$(cat "$wdir/finished_at")
        fi

        # Check if TermLink session still alive
        local session_alive="no"
        termlink list 2>/dev/null | grep -q "$name" && session_alive="yes" || true

        printf "  %-20s  status: %-20s  session: %s\n" "$name" "$status" "$session_alive"
        if [ -n "$started" ]; then
            printf "  %-20s  started: %s" "" "$started"
            [ -n "$finished" ] && printf "  finished: %s" "$finished"
            echo ""
        fi
        count=$((count + 1))
    done

    [ "$count" = "0" ] && echo "No workers dispatched."
}

cmd_wait() {
    ensure_termlink
    local name="" wait_all=false timeout=600

    while [[ $# -gt 0 ]]; do
        case $1 in
            --name) name="$2"; shift 2 ;;
            --all) wait_all=true; shift ;;
            --timeout) timeout="$2"; shift 2 ;;
            *) die "Unknown option: $1" ;;
        esac
    done

    if [ "$wait_all" = true ]; then
        # Wait for all workers
        [ -d "$DISPATCH_DIR" ] || die "No workers dispatched"
        local all_done=false
        local deadline=$(($(date +%s) + timeout))

        while [ "$all_done" != true ] && [ "$(date +%s)" -lt "$deadline" ]; do
            all_done=true
            for wdir in "$DISPATCH_DIR"/*/; do
                [ -d "$wdir" ] || continue
                if [ ! -f "$wdir/exit_code" ]; then
                    all_done=false
                    break
                fi
            done
            [ "$all_done" = true ] || sleep 2
        done

        if [ "$all_done" = true ]; then
            echo "All workers complete."
        else
            echo "Timeout waiting for workers."
            return 1
        fi
    else
        [ -z "$name" ] && die "Missing --name (or use --all)"
        local wdir
        wdir=$(worker_dir "$name")
        [ -d "$wdir" ] || die "No worker named '$name'"

        # Try event-based wait first, fall back to file polling
        if termlink list 2>/dev/null | grep -q "$name"; then
            termlink event wait "$name" --topic worker.done --timeout "$timeout" >/dev/null 2>&1 || true
        fi

        # File-based confirmation
        local deadline=$(($(date +%s) + timeout))
        while [ ! -f "$wdir/exit_code" ] && [ "$(date +%s)" -lt "$deadline" ]; do
            sleep 2
        done

        if [ -f "$wdir/exit_code" ]; then
            local ec
            ec=$(cat "$wdir/exit_code")
            echo "Worker $name finished (exit: $ec)"
            return "$ec"
        else
            echo "Timeout waiting for worker $name"
            return 1
        fi
    fi
}

cmd_result() {
    local name=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --name) name="$2"; shift 2 ;;
            *) die "Unknown option: $1" ;;
        esac
    done

    [ -z "$name" ] && die "Missing --name"
    local wdir
    wdir=$(worker_dir "$name")
    [ -d "$wdir" ] || die "No worker named '$name'"

    if [ -f "$wdir/result.md" ]; then
        cat "$wdir/result.md"
    else
        echo "No result yet (worker still running?)"
        return 1
    fi
}

cmd_cleanup() {
    echo "Cleaning up workers..."

    if [ -d "$DISPATCH_DIR" ]; then
        # Collect tracked window IDs
        local window_ids=""
        for wdir in "$DISPATCH_DIR"/*/; do
            [ -d "$wdir" ] || continue
            if [ -f "$wdir/window_id" ]; then
                local wid
                wid=$(cat "$wdir/window_id")
                [ -n "$wid" ] && window_ids="${window_ids:+$window_ids }$wid"
            fi
        done

        # Kill TermLink session processes
        for wdir in "$DISPATCH_DIR"/*/; do
            [ -d "$wdir" ] || continue
            local name
            name=$(basename "$wdir")
            local pid
            pid=$(termlink list 2>/dev/null | grep "$name" | awk '{print $1}')
            if [ -n "$pid" ]; then
                kill "$pid" 2>/dev/null || true
            fi
        done

        sleep 1
        termlink clean 2>/dev/null || true

        # 3-phase Terminal.app window cleanup (from T-074)
        # Phase 1: kill child processes via TTY (spare login/shell)
        for wid in $window_ids; do
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
                local pids
                pids=$(ps -t "$tty_short" -o pid=,comm= 2>/dev/null \
                    | grep -v -E '(login|-zsh|-bash)' \
                    | awk '{print $1}' || true)
                if [ -n "$pids" ]; then
                    echo "$pids" | xargs kill -9 2>/dev/null || true
                fi
            fi
        done

        sleep 2

        # Phase 2: send "exit" to each window (shell exits, window auto-closes)
        for wid in $window_ids; do
            osascript -e "
                tell application \"Terminal\"
                    try
                        do script \"exit\" in window id $wid
                    end try
                end tell
            " 2>/dev/null || true
        done

        sleep 2

        # Phase 3: close any remaining windows by reference (fallback)
        if [ -n "$window_ids" ]; then
            local id_list=""
            for wid in $window_ids; do
                id_list="${id_list:+$id_list, }$wid"
            done
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

        rm -rf "$DISPATCH_DIR"
        echo "All workers cleaned up."
    else
        echo "No workers to clean up."
    fi
}

# --- Main ---

ensure_termlink
ensure_claude

case "${1:-}" in
    status)   cmd_status ;;
    wait)     shift; cmd_wait "$@" ;;
    result)   shift; cmd_result "$@" ;;
    cleanup)  cmd_cleanup ;;
    --name)   cmd_spawn "$@" ;;
    -h|--help|"")
        echo "Usage:"
        echo "  tl-dispatch.sh --name <worker> --prompt \"...\" [--project /path] [--timeout 300]"
        echo "  tl-dispatch.sh --name <worker> --prompt-file /path/to/prompt.md"
        echo "  tl-dispatch.sh status"
        echo "  tl-dispatch.sh wait --name <worker>  |  wait --all"
        echo "  tl-dispatch.sh result --name <worker>"
        echo "  tl-dispatch.sh cleanup"
        ;;
    *)        die "Unknown command: $1. Use --help for usage." ;;
esac
