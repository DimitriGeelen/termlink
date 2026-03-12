#!/usr/bin/env bash
# TermLink Agent Mesh — Dispatch a task to a worker agent
# Usage: dispatch.sh "prompt text" [--worker-name NAME]
#
# Flow:
#   1. Ensures hub is running
#   2. Spawns a worker agent (Claude Code via agent-wrapper.sh)
#   3. Waits for the worker to register
#   4. Emits task.dispatch event to worker
#   5. Worker executes, writes result to file
#   6. Orchestrator reads result
#   7. Cleanup

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROMPT="${1:?Usage: dispatch.sh \"prompt text\" [--worker-name NAME]}"
WORKER_NAME="${3:-mesh-worker-$$}"
RESULT_FILE="/tmp/termlink-mesh-result-$$.txt"
TIMEOUT="${TERMLINK_DISPATCH_TIMEOUT:-120}"

# --- Step 1: Ensure hub is running ---
if ! termlink info 2>/dev/null | grep -q "Hub socket:"; then
    echo "ERROR: Cannot check hub status" >&2
    exit 1
fi

HUB_SOCK=$(termlink info 2>/dev/null | grep "Hub socket:" | awk '{print $3}')
if [ ! -S "$HUB_SOCK" ]; then
    echo "Starting hub..." >&2
    termlink hub &
    sleep 1
fi

# --- Step 2: Spawn worker agent ---
echo "Dispatching to worker: $WORKER_NAME" >&2
echo "Prompt: ${PROMPT:0:80}..." >&2

# The worker runs the agent-wrapper which executes claude --print
# We run it in background and capture its output
termlink run \
    -n "$WORKER_NAME" \
    -t "worker,agent-mesh" \
    --timeout "$TIMEOUT" \
    -- "$SCRIPT_DIR/agent-wrapper.sh" "$PROMPT" > "$RESULT_FILE" 2>/dev/null &
WORKER_PID=$!

# --- Step 3: Wait for completion ---
echo "Worker PID: $WORKER_PID (timeout: ${TIMEOUT}s)" >&2

if wait $WORKER_PID 2>/dev/null; then
    echo "Worker completed successfully" >&2
else
    EXIT_CODE=$?
    echo "Worker failed (exit $EXIT_CODE)" >&2
    rm -f "$RESULT_FILE"
    exit $EXIT_CODE
fi

# --- Step 4: Output result ---
if [ -f "$RESULT_FILE" ] && [ -s "$RESULT_FILE" ]; then
    cat "$RESULT_FILE"
    rm -f "$RESULT_FILE"
else
    echo "ERROR: No result produced" >&2
    rm -f "$RESULT_FILE"
    exit 1
fi
