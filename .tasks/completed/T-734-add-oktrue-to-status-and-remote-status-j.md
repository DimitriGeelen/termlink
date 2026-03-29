---
id: T-734
name: "Add ok:true to status and remote status JSON success responses"
description: >
  Add ok:true to status and remote status JSON success responses

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-29T12:03:35Z
last_update: 2026-03-29T12:04:38Z
date_finished: 2026-03-29T12:04:38Z
---

# T-734: Add ok:true to status and remote status JSON success responses

## Context

`status --json` and `remote status --json` pass through bare RPC results without `"ok": true`.

## Acceptance Criteria

### Agent
- [x] `status --json` success wraps result with `"ok": true`
- [x] `remote status --json` success wraps result with `"ok": true`
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

### 2026-03-29T12:03:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-734-add-oktrue-to-status-and-remote-status-j.md
- **Context:** Initial task creation

### 2026-03-29T12:04:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
