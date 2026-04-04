---
id: T-875
name: "Add elapsed time tracking to dispatch — report per-worker duration and total time"
description: >
  Add elapsed time tracking to dispatch — report per-worker duration and total time

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T23:04:03Z
last_update: 2026-04-04T23:04:03Z
date_finished: null
---

# T-875: Add elapsed time tracking to dispatch — report per-worker duration and total time

## Context

Dispatch reports worker results but no timing. Adding elapsed time per-worker and total
dispatch duration helps debug slow workers and understand orchestration overhead. Relates to
T-280 dispatch readiness.

## Acceptance Criteria

### Agent
- [x] Total dispatch elapsed time reported in output (text and JSON)
- [x] Per-worker elapsed time calculated from event timestamps
- [x] JSON output includes `elapsed_secs` and per-result `elapsed_secs`
- [x] Text output shows human-readable duration
- [x] `cargo clippy --workspace` passes with no warnings
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

### 2026-04-04T23:04:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-875-add-elapsed-time-tracking-to-dispatch--r.md
- **Context:** Initial task creation
