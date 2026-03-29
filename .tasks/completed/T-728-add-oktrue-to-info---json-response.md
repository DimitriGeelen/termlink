---
id: T-728
name: "Add ok:true to info --json response"
description: >
  Add ok:true to info --json response

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-29T11:45:56Z
last_update: 2026-03-29T11:46:42Z
date_finished: 2026-03-29T11:46:42Z
---

# T-728: Add ok:true to info --json response

## Context

`info --json` outputs runtime info without `"ok": true`.

## Acceptance Criteria

### Agent
- [x] `info --json` includes `"ok": true` in response
- [x] Project compiles with `cargo check`

## Verification

cargo check 2>&1 | grep -q 'Finished'

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

### 2026-03-29T11:45:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-728-add-oktrue-to-info---json-response.md
- **Context:** Initial task creation

### 2026-03-29T11:46:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
