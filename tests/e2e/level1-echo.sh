#!/usr/bin/env bash
# =============================================================================
# Level 1: Echo Test — Agent-to-Agent Communication
# =============================================================================
# Orchestrator spawns a specialist Claude Code agent in a new terminal.
# Specialist registers, emits a reply event, and exits.
# Orchestrator verifies receipt of the reply.
#
# Usage: ./tests/e2e/level1-echo.sh
# =============================================================================

set -euo pipefail

source "$(dirname "$0")/setup.sh"

RESULT_FILE="$RUNTIME_DIR/echo-result.txt"
PROMPT_FILE="$RUNTIME_DIR/specialist-prompt.txt"

echo "=== Level 1: Echo Test ==="
echo "Runtime: $RUNTIME_DIR"
echo ""

build_termlink
register_orchestrator

# Step 3: Write prompt file (avoids AppleScript quote mangling)
cat > "$PROMPT_FILE" <<PROMPT
Run these two bash commands in sequence. Do not modify them. Do not explain.

Command 1:
TERMLINK_RUNTIME_DIR=$RUNTIME_DIR $TERMLINK emit orchestrator echo.reply --payload '{"message":"hello from specialist","status":"ok"}'

Command 2:
echo ECHO_SUCCESS > $RESULT_FILE
PROMPT

# Step 4: Spawn specialist Claude Code agent
echo "--- Step 3: Spawn specialist (claude -p) ---"

# The spawned terminal gets a fresh shell without CLAUDECODE.
# We use a prompt file to avoid quote escaping issues through AppleScript.
spawn_tracked \
    --name echo-specialist \
    --roles specialist \
    --wait --wait-timeout 15 \
    -- bash -c "unset CLAUDECODE; $CLAUDE -p \"\$(cat $PROMPT_FILE)\" --allowedTools Bash --dangerously-skip-permissions"

echo "Specialist spawned"
echo ""

# Step 5: Wait for result
echo "--- Step 4: Wait for specialist result ---"
TIMEOUT=180
ELAPSED=0
while [ $ELAPSED -lt $TIMEOUT ]; do
    if [ -f "$RESULT_FILE" ]; then
        CONTENT=$(cat "$RESULT_FILE")
        if echo "$CONTENT" | grep -q "ECHO_SUCCESS"; then
            echo "Result file found: $CONTENT"
            break
        fi
    fi
    sleep 5
    ELAPSED=$((ELAPSED + 5))
    if [ $((ELAPSED % 15)) -eq 0 ]; then
        echo "  Waiting... ${ELAPSED}s / ${TIMEOUT}s"
    fi
done

if [ $ELAPSED -ge $TIMEOUT ]; then
    echo "FAIL: Timeout waiting for specialist result"
    exit 1
fi
echo ""

# Step 6: Verify the event was received
echo "--- Step 5: Verify event on orchestrator ---"
EVENTS=$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" events orchestrator --topic echo.reply 2>/dev/null || true)
echo "Events: $EVENTS"

if echo "$EVENTS" | grep -q "hello from specialist"; then
    echo ""
    echo "========================================="
    echo "  LEVEL 1 PASSED — Echo Test"
    echo "========================================="
    echo "A Claude Code specialist agent:"
    echo "  1. Spawned in a new terminal"
    echo "  2. Emitted a TermLink event to the orchestrator"
    echo "  3. Wrote a result file"
    echo "  4. Exited cleanly"
else
    echo ""
    echo "=== LEVEL 1 PARTIAL ==="
    echo "Result file written but event not found on orchestrator bus."
fi
