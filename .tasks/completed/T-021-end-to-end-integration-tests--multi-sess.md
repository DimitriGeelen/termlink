---
id: T-021
name: "End-to-end integration tests — multi-session communication"
description: >
  End-to-end integration tests — multi-session communication

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T17:52:58Z
last_update: 2026-03-08T18:17:19Z
date_finished: 2026-03-08T18:17:19Z
---

# T-021: End-to-end integration tests — multi-session communication

## Context

Integration tests validating the full stack built in T-015 through T-019: two sessions communicating via Unix sockets, client/server JSON-RPC, command execution, discovery, and lifecycle.

## Acceptance Criteria

### Agent
- [x] Integration test file created in termlink-session crate
- [x] Test: two sessions register, one pings the other
- [x] Test: session A executes a command on session B
- [x] Test: discovery lists both sessions
- [x] Test: deregister cleans up, session no longer discoverable
- [x] All existing + new tests pass (`cargo test --workspace`) — 83 tests

## Verification

PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace

## Updates

### 2026-03-08T17:52:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-021-end-to-end-integration-tests--multi-sess.md
- **Context:** Initial task creation

### 2026-03-08T18:17:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
