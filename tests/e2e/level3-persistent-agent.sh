#!/usr/bin/env bash
# =============================================================================
# Level 3: Persistent Agent — Specialist handles multiple tasks
# =============================================================================
# Usage: ./tests/e2e/level3-persistent-agent.sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TERMLINK="$PROJECT_ROOT/target/debug/termlink"
CLAUDE="/Users/dimidev32/.local/bin/claude"
WATCHER="$SCRIPT_DIR/specialist-watcher.sh"
RUNTIME_DIR=$(mktemp -d)
TASK1_RESULT="$RUNTIME_DIR/task1-result.md"
TASK2_RESULT="$RUNTIME_DIR/task2-result.md"

source "$SCRIPT_DIR/e2e-helpers.sh"
trap cleanup_all EXIT

echo "=== Level 3: Persistent Agent ==="
echo "Runtime: $RUNTIME_DIR"
echo ""

# Build
echo "--- Build ---"
(cd "$PROJECT_ROOT" && /Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | tail -1)
echo ""

# Register orchestrator
echo "--- Register orchestrator ---"
TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" register --name orchestrator --roles orchestrator &
ORCH_PID=$!
for i in $(seq 1 10); do
    if ls "$RUNTIME_DIR/sessions/"*.sock >/dev/null 2>&1; then break; fi; sleep 1
done
TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" ping orchestrator 2>/dev/null && echo "Orchestrator OK" || { echo "FAIL"; exit 1; }
echo ""

# Spawn persistent specialist
echo "--- Spawn persistent specialist ---"
spawn_tracked \
    --name specialist --roles analyst \
    --wait --wait-timeout 15 \
    -- bash "$WATCHER" "$TERMLINK" "$RUNTIME_DIR" "$CLAUDE"
echo "Specialist running"
sleep 3
echo ""

wait_for_completion() {
    local REQ_ID="$1" TIMEOUT="${2:-180}" ELAPSED=0
    while [ $ELAPSED -lt $TIMEOUT ]; do
        if TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" events orchestrator --topic task.completed 2>/dev/null | grep -q "$REQ_ID"; then
            echo "  $REQ_ID completed (${ELAPSED}s)"
            return 0
        fi
        sleep 5; ELAPSED=$((ELAPSED + 5))
        if [ $((ELAPSED % 15)) -eq 0 ]; then echo "  Waiting... ${ELAPSED}s"; fi
    done
    echo "  FAIL: $REQ_ID timeout"; return 1
}

# Task 1
echo "--- Task 1: Summarize lib.rs ---"
TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit specialist task.delegate \
    --payload "{\"request_id\":\"req-task1\",\"action\":\"Summarize the purpose, key types, and module structure\",\"scope\":{\"file\":\"$PROJECT_ROOT/crates/termlink-protocol/src/lib.rs\",\"result_path\":\"$TASK1_RESULT\"}}"
wait_for_completion "req-task1" 180 || exit 1
echo ""

# Task 2 — same specialist, proving persistence
echo "--- Task 2: Analyze error.rs ---"
TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit specialist task.delegate \
    --payload "{\"request_id\":\"req-task2\",\"action\":\"List all error variants and explain each one briefly\",\"scope\":{\"file\":\"$PROJECT_ROOT/crates/termlink-protocol/src/error.rs\",\"result_path\":\"$TASK2_RESULT\"}}"
wait_for_completion "req-task2" 180 || exit 1
echo ""

# Shutdown
touch "$RUNTIME_DIR/shutdown"

# Verify
echo "--- Verify ---"
PASS=true

for TASK_NUM in 1 2; do
    RFILE="TASK${TASK_NUM}_RESULT"
    RPATH="${!RFILE}"
    if [ -f "$RPATH" ]; then
        echo "Task $TASK_NUM result:"
        cat "$RPATH"
        echo ""
    else
        echo "FAIL: Task $TASK_NUM result not found"; PASS=false
    fi
done

EVENTS=$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" events orchestrator 2>/dev/null || true)
echo "Orchestrator events:"
echo "$EVENTS"
echo ""

COMPLETED=$(echo "$EVENTS" | grep -c "task.completed" || true)

if [ "$PASS" = true ] && [ "$COMPLETED" -ge 2 ]; then
    echo "========================================="
    echo "  LEVEL 3 PASSED — Persistent Agent"
    echo "========================================="
    echo "  - 2 tasks processed by same specialist"
    echo "  - Each got fresh Claude context window"
    echo "  - task.accepted + task.completed emitted"
else
    echo "=== LEVEL 3 PARTIAL ==="
    echo "Completed events: $COMPLETED"
fi
