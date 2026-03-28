---
id: T-621
name: "Add --timeout flag to event watch for time-bounded watching"
description: >
  Add --timeout flag to event watch for time-bounded watching

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:40:52Z
last_update: 2026-03-28T17:42:26Z
date_finished: 2026-03-28T17:42:26Z
---

# T-621: Add --timeout flag to event watch for time-bounded watching

## Context

The `event watch` command currently only exits on Ctrl+C. Adding `--timeout` enables time-bounded watching for scripting.

## Acceptance Criteria

### Agent
- [x] `--timeout N` flag added to `EventCommand::Watch` and hidden `Command::Watch` in cli.rs
- [x] `cmd_watch` in events.rs accepts timeout parameter and exits after N seconds
- [x] `--timeout 0` (default) means no timeout (existing behavior preserved)
- [x] main.rs wires the timeout parameter through both dispatch paths
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

### 2026-03-28T17:40:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-621-add---timeout-flag-to-event-watch-for-ti.md
- **Context:** Initial task creation

### 2026-03-28T17:42:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
