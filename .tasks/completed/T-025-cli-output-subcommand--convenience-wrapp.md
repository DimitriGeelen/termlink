---
id: T-025
name: "CLI output subcommand — convenience wrapper for query.output"
description: >
  CLI output subcommand — convenience wrapper for query.output

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T19:30:33Z
last_update: 2026-03-08T19:32:26Z
date_finished: 2026-03-08T19:32:26Z
---

# T-025: CLI output subcommand — convenience wrapper for query.output

## Context

Add `termlink output <target>` CLI subcommand as a convenience wrapper for `query.output`. Supports `--lines N` (default 50) and `--bytes N` flags.

## Acceptance Criteria

### Agent
- [x] `Output` variant added to CLI Command enum with `target`, `--lines`, `--bytes` args
- [x] `cmd_output` function sends `query.output` and prints raw output
- [x] Builds and all 102 tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo build 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | tail -1

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

### 2026-03-08T19:30:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-025-cli-output-subcommand--convenience-wrapp.md
- **Context:** Initial task creation

### 2026-03-08T19:32:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
