---
id: T-150
name: "Refactor cmd_spawn — extract spawn backend enum, keep Terminal.app as default"
description: >
  Refactor cmd_spawn — extract spawn backend enum, keep Terminal.app as default

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-16T05:43:05Z
last_update: 2026-03-16T05:43:05Z
date_finished: null
---

# T-150: Refactor cmd_spawn — extract spawn backend enum, keep Terminal.app as default

## Context

T-149 inception. Extract spawn backend from cmd_spawn so T-151/T-152 can add tmux and background backends.

## Acceptance Criteria

### Agent
- [x] SpawnBackend enum with Terminal, Tmux, Background, Auto variants
- [x] `spawn_via_terminal()` extracted from cmd_spawn (osascript logic isolated)
- [x] `cmd_spawn()` delegates to backend function via resolve + match
- [x] `--backend` arg added to Spawn CLI command (auto default)
- [x] All existing tests pass (264/264)
- [x] `termlink spawn --help` shows --backend option with all 4 values

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

### 2026-03-16T05:43:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-150-refactor-cmdspawn--extract-spawn-backend.md
- **Context:** Initial task creation
