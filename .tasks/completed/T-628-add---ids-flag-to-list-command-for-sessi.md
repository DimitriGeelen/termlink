---
id: T-628
name: "Add --ids flag to list command for session ID output"
description: >
  Add --ids flag to list command for session ID output

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:55:21Z
last_update: 2026-03-28T17:57:20Z
date_finished: 2026-03-28T17:57:20Z
---

# T-628: Add --ids flag to list command for session ID output

## Context

`list --names` outputs display names but there's no way to get just IDs. `discover` has `--id` already.

## Acceptance Criteria

### Agent
- [x] `--ids` flag added to `Command::List` in cli.rs
- [x] `cmd_list` outputs one session ID per line when `ids` is true
- [x] main.rs wires the ids parameter
- [x] `cargo check -p termlink` passes

## Verification

cargo check -p termlink 2>&1 | tail -1

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

### 2026-03-28T17:55:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-628-add---ids-flag-to-list-command-for-sessi.md
- **Context:** Initial task creation

### 2026-03-28T17:57:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
