#!/usr/bin/env bash
# TermLink Agent Mesh — Long-running worker
# Registers as a TermLink session, watches for task.dispatch events,
# executes each task via Claude Code, emits task.result back.
#
# Usage: worker.sh [--name NAME] [--roles ROLES]
#
# Environment:
#   TERMLINK_WORKER_NAME     Worker display name (default: mesh-worker-<pid>)
#   TERMLINK_WORKER_ROLES    Comma-separated roles (default: worker)
#   TERMLINK_WORKER_TIMEOUT  Per-task timeout in seconds (default: 120)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKER_NAME="${TERMLINK_WORKER_NAME:-mesh-worker-$$}"
WORKER_ROLES="${TERMLINK_WORKER_ROLES:-worker}"
TASK_TIMEOUT="${TERMLINK_WORKER_TIMEOUT:-120}"
POLL_INTERVAL=1  # seconds

# Parse args
while [[ $# -gt 0 ]]; do
    case "$1" in
        --name) WORKER_NAME="$2"; shift 2 ;;
        --roles) WORKER_ROLES="$2"; shift 2 ;;
        *) echo "Unknown arg: $1" >&2; exit 1 ;;
    esac
done

# --- Register as TermLink session ---
# We register in the background so we can poll events in the foreground
WORKER_REG_FILE=$(mktemp /tmp/termlink-worker-XXXXXX)
termlink register --name "$WORKER_NAME" --roles "$WORKER_ROLES" --tags "agent-mesh" &
REGISTER_PID=$!
sleep 1

# Verify registration
if ! termlink ping "$WORKER_NAME" >/dev/null 2>&1; then
    echo "ERROR: Failed to register worker '$WORKER_NAME'" >&2
    kill $REGISTER_PID 2>/dev/null
    exit 1
fi

echo "Worker '$WORKER_NAME' registered and listening for tasks..." >&2
echo "  Roles: $WORKER_ROLES" >&2
echo "  Task timeout: ${TASK_TIMEOUT}s" >&2
echo "  Poll interval: ${POLL_INTERVAL}s" >&2
echo "" >&2

# Cleanup on exit
cleanup() {
    echo "" >&2
    echo "Shutting down worker '$WORKER_NAME'..." >&2
    kill $REGISTER_PID 2>/dev/null
    rm -f "$WORKER_REG_FILE"
    exit 0
}
trap cleanup EXIT INT TERM

# --- Event loop ---
# Start with no cursor (returns all events). After first poll, use --since with
# the last seen seq. Note: --since is exclusive (seq > cursor), so we track
# the last processed seq and use it directly.
CURSOR=""
TASKS_COMPLETED=0

while true; do
    # Poll for task.dispatch events
    if [[ -z "$CURSOR" ]]; then
        EVENTS=$(termlink events "$WORKER_NAME" --topic task.dispatch 2>/dev/null || true)
    else
        EVENTS=$(termlink events "$WORKER_NAME" --topic task.dispatch --since "$CURSOR" 2>/dev/null || true)
    fi

    if [[ -n "$EVENTS" ]] && ! echo "$EVENTS" | grep -q "^No events"; then
        # Parse each event line: [SEQ] topic: PAYLOAD (t=TIMESTAMP)
        while IFS= read -r line; do
            [[ -z "$line" ]] && continue
            [[ "$line" =~ ^[0-9]+\ event ]] && continue  # skip summary line

            # Extract sequence number and payload
            SEQ=$(echo "$line" | grep -oE '^\[([0-9]+)\]' | tr -d '[]')
            PAYLOAD=$(echo "$line" | sed 's/^\[[0-9]*\] [^:]*: //' | sed 's/ (t=[0-9]*)$//')

            [[ -z "$SEQ" ]] && continue

            # Update cursor past this event
            CURSOR=$SEQ

            # Extract prompt from payload JSON
            PROMPT=$(echo "$PAYLOAD" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('prompt',''))" 2>/dev/null || true)
            TASK_ID=$(echo "$PAYLOAD" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('task_id','unknown'))" 2>/dev/null || true)
            REPLY_TO=$(echo "$PAYLOAD" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('reply_to',''))" 2>/dev/null || true)

            if [[ -z "$PROMPT" ]]; then
                echo "[WARN] Empty prompt in event seq=$SEQ, skipping" >&2
                continue
            fi

            echo "[TASK] $TASK_ID — processing (seq=$SEQ)..." >&2
            echo "  Prompt: ${PROMPT:0:80}..." >&2

            # Execute via agent-wrapper
            RESULT_FILE=$(mktemp /tmp/termlink-result-XXXXXX)
            if "$SCRIPT_DIR/agent-wrapper.sh" "$PROMPT" > "$RESULT_FILE" 2>/dev/null; then
                RESULT=$(cat "$RESULT_FILE")
                STATUS="done"
                echo "[DONE] $TASK_ID — success" >&2
            else
                RESULT="Worker execution failed (exit $?)"
                STATUS="error"
                echo "[FAIL] $TASK_ID — $RESULT" >&2
            fi
            rm -f "$RESULT_FILE"

            # Emit result event back
            if [[ -n "$REPLY_TO" ]]; then
                # Reply to the requesting session
                RESULT_JSON=$(python3 -c "import json,sys; print(json.dumps({'task_id': sys.argv[1], 'status': sys.argv[2], 'result': sys.argv[3], 'worker': sys.argv[4]}))" "$TASK_ID" "$STATUS" "$RESULT" "$WORKER_NAME" 2>/dev/null)
                termlink emit "$REPLY_TO" task.result -p "$RESULT_JSON" 2>/dev/null || true
                echo "  Result emitted to $REPLY_TO" >&2
            else
                echo "  [WARN] No reply_to in payload, result not emitted" >&2
            fi

            TASKS_COMPLETED=$((TASKS_COMPLETED + 1))
            echo "  Tasks completed: $TASKS_COMPLETED" >&2
            echo "" >&2

        done <<< "$EVENTS"
    fi

    sleep "$POLL_INTERVAL"
done
