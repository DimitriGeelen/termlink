---
id: T-036
name: "Session event system — publish/subscribe for cross-session messaging"
description: >
  Session event system — publish/subscribe for cross-session messaging

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T20:54:29Z
last_update: 2026-03-08T20:54:29Z
date_finished: null
---

# T-036: Session event system — publish/subscribe for cross-session messaging

## Context

Session-level event bus for structured cross-session messaging. Each session maintains an event ring buffer with sequence numbers. Events have topic + JSON payload. Core message bus capability from paradigm decision (T-003).

## Acceptance Criteria

### Agent
- [x] `EventBus` struct with ring buffer, sequence numbers, emit/poll
- [x] `event.emit` RPC handler: topic + payload → stored event with seq
- [x] `event.poll` RPC handler: since_seq → returns new events
- [x] `event.topics` RPC handler: list distinct topics
- [x] EventBus integrated into SessionContext
- [x] CLI `events` subcommand to poll events from a session
- [x] CLI `emit` subcommand to emit an event to a session
- [x] Unit tests for EventBus (emit, poll, overflow) — 6 tests
- [x] Handler tests for RPC methods — 4 tests
- [x] All 126 tests pass, builds without warnings

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | tail -5

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

### 2026-03-08T20:54:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-036-session-event-system--publishsubscribe-f.md
- **Context:** Initial task creation
