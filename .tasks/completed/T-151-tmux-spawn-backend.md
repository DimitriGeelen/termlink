---
id: T-151
name: "tmux spawn backend"
description: >
  Implement spawn_via_tmux: tmux new-session -d, cleanup via kill-session

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-16T05:45:25Z
last_update: 2026-03-16T06:08:51Z
date_finished: 2026-03-16T06:08:51Z
---

# T-151: tmux spawn backend

## Context

T-150 extracted spawn backends. This task validates and refines spawn_via_tmux.

## Acceptance Criteria

### Agent
- [x] `termlink spawn --backend tmux --name test --shell --wait` creates tmux session and registers
- [x] tmux session named `tl-<name>` appears in `tmux list-sessions`
- [x] `termlink pty inject` and `termlink pty output` work on the tmux-spawned session
- [x] `termlink interact --json` returns structured output with correct exit_code
- [x] `tmux kill-session -t tl-<name>` cleans up the session
- [x] All existing tests pass (264)

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

### 2026-03-16T05:45:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-151-tmux-spawn-backend.md
- **Context:** Initial task creation

### 2026-03-16T05:56:00Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-16T06:08:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
