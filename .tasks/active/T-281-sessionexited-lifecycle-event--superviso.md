---
id: T-281
name: "session.exited lifecycle event — supervisor emits before cleanup"
description: >
  Hub supervisor emits session.exited event before removing dead sessions. Crash safety net for dispatch orchestration. ~80 LOC across supervisor.rs, router.rs, protocol.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [dispatch, lifecycle, T-280]
components: []
related_tasks: [T-280, T-257, T-256]
created: 2026-03-25T15:08:31Z
last_update: 2026-03-25T15:08:31Z
date_finished: null
---

# T-281: session.exited lifecycle event — supervisor emits before cleanup

## Context

From T-280 inception (GO). When a session's PID dies, TermLink emits nothing — hub supervisor
silently removes dead sessions. This makes dispatch orchestration unreliable: crashed workers
produce no signal, and orchestrators wait forever. Fix: supervisor emits `session.exited` event
before cleanup, giving orchestrators a reliable crash-detection signal.

## Acceptance Criteria

### Agent
- [x] Hub supervisor emits `session.exited` event (with session_id, display_name, pid) before removing dead sessions
- [x] Event is broadcast to all live sessions via hub router (fan-out)
- [x] Event includes exit reason field distinguishing clean exit vs process death
- [x] Integration test: spawn session, kill its PID, verify `session.exited` event appears on another session's event bus
- [x] Integration test: verify no event emitted for sessions that are still alive
- [x] All existing tests pass (0 regressions)
- [x] `cargo test --workspace` passes with 0 warnings

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace

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

### 2026-03-25T15:08:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-281-sessionexited-lifecycle-event--superviso.md
- **Context:** Initial task creation
