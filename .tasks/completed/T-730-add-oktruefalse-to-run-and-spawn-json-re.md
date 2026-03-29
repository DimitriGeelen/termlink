---
id: T-730
name: "Add ok:true/false to run and spawn JSON responses"
description: >
  Add ok:true/false to run and spawn JSON responses

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/execution.rs]
related_tasks: []
created: 2026-03-29T11:48:56Z
last_update: 2026-03-29T11:50:16Z
date_finished: 2026-03-29T11:50:16Z
---

# T-730: Add ok:true/false to run and spawn JSON responses

## Context

`run --json` and `spawn --json` have multiple response paths missing `"ok": true/false`.

## Acceptance Criteria

### Agent
- [x] `run --json` success includes `"ok": true`
- [x] `run --json` error includes `"ok": false`
- [x] `spawn --json` no-wait success includes `"ok": true`
- [x] `spawn --json` wait success includes `"ok": true`
- [x] `spawn --json` wait timeout includes `"ok": false`
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

### 2026-03-29T11:48:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-730-add-oktruefalse-to-run-and-spawn-json-re.md
- **Context:** Initial task creation

### 2026-03-29T11:50:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
