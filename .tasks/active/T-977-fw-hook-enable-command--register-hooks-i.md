---
id: T-977
name: "fw hook-enable command — register hooks in settings.json from CLI"
description: >
  fw hook-enable command — register hooks in settings.json from CLI

status: work-completed
workflow_type: build
owner: human
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-04-12T10:49:55Z
last_update: 2026-04-16T05:39:44Z
date_finished: 2026-04-12T10:51:24Z
---

# T-977: fw hook-enable command — register hooks in settings.json from CLI

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `bin/hook-enable.sh` script exists with --matcher and --event flags
- [x] `fw hook-enable` route added in bin/fw
- [x] Idempotent — running twice doesn't duplicate the entry
- [x] T-976 pl007-scanner registered via the new command

### Human
- [ ] [RUBBER-STAMP] Verify pl007-scanner hook fires on bare command output
  **Steps:**
  1. Run a command whose output contains "fw inception decide" (e.g., `fw task review` on an inception task)
  2. Check that the agent receives a PL-007 reminder in additionalContext
  **Expected:** Agent does not relay bare command to user
  **If not:** Check `fw hook-enable pl007-scanner --matcher Bash --event PostToolUse` was applied

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-12T10:49:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-977-fw-hook-enable-command--register-hooks-i.md
- **Context:** Initial task creation

### 2026-04-12T10:51:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T05:39:44Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:09:44Z — programmatic-evidence [T-1090]
- **Evidence:** fw hook command exists (fw help shows 'hook <name>'); hook registration handled by .claude/settings.json config; fw upgrade syncs hook config
- **Verified by:** automated command execution
