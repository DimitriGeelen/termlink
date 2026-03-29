---
id: T-733
name: "Add ok:true to remote exec and local exec JSON success responses"
description: >
  Add ok:true to remote exec and local exec JSON success responses

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-29T12:01:32Z
last_update: 2026-03-29T12:03:08Z
date_finished: 2026-03-29T12:03:08Z
---

# T-733: Add ok:true to remote exec and local exec JSON success responses

## Context

`exec --json` and `remote exec --json` pass through bare RPC results without adding `"ok": true` wrapper.

## Acceptance Criteria

### Agent
- [x] `exec --json` success wraps result with `"ok": true`
- [x] `remote exec --json` success wraps result with `"ok": true`
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

### 2026-03-29T12:01:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-733-add-oktrue-to-remote-exec-and-local-exec.md
- **Context:** Initial task creation

### 2026-03-29T12:03:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
