---
id: T-194
name: "Fix register --shell liveness check failing inside tmux/scripts"
description: >
  Two issues discovered during T-192/T-193 simulation testing:
  1. termlink list auto-cleans sessions it considers stale (destructive side-effect)
  2. register --shell PTY fails to init when stdout is /dev/null (no terminal)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [bug, liveness, pty]
components: []
related_tasks: [T-192, T-193]
created: 2026-03-20T13:12:58Z
last_update: 2026-03-20T13:20:02Z
date_finished: 2026-03-20T13:20:02Z
---

# T-194: Fix register --shell liveness check failing inside tmux/scripts

## Context

Discovered during T-192/T-193: `register --shell` sessions don't appear in
`termlink list` when run inside scripts because:
1. `list_sessions` (manager.rs:225) auto-cleans sessions it considers stale — destructive
2. PTY shell doesn't init when stdout is /dev/null (background process redirects)

## Acceptance Criteria

### Agent
- [x] `termlink list` does NOT auto-clean stale sessions (separate from `termlink clean`)
- [x] `register --shell` works when run as background process with stdout redirected
- [x] Existing tests pass (`cargo test --package termlink-session`) — 18/18
- [x] Existing liveness tests already cover alive PID + valid socket scenarios (4 tests in liveness.rs)

### Human
<!-- No human ACs -->

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-session

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

### 2026-03-20T13:12:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-194-fix-register---shell-liveness-check-fail.md
- **Context:** Initial task creation

### 2026-03-20T13:20:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
