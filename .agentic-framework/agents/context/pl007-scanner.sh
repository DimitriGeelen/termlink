#!/bin/bash
# PL-007 Scanner — PostToolUse hook for bare command pattern detection (T-976)
#
# Scans Bash tool output for bare command patterns that the agent might
# relay to the user. When detected, injects a PL-007 reminder via
# additionalContext telling the agent to use fw task review / termlink
# inject instead of outputting bare commands.
#
# This is a MITIGATION for T-972 RC-2 (no agent text governance).
# It cannot prevent the agent from outputting bare commands, but it can
# remind the agent immediately after seeing them in tool output.
#
# Exit code: always 0 (PostToolUse hooks are advisory, cannot block)
# Output: JSON with additionalContext when patterns detected, empty otherwise
#
# Part of: Agentic Engineering Framework
# Created by: T-976 (from T-972 RC-2)

set -uo pipefail

# Read stdin JSON from Claude Code
INPUT=$(cat)

echo "$INPUT" | python3 -c "
import sys, json

try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

# Only check Bash tool calls
if data.get('tool_name') != 'Bash':
    sys.exit(0)

response = data.get('tool_response', {})
if not isinstance(response, dict):
    sys.exit(0)

stdout = str(response.get('stdout', ''))
stderr = str(response.get('stderr', ''))
output = stdout + '\n' + stderr

# Patterns that indicate bare command suggestions in tool output.
# These are commands the agent should NOT relay to the user.
PATTERNS = [
    ('fw inception decide', 'Use fw task review T-XXX instead — it opens Watchtower with one-click GO/NO-GO.'),
    ('fw tier0 approve', 'The approval link is already shown in Watchtower. Do not tell the user to run this.'),
    ('bin/fw inception decide', 'Use fw task review T-XXX instead — it opens Watchtower with one-click GO/NO-GO.'),
    ('bin/fw tier0 approve', 'The approval link is already shown in Watchtower. Do not tell the user to run this.'),
]

# Skip if this is the agent running fw task review (the review output itself
# contains the command — that's expected, the issue is the agent relaying it)
tool_input = data.get('tool_input', {})
command = ''
if isinstance(tool_input, dict):
    command = str(tool_input.get('command', ''))
elif isinstance(tool_input, str):
    command = tool_input

# If the agent is already running fw task review, don't warn about its output
if 'fw task review' in command or 'task review' in command:
    sys.exit(0)

# If the agent is running fw inception decide, don't warn (they're doing the right thing)
if 'fw inception decide' in command:
    sys.exit(0)

# Check for patterns
matched = []
for pattern, advice in PATTERNS:
    if pattern in output.lower() or pattern in output:
        matched.append(advice)

if matched:
    # Deduplicate
    unique_advice = list(dict.fromkeys(matched))
    reminder = 'PL-007 REMINDER: The tool output contains bare command suggestions. Do NOT relay these to the user. ' + ' '.join(unique_advice)
    result = {'additionalContext': reminder}
    print(json.dumps(result))
" 2>/dev/null || true
