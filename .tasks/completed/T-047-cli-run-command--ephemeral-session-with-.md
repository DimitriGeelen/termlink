---
id: T-047
name: "CLI run command — ephemeral session with command execution and auto-cleanup"
description: >
  CLI run command — ephemeral session with command execution and auto-cleanup

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T22:55:28Z
last_update: 2026-03-08T23:00:38Z
date_finished: 2026-03-08T23:00:38Z
---

# T-047: CLI run command — ephemeral session with command execution and auto-cleanup

## Context

One-liner for automation: `termlink run --name builder make build`. Registers ephemeral session, executes command, auto-deregisters. Session is queryable via RPC during execution.

## Acceptance Criteria

### Agent
- [x] `termlink run <command>` registers, executes, deregisters
- [x] `--name` and `--tags` flags for session metadata
- [x] `--timeout` flag (default 300s)
- [x] Exit code propagated from command
- [x] stdout/stderr forwarded correctly
- [x] Session is RPC-queryable during execution
- [x] All tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test 2>&1 | grep -E "^test result:" | grep -v "0 passed" | head -4

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

### 2026-03-08T22:55:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-047-cli-run-command--ephemeral-session-with-.md
- **Context:** Initial task creation

### 2026-03-08T23:00:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
