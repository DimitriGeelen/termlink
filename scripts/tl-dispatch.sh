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
SPAWN_BACKEND="${TL_DISPATCH_BACKEND:-auto}"

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

# _resolve_dispatch_model — substrate-side model resolution (U-005 / T-1442).
#
# Inputs:  $1 = explicit --model value (may be empty)
#          $2 = task_type (may be empty)
# Outputs: prints "<model>|<fallback_used>" on stdout, where:
#          <model>         = effective model (string, possibly empty)
#          <fallback_used> = "true" | "false" | "" (empty when no model resolves)
#
# Resolution order (mirrors framework's agents/termlink/termlink.sh):
#   1. Explicit --model wins → fallback_used=false
#   2. DISPATCH_MODEL_FOR_<TYPE> env var (uppercased)  → fallback_used=false
#   3. DISPATCH_MODEL_DEFAULT env var → fallback_used=true (no per-type specialist)
#   4. Nothing → empty model, empty fallback flag (caller emits JSON null)
#
# Env vars are read directly. Callers (e.g. the framework adapter) export them
# from .framework.yaml or wherever their config lives.
_resolve_dispatch_model() {
    local explicit="$1" task_type="$2"
    if [ -n "$explicit" ]; then
        echo "${explicit}|false"
        return 0
    fi
    if [ -n "$task_type" ]; then
        local key="DISPATCH_MODEL_FOR_$(echo "$task_type" | tr '[:lower:]' '[:upper:]')"
        local v="${!key:-}"
        if [ -n "$v" ]; then
            echo "${v}|false"
            return 0
        fi
    fi
    local d="${DISPATCH_MODEL_DEFAULT:-}"
    if [ -n "$d" ]; then
        echo "${d}|true"
        return 0
    fi
    echo "|"
    return 0
}

# _json_str_or_null — emit a JSON-quoted string, or `null` when empty.
_json_str_or_null() {
    local v="$1"
    if [ -z "$v" ]; then
        printf 'null'
    else
        printf '"%s"' "$v"
    fi
}

# _json_bool_or_null — emit `true`/`false` literal, or `null` when empty.
_json_bool_or_null() {
    local v="$1"
    case "$v" in
        true|false) printf '%s' "$v" ;;
        *) printf 'null' ;;
    esac
}

# --- Commands ---

cmd_spawn() {
    local name="" prompt="" prompt_file="" project_dir="" timeout=600 backend="$SPAWN_BACKEND"
    local model="" task_type=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --name) name="$2"; shift 2 ;;
            --prompt) prompt="$2"; shift 2 ;;
            --prompt-file) prompt_file="$2"; shift 2 ;;
            --project) project_dir="$2"; shift 2 ;;
            --timeout) timeout="$2"; shift 2 ;;
            --backend) backend="$2"; shift 2 ;;
            --model) model="$2"; shift 2 ;;
            --task-type) task_type="$2"; shift 2 ;;
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

    # U-005: substrate resolves model + fallback flag from explicit / per-type / default chain.
    # If nothing resolves, both fields stay null in meta.json (don't lie about state).
    local resolved model_used fallback_used
    resolved=$(_resolve_dispatch_model "$model" "$task_type")
    model_used="${resolved%%|*}"
    fallback_used="${resolved##*|}"

    # Create worker directory
    local wdir
    wdir=$(worker_dir "$name")
    mkdir -p "$wdir"

    # Write prompt to file (avoids shell escaping issues)
    echo "$prompt" > "$wdir/prompt.md"

    # Record metadata. Schema parity with framework's agents/termlink/termlink.sh
    # (T-1643/W4) — task_type, model, model_used, fallback_used are all present.
    # model_used / fallback_used are populated here (substrate-side); the framework's
    # half writes nulls and relies on us. JSON null when no model resolves.
    cat > "$wdir/meta.json" <<METAEOF
{
  "name": "$name",
  "project": "$project_dir",
  "timeout": $timeout,
  "backend": "$backend",
  "task_type": "${task_type}",
  "model": "${model_used}",
  "model_used": $(_json_str_or_null "$model_used"),
  "fallback_used": $(_json_bool_or_null "$fallback_used"),
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
MODEL="$5"

cd "$PROJECT_DIR"

# Build --model flag if a model was resolved at spawn time (U-005).
MODEL_FLAG=""
if [ -n "$MODEL" ]; then
    MODEL_FLAG="--model $MODEL"
fi

# Run claude with the prompt, capture output
# Use background process + kill for timeout (macOS has no `timeout` command)
claude -p "$(cat "$WDIR/prompt.md")" $MODEL_FLAG --output-format text > "$WDIR/result.md" 2>"$WDIR/stderr.log" &
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

    # Spawn session via termlink spawn (delegates backend selection to the binary)
    termlink spawn --name "$name" --shell --wait --wait-timeout 15 --backend "$backend" \
        || die "Failed to spawn session '$name' (backend: $backend)"

    # Record the PID for cleanup
    local pid
    pid=$(termlink list 2>/dev/null | grep "$name" | awk '{print $4}')
    [ -n "$pid" ] && echo "$pid" > "$wdir/pid"

    # For tmux, record session name
    if [ "$backend" = "tmux" ] || { [ "$backend" = "auto" ] && tmux has-session -t "tl-$name" 2>/dev/null; }; then
        echo "tl-$name" > "$wdir/tmux_session"
    fi

    # Inject the worker script (fire-and-forget, don't wait)
    sleep 1
    termlink pty inject "$name" "bash $wdir/run.sh '$name' '$project_dir' '$wdir' '$timeout' '$model_used'" --enter >/dev/null 2>&1

    echo "Worker spawned: $name (backend: $backend)"
    echo "  Project: $project_dir"
    echo "  Result:  $wdir/result.md"
    echo "  Timeout: ${timeout}s"
    [ -n "$task_type" ] && echo "  Task-type: $task_type"
    [ -n "$model_used" ] && echo "  Model: $model_used (fallback: $fallback_used)"
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
        # Per-worker cleanup based on backend
        for wdir in "$DISPATCH_DIR"/*/; do
            [ -d "$wdir" ] || continue
            local name
            name=$(basename "$wdir")

            # Kill the TermLink session process by PID
            if [ -f "$wdir/pid" ]; then
                local pid
                pid=$(cat "$wdir/pid")
                kill "$pid" 2>/dev/null || true
            fi

            # Kill tmux session if it exists
            if [ -f "$wdir/tmux_session" ]; then
                local tmux_name
                tmux_name=$(cat "$wdir/tmux_session")
                tmux kill-session -t "$tmux_name" 2>/dev/null || true
            fi
        done

        # Collect tracked window IDs for Terminal.app cleanup
        local window_ids=""
        for wdir in "$DISPATCH_DIR"/*/; do
            [ -d "$wdir" ] || continue
            if [ -f "$wdir/window_id" ]; then
                local wid
                wid=$(cat "$wdir/window_id")
                [ -n "$wid" ] && window_ids="${window_ids:+$window_ids }$wid"
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
        echo "  tl-dispatch.sh --name <worker> --prompt \"...\" [--project /path] [--timeout 300] [--backend auto]"
        echo "  tl-dispatch.sh --name <worker> --prompt-file /path/to/prompt.md"
        echo "  tl-dispatch.sh status"
        echo "  tl-dispatch.sh wait --name <worker>  |  wait --all"
        echo "  tl-dispatch.sh result --name <worker>"
        echo "  tl-dispatch.sh cleanup"
        echo ""
        echo "Backends: auto (default), terminal (macOS), tmux (headless), background (fallback)"
        echo "Set TL_DISPATCH_BACKEND=tmux or use --backend tmux to override."
        ;;
    *)        die "Unknown command: $1. Use --help for usage." ;;
esac
