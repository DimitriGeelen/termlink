#!/usr/bin/env bash
# Role-aware specialist watcher — uses role-specific system prompts
# Usage: role-watcher.sh <termlink-binary> <runtime-dir> <claude-binary> <session-name> <role>
#
# Differences from specialist-watcher.sh:
#   - Takes a role parameter (reviewer, tester, documenter, git-committer)
#   - Loads role-specific system prompt from role-prompts/<role>.md
#   - Sets role-appropriate --allowedTools per role
set -uo pipefail

TERMLINK="$1"
RUNTIME_DIR="$2"
CLAUDE="$3"
SESSION_NAME="$4"
ROLE="$5"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROLE_PROMPT_FILE="$SCRIPT_DIR/role-prompts/${ROLE}.md"

export TERMLINK_RUNTIME_DIR="$RUNTIME_DIR"
unset CLAUDECODE 2>/dev/null || true

# Validate role prompt exists
if [ ! -f "$ROLE_PROMPT_FILE" ]; then
    echo "[role-watcher] ERROR: No prompt for role '$ROLE' at $ROLE_PROMPT_FILE"
    exit 1
fi

SYSTEM_PROMPT=$(cat "$ROLE_PROMPT_FILE")

# Role-specific tool permissions
case "$ROLE" in
    reviewer)    TOOLS="Read,Write" ;;
    tester)      TOOLS="Bash,Read,Write" ;;
    documenter)  TOOLS="Read,Write" ;;
    git-committer)   TOOLS="Bash,Read,Write" ;;
    infrastructure)  TOOLS="Bash,Read,Write" ;;
    *)               TOOLS="Bash,Read,Write" ;;
esac

echo "[role-watcher:$ROLE] Started (pid=$$, tools=$TOOLS)"
echo $$ >> "$RUNTIME_DIR/pids.txt"

CURSOR=""

while true; do
    if [ -f "$RUNTIME_DIR/shutdown" ]; then
        echo "[role-watcher:$ROLE] Shutdown signal"; break
    fi

    # Poll for task.delegate events
    if [ -z "$CURSOR" ]; then
        RAW=$("$TERMLINK" events "$SESSION_NAME" --topic task.delegate 2>/dev/null || echo "")
    else
        RAW=$("$TERMLINK" events "$SESSION_NAME" --topic task.delegate --since "$CURSOR" 2>/dev/null || echo "")
    fi

    if [ -z "$RAW" ] || echo "$RAW" | grep -q "No events"; then
        sleep 2; continue
    fi

    # Extract first event
    EVENT_LINE=$(echo "$RAW" | grep '^\[' | head -1)
    if [ -z "$EVENT_LINE" ]; then sleep 2; continue; fi

    SEQ=$(echo "$EVENT_LINE" | sed -n 's/^\[\([0-9]*\)\].*/\1/p')
    PAYLOAD=$(echo "$EVENT_LINE" | sed 's/^\[[0-9]*\] task\.delegate: //' | sed 's/ (t=[0-9]*)$//')

    echo "[role-watcher:$ROLE] Received task"

    # Parse JSON fields
    REQUEST_ID=$(echo "$PAYLOAD" | python3 -c "import sys,json; print(json.load(sys.stdin).get('request_id','?'))")
    ACTION=$(echo "$PAYLOAD" | python3 -c "import sys,json; print(json.load(sys.stdin).get('action',''))")
    TARGET_FILE=$(echo "$PAYLOAD" | python3 -c "import sys,json; print(json.load(sys.stdin).get('scope',{}).get('file',''))")
    RESULT_PATH=$(echo "$PAYLOAD" | python3 -c "import sys,json; print(json.load(sys.stdin).get('scope',{}).get('result_path',''))")

    echo "[role-watcher:$ROLE] Dispatching: request_id=$REQUEST_ID"

    # Emit task.accepted
    "$TERMLINK" emit orchestrator task.accepted \
        --payload "{\"request_id\":\"$REQUEST_ID\",\"specialist\":\"$ROLE\",\"status\":\"accepted\"}" 2>/dev/null || true

    # Build task prompt combining role system prompt + specific task
    TASK_PROMPT_FILE="$RUNTIME_DIR/prompt-$REQUEST_ID.txt"
    cat > "$TASK_PROMPT_FILE" <<PROMPT
$SYSTEM_PROMPT

---

## Your Task

Target file: $TARGET_FILE
Action: $ACTION
Write results to: $RESULT_PATH
PROMPT

    # Dispatch to fresh Claude with role-appropriate tools
    echo "[role-watcher:$ROLE] Running claude for $REQUEST_ID..."
    CLAUDE_EXIT=0
    "$CLAUDE" -p "$(cat "$TASK_PROMPT_FILE")" \
        --allowedTools "$TOOLS" \
        --dangerously-skip-permissions 2>&1 || CLAUDE_EXIT=$?

    if [ "$CLAUDE_EXIT" -eq 0 ]; then
        # Emit task.completed
        "$TERMLINK" emit orchestrator task.completed \
            --payload "{\"request_id\":\"$REQUEST_ID\",\"specialist\":\"$ROLE\",\"status\":\"completed\",\"result_path\":\"$RESULT_PATH\"}" 2>/dev/null || true
        echo "[role-watcher:$ROLE] Task $REQUEST_ID completed"
    else
        # Emit task.failed — Claude crashed or errored
        echo "[role-watcher:$ROLE] Claude failed for $REQUEST_ID (exit=$CLAUDE_EXIT)"
        "$TERMLINK" emit orchestrator task.failed \
            --payload "{\"request_id\":\"$REQUEST_ID\",\"specialist\":\"$ROLE\",\"status\":\"failed\",\"exit_code\":$CLAUDE_EXIT}" 2>/dev/null || true
        echo "[role-watcher:$ROLE] Task $REQUEST_ID failed"
    fi

    if [ -n "$SEQ" ]; then CURSOR="$SEQ"; fi
done

echo "[role-watcher:$ROLE] Exiting"
