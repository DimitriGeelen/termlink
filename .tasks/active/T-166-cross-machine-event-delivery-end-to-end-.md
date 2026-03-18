---
id: T-166
name: "Cross-machine event delivery end-to-end test"
description: >
  Validate that event.emit on machine A reaches machine B session via TCP hub. Test broadcast and collect across TCP.

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: [tcp, events]
components: []
related_tasks: []
created: 2026-03-18T10:08:34Z
last_update: 2026-03-18T16:04:09Z
date_finished: null
---

# T-166: Cross-machine event delivery end-to-end test

## Context

Validate that event.broadcast via TCP hub reaches sessions, and event.collect aggregates events from sessions reached via TCP. End-to-end test for T-164 TCP auth + T-163 cross-machine communication.

## Acceptance Criteria

### Agent
- [x] Test: TCP-authenticated client can broadcast events to local sessions via hub
- [x] Test: TCP-authenticated client can collect events from local sessions via hub
- [x] Test: Unauthenticated TCP client cannot broadcast or collect (scope enforcement)
- [x] All existing hub tests still pass (`cargo test -p termlink-hub`)

## Verification

bash -c 'out=$(/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub 2>&1); echo "$out" | grep -q "0 failed"'
grep -q "tcp_broadcast_delivers_to_sessions" crates/termlink-hub/src/router.rs
grep -q "tcp_collect_aggregates_events" crates/termlink-hub/src/router.rs

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

### 2026-03-18T10:08:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-166-cross-machine-event-delivery-end-to-end-.md
- **Context:** Initial task creation

### 2026-03-18T16:04:09Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
