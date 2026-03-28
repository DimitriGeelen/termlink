---
id: T-567
name: "Add --timeout flag to termlink status"
description: >
  Add --timeout flag to termlink status

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:39:58Z
last_update: 2026-03-28T12:41:08Z
date_finished: 2026-03-28T12:41:08Z
---

# T-567: Add --timeout flag to termlink status

## Context

Add `--timeout` flag to `termlink status` (default 5s), same pattern as ping.

## Acceptance Criteria

### Agent
- [x] `Status` variant in cli.rs has `timeout: u64` field with default 5
- [x] `cmd_status` wraps RPC call in tokio::time::timeout
- [x] All existing tests pass

## Verification

cargo clippy -p termlink -- -D warnings 2>&1 | tail -1 | grep -qv error

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

### 2026-03-28T12:39:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-567-add---timeout-flag-to-termlink-status.md
- **Context:** Initial task creation

### 2026-03-28T12:41:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
