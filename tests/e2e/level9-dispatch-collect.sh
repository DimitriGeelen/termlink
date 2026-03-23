#!/usr/bin/env bash
# =============================================================================
# Level 9: Dispatch-Collect Pattern — worker push + orchestrator fan-in
# =============================================================================
# Exercises the collect-based dispatch convention (T-257):
#   1. Spawn 3 workers, each emits task.completed → orchestrator collects all 3
#   2. Workers emit task.progress during work → orchestrator sees progress
#   3. Partial failure: 1 worker dies → collect times out, reports 2/3
#   4. Parent session ID injection: workers receive TERMLINK_PARENT_SESSION
#   5. Payload structure: task_id, summary, status in completed events
#   6. Multiple topics: collect filters by topic correctly
#
# Uses real termlink sessions + hub (no mocks).
#
# Usage: ./tests/e2e/level9-dispatch-collect.sh
# =============================================================================

set -euo pipefail

source "$(dirname "$0")/setup.sh"

echo "============================================="
echo "  Level 9: Dispatch-Collect Pattern"
echo "============================================="
echo "Runtime: $RUNTIME_DIR"
echo ""

build_termlink

# --- Start hub ---
echo "--- Start hub ---"
TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" hub start &
HUB_PID=$!

for i in $(seq 1 10); do
    if [ -S "$RUNTIME_DIR/hub.sock" ]; then break; fi
    sleep 1
done

if [ ! -S "$RUNTIME_DIR/hub.sock" ]; then
    echo "FAIL: hub did not start"; exit 1
fi
echo "Hub running (PID $HUB_PID)"
echo ""

# --- Test helpers ---
PASS=0
FAIL=0
TOTAL=0

report() {
    local name="$1" result="$2"
    TOTAL=$((TOTAL + 1))
    if [ "$result" = "PASS" ]; then
        PASS=$((PASS + 1))
        echo "  PASS: $name"
    else
        FAIL=$((FAIL + 1))
        echo "  FAIL: $name"
    fi
}

# run_with_timeout: macOS-compatible timeout (no GNU timeout needed)
# Usage: run_with_timeout SECONDS COMMAND [ARGS...]
run_with_timeout() {
    local secs="$1"; shift
    "$@" &
    local pid=$!
    (sleep "$secs" && kill "$pid" 2>/dev/null) &
    local timer_pid=$!
    wait "$pid" 2>/dev/null || true
    kill "$timer_pid" 2>/dev/null || true
}

# --- Register orchestrator session ---
echo "--- Register orchestrator ---"
tl register --name "dispatch-orch" --roles "orchestrator" --tags "dispatch-test" &
ORCH_PID=$!
sleep 2
if tl ping "dispatch-orch" 2>/dev/null; then
    echo "Orchestrator OK"
else
    echo "FAIL: orchestrator not registered"; exit 1
fi
echo ""

# Get orchestrator session ID for parent injection
ORCH_SESSION_ID=$(tl list 2>/dev/null | grep "dispatch-orch" | awk '{print $1}')
echo "Orchestrator session: $ORCH_SESSION_ID"
echo ""

# =============================================================================
# Scenario 1: Basic dispatch-collect — 3 workers, all succeed
# =============================================================================
echo "=== Scenario 1: Basic dispatch-collect (3 workers) ==="

# Spawn 3 worker sessions
for i in 1 2 3; do
    tl register --name "worker-$i" --roles "worker" --tags "task:T-TEST,worker" &
    eval "WORKER${i}_PID=$!"
done
sleep 2

# Verify workers alive
ALIVE=0
for i in 1 2 3; do
    if tl ping "worker-$i" 2>/dev/null; then
        ALIVE=$((ALIVE + 1))
    fi
done
report "All 3 workers registered" "$([ "$ALIVE" -eq 3 ] && echo PASS || echo FAIL)"

# Workers emit task.completed with structured payload
for i in 1 2 3; do
    tl event emit "worker-$i" "task.completed" \
        -p "{\"task_id\":\"T-TEST\",\"worker\":\"worker-$i\",\"summary\":\"Worker $i done\",\"status\":\"ok\"}" 2>/dev/null
done

# Orchestrator collects all 3 via hub fan-in
# Output format: [worker-1#0] task.completed: {"task_id":"T-TEST",...} (t=...)
COLLECT_OUTPUT=$(tl event collect --topic "task.completed" --count 3 --interval 250 2>/dev/null || true)
COLLECT_COUNT=$(echo "$COLLECT_OUTPUT" | grep -c 'task\.completed:' || true)

report "Collect received 3 task.completed events" "$([ "$COLLECT_COUNT" -ge 3 ] && echo PASS || echo FAIL)"

# Verify payload structure (payload is inline JSON after topic:)
HAS_TASK_ID=$(echo "$COLLECT_OUTPUT" | grep -c 'task_id' || true)
HAS_SUMMARY=$(echo "$COLLECT_OUTPUT" | grep -c 'summary' || true)
report "Payloads contain task_id and summary" "$([ "$HAS_TASK_ID" -ge 3 ] && [ "$HAS_SUMMARY" -ge 3 ] && echo PASS || echo FAIL)"

echo ""

# =============================================================================
# Scenario 2: Progress events — workers emit progress before completion
# =============================================================================
echo "=== Scenario 2: Progress events ==="

# Worker 1 emits progress
tl event emit "worker-1" "task.progress" \
    -p "{\"task_id\":\"T-TEST\",\"worker\":\"worker-1\",\"percent\":50,\"message\":\"Halfway\"}" 2>/dev/null
tl event emit "worker-1" "task.progress" \
    -p "{\"task_id\":\"T-TEST\",\"worker\":\"worker-1\",\"percent\":100,\"message\":\"Done\"}" 2>/dev/null

# Collect progress events
PROGRESS_OUTPUT=$(tl event collect --topic "task.progress" --count 2 --interval 250 2>/dev/null || true)
PROGRESS_COUNT=$(echo "$PROGRESS_OUTPUT" | grep -c 'task\.progress:' || true)

report "Collected 2 progress events from worker-1" "$([ "$PROGRESS_COUNT" -ge 2 ] && echo PASS || echo FAIL)"

# Verify progress payload has percent
HAS_PERCENT=$(echo "$PROGRESS_OUTPUT" | grep -c 'percent' || true)
report "Progress events contain percent field" "$([ "$HAS_PERCENT" -ge 2 ] && echo PASS || echo FAIL)"

echo ""

# =============================================================================
# Scenario 3: Partial failure — 1 worker dies, collect times out
# =============================================================================
echo "=== Scenario 3: Partial failure (1 of 3 workers dies) ==="

# Kill worker-3
kill "$WORKER3_PID" 2>/dev/null || true
sleep 1

# Workers 1 and 2 emit on a fresh topic
tl event emit "worker-1" "task.result" \
    -p "{\"task_id\":\"T-PARTIAL\",\"worker\":\"worker-1\",\"status\":\"ok\"}" 2>/dev/null
tl event emit "worker-2" "task.result" \
    -p "{\"task_id\":\"T-PARTIAL\",\"worker\":\"worker-2\",\"status\":\"ok\"}" 2>/dev/null

# Collect with count=3 but short timeout — should get 2, then time out
# Use run_with_timeout since macOS has no `timeout` command
PARTIAL_OUTPUT_FILE="$RUNTIME_DIR/partial-output.txt"
run_with_timeout 8 bash -c "TERMLINK_RUNTIME_DIR=\"$RUNTIME_DIR\" \"$TERMLINK\" event collect --topic task.result --count 3 --interval 250 > \"$PARTIAL_OUTPUT_FILE\" 2>&1"
PARTIAL_OUTPUT=$(cat "$PARTIAL_OUTPUT_FILE" 2>/dev/null || true)
PARTIAL_COUNT=$(echo "$PARTIAL_OUTPUT" | grep -c 'task\.result:' || true)

report "Collected 2/3 results before timeout (dead worker)" "$([ "$PARTIAL_COUNT" -eq 2 ] && echo PASS || echo FAIL)"

echo ""

# =============================================================================
# Scenario 4: Parent session ID injection
# =============================================================================
echo "=== Scenario 4: Parent session ID injection ==="

# Verify ORCH_SESSION_ID was captured
report "Orchestrator session ID captured" "$([ -n "$ORCH_SESSION_ID" ] && echo PASS || echo FAIL)"

# Worker can address orchestrator by session ID
if tl ping "$ORCH_SESSION_ID" 2>/dev/null; then
    report "Worker can ping orchestrator by session ID" "PASS"
else
    report "Worker can ping orchestrator by session ID" "FAIL"
fi

# Worker emits to orchestrator's bus (push-back pattern)
tl event emit "$ORCH_SESSION_ID" "worker.report" \
    -p "{\"from\":\"worker-1\",\"message\":\"Reporting back to parent\"}" 2>/dev/null

# Orchestrator reads its own events
# Poll output format: [0] worker.report: {"from":"worker-1",...} (t=...)
SELF_EVENTS=$(tl event poll "dispatch-orch" --topic "worker.report" 2>/dev/null || true)
HAS_REPORT=$(echo "$SELF_EVENTS" | grep -c 'worker\.report:' || true)

report "Worker pushed event to orchestrator's bus" "$([ "$HAS_REPORT" -ge 1 ] && echo PASS || echo FAIL)"

echo ""

# =============================================================================
# Scenario 5: Topic filtering — collect only gets matching topic
# =============================================================================
echo "=== Scenario 5: Topic filtering ==="

# Emit on two different topics
tl event emit "worker-1" "build.done" \
    -p "{\"type\":\"build\"}" 2>/dev/null
tl event emit "worker-2" "test.done" \
    -p "{\"type\":\"test\"}" 2>/dev/null
tl event emit "worker-1" "build.done" \
    -p "{\"type\":\"build2\"}" 2>/dev/null

# Collect only build.done
BUILD_OUTPUT=$(tl event collect --topic "build.done" --count 2 --interval 250 2>/dev/null || true)
BUILD_COUNT=$(echo "$BUILD_OUTPUT" | grep -c 'build\.done:' || true)
TEST_LEAK=$(echo "$BUILD_OUTPUT" | grep -c 'test\.done:' || true)

report "Collect filters by topic (got 2 build.done)" "$([ "$BUILD_COUNT" -ge 2 ] && echo PASS || echo FAIL)"
report "No topic leakage (0 test.done in build collect)" "$([ "$TEST_LEAK" -eq 0 ] && echo PASS || echo FAIL)"

echo ""

# =============================================================================
# Scenario 6: Discover workers by tag
# =============================================================================
echo "=== Scenario 6: Discover workers by tag ==="

DISCOVERED=$(tl discover --tag "task:T-TEST" 2>/dev/null || true)
DISC_COUNT=$(echo "$DISCOVERED" | grep -c "worker" || true)

# worker-3 is dead but may still be registered
report "Discover finds workers by task tag" "$([ "$DISC_COUNT" -ge 2 ] && echo PASS || echo FAIL)"

echo ""

# =============================================================================
# Summary
# =============================================================================
echo "============================================="
echo "  Results: $PASS/$TOTAL passed, $FAIL failed"
echo "============================================="

# Cleanup: kill remaining workers
kill "$WORKER1_PID" "$WORKER2_PID" 2>/dev/null || true
kill "$HUB_PID" 2>/dev/null || true

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
