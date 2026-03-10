#!/usr/bin/env bash
# Persistent specialist watcher — polls for task.delegate events and dispatches to claude -p
# Usage: specialist-watcher.sh <termlink-binary> <runtime-dir> <claude-binary>
set -uo pipefail

TERMLINK="$1"
RUNTIME_DIR="$2"
CLAUDE="$3"

export TERMLINK_RUNTIME_DIR="$RUNTIME_DIR"
unset CLAUDECODE 2>/dev/null || true

echo "[watcher] Started (pid=$$)"
echo $$ > "$RUNTIME_DIR/watcher.pid"

CURSOR=""

while true; do
    # Check shutdown
    if [ -f "$RUNTIME_DIR/shutdown" ]; then
        echo "[watcher] Shutdown signal"; break
    fi

    # Poll for task.delegate events after cursor
    if [ -z "$CURSOR" ]; then
        RAW=$("$TERMLINK" events specialist --topic task.delegate 2>/dev/null || echo "")
    else
        RAW=$("$TERMLINK" events specialist --topic task.delegate --since "$CURSOR" 2>/dev/null || echo "")
    fi

    # Skip if no new events
    if [ -z "$RAW" ] || echo "$RAW" | grep -q "No events"; then
        sleep 2; continue
    fi

    # Extract first event line: [N] task.delegate: {...} (t=xxx)
    EVENT_LINE=$(echo "$RAW" | grep '^\[' | head -1)
    if [ -z "$EVENT_LINE" ]; then sleep 2; continue; fi

    # Extract seq number and payload
    SEQ=$(echo "$EVENT_LINE" | sed -n 's/^\[\([0-9]*\)\].*/\1/p')
    PAYLOAD=$(echo "$EVENT_LINE" | sed 's/^\[[0-9]*\] task\.delegate: //' | sed 's/ (t=[0-9]*)$//')

    echo "[watcher] Received: $PAYLOAD"

    # Parse JSON fields
    REQUEST_ID=$(echo "$PAYLOAD" | python3 -c "import sys,json; print(json.load(sys.stdin).get('request_id','?'))")
    ACTION=$(echo "$PAYLOAD" | python3 -c "import sys,json; print(json.load(sys.stdin).get('action',''))")
    TARGET_FILE=$(echo "$PAYLOAD" | python3 -c "import sys,json; print(json.load(sys.stdin).get('scope',{}).get('file',''))")
    RESULT_PATH=$(echo "$PAYLOAD" | python3 -c "import sys,json; print(json.load(sys.stdin).get('scope',{}).get('result_path',''))")

    echo "[watcher] Dispatching: request_id=$REQUEST_ID"

    # Emit task.accepted
    "$TERMLINK" emit orchestrator task.accepted \
        --payload "{\"request_id\":\"$REQUEST_ID\",\"status\":\"accepted\"}" 2>/dev/null || true

    # Build prompt file
    TASK_PROMPT_FILE="$RUNTIME_DIR/prompt-$REQUEST_ID.txt"
    cat > "$TASK_PROMPT_FILE" <<PROMPT
You are a code analysis specialist. Do these steps:
1. Read this file: $TARGET_FILE
2. $ACTION
3. Write your analysis to: $RESULT_PATH
Be concise (5-10 lines). Use the Read tool and Bash tool.
PROMPT

    # Dispatch to fresh Claude (fresh context window!)
    echo "[watcher] Running claude for $REQUEST_ID..."
    "$CLAUDE" -p "$(cat "$TASK_PROMPT_FILE")" \
        --allowedTools "Bash,Read,Write" \
        --dangerously-skip-permissions 2>&1 || echo "[watcher] claude exit: $?"

    # Emit task.completed
    "$TERMLINK" emit orchestrator task.completed \
        --payload "{\"request_id\":\"$REQUEST_ID\",\"status\":\"completed\",\"result_path\":\"$RESULT_PATH\"}" 2>/dev/null || true

    echo "[watcher] Task $REQUEST_ID done"

    # Update cursor: use the seq of the event we just processed + 1
    # (--since is exclusive: returns events with seq > CURSOR)
    if [ -n "$SEQ" ]; then CURSOR="$SEQ"; fi
done

echo "[watcher] Exiting"
