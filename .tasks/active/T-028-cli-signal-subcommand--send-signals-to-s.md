---
id: T-028
name: "CLI signal subcommand — send signals to session child processes"
description: >
  CLI signal subcommand — send signals to session child processes

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T19:54:35Z
last_update: 2026-03-08T19:54:35Z
date_finished: null
---

# T-028: CLI signal subcommand — send signals to session child processes

## Context

Add `termlink signal <target> <signal>` to send signals to session processes. Accepts names (TERM, INT, KILL, HUP, etc.) or numbers, with optional SIG prefix.

## Acceptance Criteria

### Agent
- [x] `Signal` variant in CLI with `target` and `signal` args
- [x] `parse_signal` maps names (case-insensitive, SIG prefix optional) to libc constants
- [x] Falls back to numeric parsing for raw signal numbers
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

### 2026-03-08T19:54:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-028-cli-signal-subcommand--send-signals-to-s.md
- **Context:** Initial task creation
