---
id: T-783
name: "Add missing std::error::Error impl for NegotiateError"
description: >
  Add missing std::error::Error impl for NegotiateError

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-30T06:58:43Z
last_update: 2026-03-30T06:58:43Z
date_finished: null
---

# T-783: Add missing std::error::Error impl for NegotiateError

## Context

`NegotiateError` in `termlink-protocol/src/events.rs` implements `Display` but not `std::error::Error`. All other error types in the codebase implement `Error` (most via thiserror). This prevents using `NegotiateError` with `?` in contexts expecting `Box<dyn Error>`.

## Acceptance Criteria

### Agent
- [x] `NegotiateError` implements `std::error::Error`
- [x] Test verifies `NegotiateError` can be used as `Box<dyn std::error::Error>`
- [x] All `cargo test -p termlink-protocol` tests pass

## Verification

grep -q "impl std::error::Error for NegotiateError" crates/termlink-protocol/src/events.rs
cargo test -p termlink-protocol 2>&1 | tail -3

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

### 2026-03-30T06:58:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-783-add-missing-stderrorerror-impl-for-negot.md
- **Context:** Initial task creation
