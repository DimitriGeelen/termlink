---
id: T-153
name: "Spawn --backend CLI flag + auto-detection"
description: >
  Add --backend tmux|terminal|background|auto flag to spawn command with platform auto-detection

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-16T05:45:34Z
last_update: 2026-03-16T06:34:57Z
date_finished: 2026-03-16T06:34:57Z
---

# T-153: Spawn --backend CLI flag + auto-detection

## Context

Already implemented in T-150. --backend flag and resolve_spawn_backend() auto-detection built together with the refactor.

## Acceptance Criteria

### Agent
- [x] `--backend auto|terminal|tmux|background` flag on spawn command
- [x] Auto-detection: macOS+GUI→terminal, tmux available→tmux, fallback→background
- [x] `termlink spawn --help` shows all backend options
- [x] All tests pass

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

### 2026-03-16T05:45:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-153-spawn---backend-cli-flag--auto-detection.md
- **Context:** Initial task creation

### 2026-03-16T06:34:29Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-16T06:34:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
