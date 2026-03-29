---
id: T-731
name: "Add ok:true to dispatch and request JSON success responses"
description: >
  Add ok:true to dispatch and request JSON success responses

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/dispatch.rs, crates/termlink-cli/src/commands/execution.rs]
related_tasks: []
created: 2026-03-29T11:50:47Z
last_update: 2026-03-29T11:51:44Z
date_finished: 2026-03-29T11:51:44Z
---

# T-731: Add ok:true to dispatch and request JSON success responses

## Context

`dispatch --json` and `request --json` success responses missing `"ok": true`.

## Acceptance Criteria

### Agent
- [x] `dispatch --json` success includes `"ok": true`
- [x] `request --json` reply includes `"ok": true`
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

### 2026-03-29T11:50:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-731-add-oktrue-to-dispatch-and-request-json-.md
- **Context:** Initial task creation

### 2026-03-29T11:51:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
