---
id: T-1007
name: "Add router tests for session.update and event.poll RPC methods"
description: >
  Add router tests for session.update and event.poll RPC methods

status: work-completed
workflow_type: test
owner: human
horizon: now
tags: []
components: [crates/termlink-hub/src/router.rs]
related_tasks: []
created: 2026-04-13T09:19:15Z
last_update: 2026-04-13T09:22:37Z
date_finished: 2026-04-13T09:22:37Z
---

# T-1007: Add router tests for session.update and event.poll RPC methods

## Context

Add error-path and edge-case router tests for hub RPC methods that currently lack coverage: heartbeat param validation, deregister error paths, register param validation, and hub-level event.subscribe basic structure.

## Acceptance Criteria

### Agent
- [x] Add heartbeat_missing_id_returns_error test
- [x] Add heartbeat_nonexistent_session_returns_error test
- [x] Add deregister_remote_missing_id_returns_error test
- [x] Add deregister_remote_nonexistent_returns_error test
- [x] Add register_remote_missing_host_returns_error test
- [x] Add register_remote_missing_port_returns_error test
- [x] Add hub_subscribe_returns_events_structure test
- [x] All tests pass: cargo test -p termlink-hub (198 passed)

### Human
- [ ] [RUBBER-STAMP] Verify test count increased in Watchtower
  **Steps:**
  1. `cd /opt/termlink && cargo test -p termlink-hub 2>&1 | grep "test result"`
  **Expected:** More tests than before (was 35+12=47 router tests)
  **If not:** Check if tests were added to wrong module

## Verification

cargo test -p termlink-hub 2>&1 | grep "test result"

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

### 2026-04-13T09:19:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1007-add-router-tests-for-sessionupdate-and-e.md
- **Context:** Initial task creation

### 2026-04-13T09:22:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
