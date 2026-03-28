---
id: T-632
name: "Add socket_path field to list and discover --json output"
description: >
  Add socket_path field to list and discover --json output

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T18:03:26Z
last_update: 2026-03-28T18:04:25Z
date_finished: 2026-03-28T18:04:25Z
---

# T-632: Add socket_path field to list and discover --json output

## Context

JSON output from list and discover doesn't include socket_path, which is useful for direct RPC connections in scripts.

## Acceptance Criteria

### Agent
- [x] `socket_path` field added to `list --json` output
- [x] `socket_path` field added to `discover --json` output
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

### 2026-03-28T18:03:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-632-add-socketpath-field-to-list-and-discove.md
- **Context:** Initial task creation

### 2026-03-28T18:04:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
