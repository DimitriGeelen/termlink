---
id: T-082
name: "Hub session supervision loop — active liveness polling"
description: >
  Active supervision loop in hub: poll session liveness every 30s, cleanup stale sessions, log warnings for dead sessions.

status: work-completed
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-03-10T22:10:41Z
last_update: 2026-03-10T22:20:14Z
date_finished: 2026-03-10T22:20:14Z
---

# T-082: Hub session supervision loop — active liveness polling

## Context

From T-066 inception (GO). See [docs/reports/T-066-hub-daemon-inception.md].

## Acceptance Criteria

### Agent
- [x] Supervision module in termlink-hub polls session liveness periodically (30s default)
- [x] Stale sessions cleaned up automatically (socket + JSON removed)
- [x] Supervision loop respects shutdown signal
- [x] Tests verify supervision detects dead sessions (4 tests)
- [x] All existing hub tests continue to pass (28 total)

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-hub 2>&1 | tail -5

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

### 2026-03-10T22:10:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-082-hub-session-supervision-loop--active-liv.md
- **Context:** Initial task creation

### 2026-03-10T22:18:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T22:20:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
