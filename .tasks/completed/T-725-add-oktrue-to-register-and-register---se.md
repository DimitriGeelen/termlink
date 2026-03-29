---
id: T-725
name: "Add ok:true to register and register --self JSON responses"
description: >
  Add ok:true to register and register --self JSON responses

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-29T11:41:35Z
last_update: 2026-03-29T11:42:28Z
date_finished: 2026-03-29T11:42:28Z
---

# T-725: Add ok:true to register and register --self JSON responses

## Context

`register --json` and `register --self --json` output session details without `"ok": true`.

## Acceptance Criteria

### Agent
- [x] `register --json` includes `"ok": true` in startup response
- [x] `register --self --json` includes `"ok": true` in startup response
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

### 2026-03-29T11:41:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-725-add-oktrue-to-register-and-register---se.md
- **Context:** Initial task creation

### 2026-03-29T11:42:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
