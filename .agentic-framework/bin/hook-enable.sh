#!/bin/bash
# hook-enable.sh — Register a framework hook in .claude/settings.json
#
# Usage:
#   fw hook-enable <hook-name> [--matcher PATTERN] [--event PreToolUse|PostToolUse]
#
# Examples:
#   fw hook-enable pl007-scanner --matcher Bash --event PostToolUse
#   fw hook-enable my-custom-hook --matcher "Write|Edit" --event PreToolUse
#
# Default: --matcher "" (all tools), --event PostToolUse
# Created by: T-977 (from T-976)

set -euo pipefail

HOOK_NAME="${1:-}"
MATCHER=""
EVENT="PostToolUse"

shift 2>/dev/null || true
while [ $# -gt 0 ]; do
    case "$1" in
        --matcher) MATCHER="$2"; shift 2 ;;
        --event)   EVENT="$2"; shift 2 ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

if [ -z "$HOOK_NAME" ]; then
    echo "Usage: fw hook-enable <hook-name> [--matcher PATTERN] [--event PreToolUse|PostToolUse]" >&2
    exit 1
fi

# Find settings.json
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
SETTINGS="$PROJECT_ROOT/.claude/settings.json"

if [ ! -f "$SETTINGS" ]; then
    echo "ERROR: $SETTINGS not found" >&2
    exit 1
fi

# Verify hook script exists
HOOK_SCRIPT="$PROJECT_ROOT/.agentic-framework/agents/context/${HOOK_NAME}.sh"
if [ ! -f "$HOOK_SCRIPT" ]; then
    echo "ERROR: Hook script not found: $HOOK_SCRIPT" >&2
    exit 1
fi

# Check if already registered
if grep -q "$HOOK_NAME" "$SETTINGS" 2>/dev/null; then
    echo "Already registered: $HOOK_NAME"
    exit 0
fi

# Add hook entry
python3 -c "
import json

with open('$SETTINGS') as f:
    data = json.load(f)

hooks = data.setdefault('hooks', {}).setdefault('$EVENT', [])
entry = {
    'matcher': '$MATCHER',
    'hooks': [{'type': 'command', 'command': '.agentic-framework/bin/fw hook $HOOK_NAME'}]
}
hooks.append(entry)

with open('$SETTINGS', 'w') as f:
    json.dump(data, f, indent=2)
    f.write('\n')
"
echo "Registered: $HOOK_NAME ($EVENT, matcher=${MATCHER:-(all)})"
