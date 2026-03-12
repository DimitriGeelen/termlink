#!/usr/bin/env bash
# =============================================================================
# Level 2: File Task — Specialist summarizes a file
# =============================================================================
# Orchestrator delegates a real task: "summarize this source file".
# Specialist reads the file, writes a summary to disk, emits task.completed
# with result_path following the delegation event schema convention.
#
# Usage: ./tests/e2e/level2-file-task.sh
# =============================================================================

set -euo pipefail

source "$(dirname "$0")/setup.sh"

PROMPT_FILE="$RUNTIME_DIR/specialist-prompt.txt"
SUMMARY_FILE="$RUNTIME_DIR/summary.md"
RESULT_MARKER="$RUNTIME_DIR/task-done.txt"

# The file the specialist will summarize
TARGET_FILE="$PROJECT_ROOT/crates/termlink-protocol/src/lib.rs"

echo "=== Level 2: File Task ==="
echo "Runtime: $RUNTIME_DIR"
echo "Target:  $TARGET_FILE"
echo ""

build_termlink
register_orchestrator

# Step 3: Write specialist prompt
cat > "$PROMPT_FILE" <<PROMPT
You are a code analysis specialist. Your task:

1. Read this file: $TARGET_FILE
2. Write a concise summary (5-10 lines) of what the file does, its key types, and its role in the project. Write the summary to: $SUMMARY_FILE
3. Then emit a task.completed event to the orchestrator by running:
   TERMLINK_RUNTIME_DIR=$RUNTIME_DIR $TERMLINK emit orchestrator task.completed --payload '{"request_id":"req-level2","status":"completed","result_path":"$SUMMARY_FILE","summary":"File analysis complete"}'
4. Then write DONE to: $RESULT_MARKER

Do these steps in order. Use the Bash tool and Read tool.
PROMPT

# Step 4: Spawn specialist
echo "--- Step 3: Spawn specialist ---"
spawn_tracked \
    --name file-analyst \
    --roles analyst \
    --wait --wait-timeout 15 \
    -- bash -c "unset CLAUDECODE; $CLAUDE -p \"\$(cat $PROMPT_FILE)\" --allowedTools 'Bash,Read' --dangerously-skip-permissions"

echo "Specialist spawned"
echo ""

# Step 5: Wait for result
echo "--- Step 4: Wait for specialist ---"
TIMEOUT=180
ELAPSED=0
while [ $ELAPSED -lt $TIMEOUT ]; do
    if [ -f "$RESULT_MARKER" ]; then
        echo "Task complete after ${ELAPSED}s"
        break
    fi
    sleep 5
    ELAPSED=$((ELAPSED + 5))
    if [ $((ELAPSED % 15)) -eq 0 ]; then
        echo "  Waiting... ${ELAPSED}s / ${TIMEOUT}s"
    fi
done

if [ $ELAPSED -ge $TIMEOUT ]; then
    echo "FAIL: Timeout waiting for specialist"
    exit 1
fi
echo ""

# Step 6: Verify results
echo "--- Step 5: Verify results ---"
echo ""

# Check summary file
if [ -f "$SUMMARY_FILE" ]; then
    echo "Summary file exists: $SUMMARY_FILE"
    echo "--- Summary content ---"
    cat "$SUMMARY_FILE"
    echo "--- End summary ---"
    SUMMARY_OK=true
else
    echo "FAIL: Summary file not found"
    SUMMARY_OK=false
fi
echo ""

# Check event
EVENTS=$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" events orchestrator --topic task.completed 2>/dev/null || true)
echo "Events: $EVENTS"

if echo "$EVENTS" | grep -q "req-level2"; then
    EVENT_OK=true
else
    EVENT_OK=false
fi
echo ""

if [ "$SUMMARY_OK" = true ] && [ "$EVENT_OK" = true ]; then
    echo "========================================="
    echo "  LEVEL 2 PASSED — File Task"
    echo "========================================="
    echo "A Claude Code specialist agent:"
    echo "  1. Spawned in a new terminal"
    echo "  2. Read a source file"
    echo "  3. Wrote an analysis summary to disk"
    echo "  4. Emitted task.completed with result_path"
    echo "  5. Followed the delegation event schema"
elif [ "$SUMMARY_OK" = true ]; then
    echo "=== LEVEL 2 PARTIAL ==="
    echo "Summary written but task.completed event missing."
else
    echo "=== LEVEL 2 FAILED ==="
    echo "Summary: $SUMMARY_OK, Event: $EVENT_OK"
fi
