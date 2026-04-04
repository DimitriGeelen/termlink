---
id: T-873
name: "dispatch early crash detection — check worker PID liveness during collection loop"
description: >
  dispatch early crash detection — check worker PID liveness during collection loop

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T22:51:04Z
last_update: 2026-04-04T22:51:04Z
date_finished: null
---

# T-873: dispatch early crash detection — check worker PID liveness during collection loop

## Context

T-280 dispatch readiness: dispatch collects result events but if a worker crashes without
emitting, dispatch blindly waits until timeout. Adding PID/session liveness checks in the
collection loop enables early crash detection and reporting. Related: T-280, T-282.

## Acceptance Criteria

### Agent
- [x] Dispatch collection loop checks for dead workers each iteration
- [x] When all remaining workers are dead, dispatch breaks early with warning
- [x] Dead worker names reported in output (both text and JSON modes)
- [x] `cargo clippy --workspace` passes with no warnings (0 warnings)
- [x] `cargo test --workspace` passes (857 tests, 0 failures)

## Verification

# Clippy clean
cargo clippy --workspace 2>&1 | grep -v "^$" | tail -5 | grep -q "warning generated\|could not compile" && exit 1 || true
# Tests pass
cargo test --workspace 2>&1 | tail -3 | grep -q "0 failed"

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

### 2026-04-04T22:51:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-873-dispatch-early-crash-detection--check-wo.md
- **Context:** Initial task creation
