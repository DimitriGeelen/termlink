---
id: T-830
name: "Replace unsafe unwrap() calls with proper error handling in production code"
description: >
  Replace unsafe unwrap() calls with proper error handling in production code

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-03T21:38:15Z
last_update: 2026-04-03T21:50:47Z
date_finished: 2026-04-03T21:50:47Z
---

# T-830: Replace unsafe unwrap() calls with proper error handling in production code

## Context

Audit found unsafe `.unwrap()` in production code paths. Critical: `server.rs:552` unwraps inside an error handler (panic while handling an error). Also: lock `.unwrap()` calls across hub/session crates lack context messages.

## Acceptance Criteria

### Agent
- [x] hub/server.rs error handler uses hardcoded fallback JSON instead of unwrap
- [x] Lock .unwrap() calls in tofu.rs replaced with .expect("context")
- [x] Lock .unwrap() calls in remote_store.rs replaced with .expect("context")
- [x] Lock .unwrap() calls in circuit_breaker.rs replaced with .expect("context")
- [x] All 695 existing tests pass
- [x] Zero clippy warnings

## Verification

cargo test --workspace 2>&1 | tail -5
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c '^warning\[')" = "0"

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

### 2026-04-03T21:38:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-830-replace-unsafe-unwrap-calls-with-proper-.md
- **Context:** Initial task creation

### 2026-04-03T21:50:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
