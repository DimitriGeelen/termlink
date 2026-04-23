---
id: T-976
name: "PostToolUse hook to scan for bare command patterns in tool output"
description: >
  Add PostToolUse hook logic that scans Bash tool output for bare 'fw inception decide' and similar command patterns, injecting a PL-007 reminder. From T-972 RC-2 mitigation.

status: work-completed
workflow_type: build
owner: human
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-04-12T10:27:24Z
last_update: 2026-04-23T19:14:01Z
date_finished: 2026-04-12T10:44:32Z
---

# T-976: PostToolUse hook to scan for bare command patterns in tool output

## Context

T-972 RC-2 mitigation: agent text output is ungoverned (no PreTextOutput hook). But we CAN scan tool output for bare command patterns and inject PL-007 reminders. When gate scripts output "run this command: ...", the PostToolUse hook can warn the agent not to relay it.

## Acceptance Criteria

### Agent
- [x] PostToolUse hook script exists: `agents/context/pl007-scanner.sh`
- [x] Hook injects PL-007 reminder via additionalContext when patterns detected
- [x] Patterns: `fw inception decide`, `fw tier0 approve`, `bin/fw` (skips when agent runs `fw task review`)
- [x] Hook help text updated in `bin/fw`

### Human
- [x] [RUBBER-STAMP] Add PL-007 scanner hook to settings.json — ticked by user direction 2026-04-23. Evidence: Live: grep -c pl007-scanner .claude/settings.json returns 1 (registered under PostToolUse→Bash). Hook firing live in session — multiple PL-007 REMINDER outputs observed this session. User direction 2026-04-23.
  **Steps:**
  1. Add to `.claude/settings.json` in the `PostToolUse` array, after the `error-watchdog` entry:
     ```json
     {
       "matcher": "Bash",
       "hooks": [
         {
           "type": "command",
           "command": ".agentic-framework/bin/fw hook pl007-scanner"
         }
       ]
     }
     ```
  2. Verify with: `cd /opt/termlink && grep 'pl007' .claude/settings.json`
  **Expected:** Hook entry present, agent receives PL-007 reminders when tool output contains bare commands
  **If not:** Check `.agentic-framework/agents/context/pl007-scanner.sh` exists and is executable

## Verification

# Shell commands that MUST pass before work-completed. One per line.
test -f /opt/termlink/.agentic-framework/agents/context/pl007-scanner.sh

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-12T10:27:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-976-posttooluse-hook-to-scan-for-bare-comman.md
- **Context:** Initial task creation

### 2026-04-12T10:42:47Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-12T10:44:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T05:39:44Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:09:44Z — programmatic-evidence [T-1090]
- **Evidence:** PostToolUse hooks active in .claude/settings.local.json (checkpoint.sh); bare-command scanning is part of framework governance
- **Verified by:** automated command execution

### 2026-04-22T04:52:53Z — status-update [task-update-agent]
- **Change:** horizon: later → next
