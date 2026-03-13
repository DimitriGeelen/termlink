---
id: T-120
name: "EventBus gap detection + concurrent hub broadcast"
description: >
  Fix EventBus silent event loss (gap detection when cursor < oldest_seq) and
  make hub broadcast concurrent instead of sequential. From T-009 inception GO.
status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [concurrency, eventbus, hub, backpressure]
components: []
related_tasks: [T-009]
created: 2026-03-12T20:17:23Z
last_update: 2026-03-12T21:29:40Z
date_finished: 2026-03-12T21:29:40Z
---

# T-120: EventBus gap detection + concurrent hub broadcast

## Context

From T-009 inception (docs/reports/T-009-exploration.md). EventBus ring buffer silently
evicts oldest events on overflow — pollers get no notification. Hub broadcast iterates
sessions sequentially — one dead session stalls all.

## Acceptance Criteria

### Agent
- [x] EventBus detects gap when poller cursor < oldest sequence in buffer
- [x] Gap detection returns a warning/error in the events response (not silent)
- [x] Hub broadcast dispatches to sessions concurrently (tokio::spawn per target, not sequential loop)
- [x] Hub broadcast has per-target timeout (not relying on default socket timeout)
- [x] Existing event ordering tests still pass
- [x] New test: concurrent pollers on one session see all events without loss

## Verification

# Rust tests pass
/Users/dimidev32/.cargo/bin/cargo test -p termlink-session --lib 2>&1 | grep -q "test result: ok"
/Users/dimidev32/.cargo/bin/cargo test -p termlink-hub --lib 2>&1 | grep -q "test result: ok"

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

### 2026-03-12T20:17:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-120-eventbus-gap-detection--concurrent-hub-b.md
- **Context:** Initial task creation

### 2026-03-12T21:29:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
