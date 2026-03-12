#!/usr/bin/env bash
# Pre-compact hook: extract tool calls from current session and append to telemetry store.
# Non-blocking: failures are logged to stderr but do not prevent compaction.
# Task: T-112 | Design: T-104

set -euo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(pwd)}"
TELEMETRY_DIR="${PROJECT_ROOT}/.context/telemetry"
STORE="${TELEMETRY_DIR}/tool-calls.jsonl"
EXTRACTOR="${PROJECT_ROOT}/agents/telemetry/extract-tool-calls.py"

# Read current task from focus.yaml
TASK=""
FOCUS_FILE="${PROJECT_ROOT}/.context/working/focus.yaml"
if [[ -f "$FOCUS_FILE" ]]; then
    TASK=$(grep -m1 '^current_task:' "$FOCUS_FILE" | awk '{print $2}' || true)
    [[ "$TASK" == "null" ]] && TASK=""
fi

# Ensure telemetry directory exists
mkdir -p "$TELEMETRY_DIR"

# Run extractor (non-blocking — trap errors)
if [[ ! -x "$EXTRACTOR" ]]; then
    echo "WARN: extractor not found at $EXTRACTOR" >&2
    exit 0
fi

ARGS=(--include-sidechains)
[[ -n "$TASK" ]] && ARGS+=(--task "$TASK")

# Extract and append (stdout → store, stderr → /dev/null to avoid polluting store)
BEFORE=0
[[ -f "$STORE" ]] && BEFORE=$(wc -l < "$STORE")

if python3 "$EXTRACTOR" "${ARGS[@]}" >> "$STORE" 2>/dev/null; then
    AFTER=$(wc -l < "$STORE")
    ADDED=$(( AFTER - BEFORE ))
    echo "Telemetry: ${ADDED} tool call records captured" >&2
else
    echo "WARN: tool call extraction failed (non-blocking)" >&2
fi

exit 0
