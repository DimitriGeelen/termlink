---
id: T-019
name: "Command execution and key injection handlers"
description: >
  Command execution and key injection handlers

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T17:08:41Z
last_update: 2026-03-08T17:14:06Z
date_finished: 2026-03-08T17:14:06Z
---

# T-019: Command execution and key injection handlers

## Context

Implements `command.execute` (spawn shell command, capture output) and `command.inject` (resolve key entries to bytes) per T-005 protocol spec. Also adds `command.signal` and `exec` CLI subcommand.

## Acceptance Criteria

### Agent
- [x] `command.execute` handler spawns shell command and returns stdout/stderr/exit_code
- [x] `command.inject` handler resolves KeyEntry array to raw bytes
- [x] `command.signal` handler sends POSIX signal to session PID
- [x] Executor module with async command spawning, timeout, and output capture
- [x] CLI `exec` subcommand sends command.execute to target session
- [x] Tests for execution, injection, and signal handlers
- [x] `cargo test --workspace` passes

## Verification

PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace
PATH="$HOME/.cargo/bin:$PATH" cargo clippy --workspace -- -D warnings

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

### 2026-03-08T17:08:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-019-command-execution-and-key-injection-hand.md
- **Context:** Initial task creation

### 2026-03-08T17:14:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
