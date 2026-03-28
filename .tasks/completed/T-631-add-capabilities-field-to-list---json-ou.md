---
id: T-631
name: "Add capabilities field to list --json output"
description: >
  Add capabilities field to list --json output

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T18:00:35Z
last_update: 2026-03-28T18:01:24Z
date_finished: 2026-03-28T18:01:24Z
---

# T-631: Add capabilities field to list --json output

## Context

`list --json` outputs tags and roles but not capabilities. `discover --json` includes all three.

## Acceptance Criteria

### Agent
- [x] `capabilities` field added to `list --json` output
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

### 2026-03-28T18:00:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-631-add-capabilities-field-to-list---json-ou.md
- **Context:** Initial task creation

### 2026-03-28T18:01:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
