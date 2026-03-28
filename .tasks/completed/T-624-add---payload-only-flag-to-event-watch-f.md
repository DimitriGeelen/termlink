---
id: T-624
name: "Add --payload-only flag to event watch for raw payload extraction"
description: >
  Add --payload-only flag to event watch for raw payload extraction

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:47:00Z
last_update: 2026-03-28T17:48:39Z
date_finished: 2026-03-28T17:48:39Z
---

# T-624: Add --payload-only flag to event watch for raw payload extraction

## Context

`event poll` has `--payload-only` but `event watch` doesn't. Consistency for scripting.

## Acceptance Criteria

### Agent
- [x] `--payload-only` flag added to `EventCommand::Watch` and hidden `Command::Watch` in cli.rs
- [x] `cmd_watch` outputs only payload JSON when `payload_only` is true
- [x] main.rs wires payload_only through both dispatch paths
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

### 2026-03-28T17:47:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-624-add---payload-only-flag-to-event-watch-f.md
- **Context:** Initial task creation

### 2026-03-28T17:48:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
