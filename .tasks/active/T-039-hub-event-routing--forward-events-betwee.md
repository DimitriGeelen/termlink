---
id: T-039
name: "Hub event routing — forward events between sessions"
description: >
  Hub event routing — forward events between sessions

status: started-work
workflow_type: build
owner: claude-code
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T21:15:36Z
last_update: 2026-03-08T21:15:36Z
date_finished: null
---

# T-039: Hub event routing — forward events between sessions

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Hub handles `event.broadcast` — fan-out emit to multiple sessions
- [x] Hub handles `event.collect` — fan-in poll from multiple sessions with cursor tracking
- [x] `event.broadcast` supports optional `targets` filter
- [x] `event.collect` supports `since` cursors and `topic` filter
- [x] CLI `broadcast` command added
- [x] 5 new hub tests (broadcast, broadcast-filtered, collect, collect-cursors, broadcast-error)
- [x] All 131 tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-hub 2>&1 | grep -q "12 passed"
grep -q "event.broadcast" crates/termlink-hub/src/router.rs
grep -q "event.collect" crates/termlink-hub/src/router.rs
grep -q "Broadcast" crates/termlink-cli/src/main.rs

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

### 2026-03-08T21:15:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-039-hub-event-routing--forward-events-betwee.md
- **Context:** Initial task creation
