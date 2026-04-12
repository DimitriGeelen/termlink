---
id: T-989
name: "send-file hub fallback — route through hub when target not found locally (enables inbox)"
description: >
  send-file hub fallback — route through hub when target not found locally (enables inbox)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/file.rs]
related_tasks: []
created: 2026-04-12T21:59:27Z
last_update: 2026-04-12T22:04:23Z
date_finished: 2026-04-12T22:04:23Z
---

# T-989: send-file hub fallback — route through hub when target not found locally (enables inbox)

## Context

Build task from T-946/T-951 GO decisions + T-988 (hub inbox). Currently `send-file` resolves the
target session locally via `manager::find_session()`. If the target is offline or on a different
machine, it fails. With T-988's hub inbox, the hub can spool files for offline sessions. This task
adds a fallback: when local lookup fails, route the file events through the hub's `event.emit_to`,
which triggers inbox spooling for offline targets.

## Acceptance Criteria

### Agent
- [x] `cmd_file_send` falls back to hub `event.emit_to` when `find_session` fails
- [x] Hub fallback sends file.init, file.chunk, file.complete via hub socket
- [x] Response distinguishes direct delivery vs hub-spooled (`via` field in JSON output)
- [x] When neither local session nor hub is available, error message is clear
- [x] All existing CLI tests pass — 165/165 + 83/83 integration

## Verification

cargo test -p termlink
cargo clippy -p termlink -- -D warnings

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

### 2026-04-12T21:59:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-989-send-file-hub-fallback--route-through-hu.md
- **Context:** Initial task creation

### 2026-04-12T22:04:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Hub fallback implemented, 165+83 tests pass
