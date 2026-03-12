#!/usr/bin/env bash
# TermLink Agent Mesh — Orchestrator
# Discovers a worker, dispatches a task via event, waits for result.
#
# Usage: orchestrate.sh "prompt text" [--worker-role ROLE] [--timeout SECS]
#
# The orchestrator:
#   1. Registers itself as a TermLink session
#   2. Discovers a worker by role
#   3. Emits task.dispatch event to the worker (with reply_to = self)
#   4. Waits for task.result event on its own event bus
#   5. Outputs the result and deregisters

set -euo pipefail

PROMPT="${1:?Usage: orchestrate.sh \"prompt text\" [--worker-role ROLE] [--timeout SECS]}"
shift
WORKER_ROLE="worker"
TIMEOUT=120

# Parse remaining args
while [[ $# -gt 0 ]]; do
    case "$1" in
        --worker-role) WORKER_ROLE="$2"; shift 2 ;;
        --timeout) TIMEOUT="$2"; shift 2 ;;
        *) echo "Unknown arg: $1" >&2; exit 1 ;;
    esac
done

ORCH_NAME="orchestrator-$$"
TASK_ID="task-$$-$(date +%s)"

# --- Step 1: Register orchestrator session ---
termlink register --name "$ORCH_NAME" --roles orchestrator --tags "agent-mesh" &
REGISTER_PID=$!
sleep 1

if ! termlink ping "$ORCH_NAME" >/dev/null 2>&1; then
    echo "ERROR: Failed to register orchestrator" >&2
    kill $REGISTER_PID 2>/dev/null
    exit 1
fi

# Cleanup on exit
cleanup() {
    kill $REGISTER_PID 2>/dev/null
    wait $REGISTER_PID 2>/dev/null
}
trap cleanup EXIT INT TERM

# --- Step 2: Discover worker ---
WORKER=$(termlink discover --role "$WORKER_ROLE" 2>/dev/null | grep -v '^ID\|^-' | head -1 | awk '{print $2}')

if [[ -z "$WORKER" ]] || [[ "$WORKER" == "NAME" ]]; then
    echo "ERROR: No worker with role '$WORKER_ROLE' found" >&2
    echo "Start a worker first: agents/mesh/worker.sh" >&2
    exit 1
fi

echo "Discovered worker: $WORKER" >&2
echo "Dispatching task: $TASK_ID" >&2
echo "Prompt: ${PROMPT:0:80}..." >&2

# --- Step 3: Emit task.dispatch to worker ---
DISPATCH_JSON=$(python3 -c "
import json, sys
print(json.dumps({
    'task_id': sys.argv[1],
    'prompt': sys.argv[2],
    'reply_to': sys.argv[3]
}))
" "$TASK_ID" "$PROMPT" "$ORCH_NAME" 2>/dev/null)

termlink emit "$WORKER" task.dispatch -p "$DISPATCH_JSON" >/dev/null 2>&1

echo "Task dispatched, waiting for result (timeout: ${TIMEOUT}s)..." >&2

# --- Step 4: Wait for task.result ---
RESULT=$(termlink wait "$ORCH_NAME" --topic task.result --timeout "$TIMEOUT" 2>/dev/null)
WAIT_EXIT=$?

if [[ $WAIT_EXIT -ne 0 ]]; then
    echo "ERROR: Timeout waiting for worker result" >&2
    exit 1
fi

# --- Step 5: Output result ---
# Extract the actual result from the JSON payload
ACTUAL_RESULT=$(echo "$RESULT" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    if d.get('status') == 'error':
        print(f\"ERROR: {d.get('result', 'unknown error')}\", file=sys.stderr)
        sys.exit(1)
    print(d.get('result', ''))
except:
    print(sys.stdin.read())
" 2>/dev/null)

echo "$ACTUAL_RESULT"
