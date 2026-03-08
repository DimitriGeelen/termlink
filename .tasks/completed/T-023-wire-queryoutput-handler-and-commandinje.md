---
id: T-023
name: "Wire query.output handler and command.inject to PTY sessions"
description: >
  Wire query.output handler and command.inject to PTY sessions

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T18:48:37Z
last_update: 2026-03-08T18:54:58Z
date_finished: 2026-03-08T18:54:58Z
---

# T-023: Wire query.output handler and command.inject to PTY sessions

## Context

Wire PTY sessions into the handler/server stack. Add `query.output` handler, connect `command.inject` to PTY write, and add CLI `register --shell` mode. Follows T-022 (PTY manager) and T-007 GO decision.

## Acceptance Criteria

### Agent
- [x] `query.output` handler returns scrollback snapshot (last N lines or bytes)
- [x] `command.inject` writes to PTY master when PTY session is active
- [x] CLI `register --shell` starts a PTY-backed session
- [x] Server holds optional PtySession in shared state via SessionContext
- [x] Tests: query.output returns output, inject writes to PTY, PTY session end-to-end
- [x] All tests pass (`cargo test --workspace`) — 99 tests

## Verification

PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace

## Updates

### 2026-03-08T18:48:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-023-wire-queryoutput-handler-and-commandinje.md
- **Context:** Initial task creation

### 2026-03-08T18:54:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
