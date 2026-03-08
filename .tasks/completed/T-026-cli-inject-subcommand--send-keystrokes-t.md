---
id: T-026
name: "CLI inject subcommand — send keystrokes to PTY sessions"
description: >
  CLI inject subcommand — send keystrokes to PTY sessions

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T19:38:16Z
last_update: 2026-03-08T19:40:23Z
date_finished: 2026-03-08T19:40:23Z
---

# T-026: CLI inject subcommand — send keystrokes to PTY sessions

## Context

Add `termlink inject <target> <text> [--enter] [--key <name>]` for convenient keystroke injection into PTY sessions.

## Acceptance Criteria

### Agent
- [x] `Inject` variant in CLI Command enum with `target`, `text`, `--enter`, `--key` args
- [x] `cmd_inject` builds KeyEntry array and sends `command.inject`
- [x] `--enter` appends Enter key after text
- [x] `--key` sends a named key instead of text
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

### 2026-03-08T19:38:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-026-cli-inject-subcommand--send-keystrokes-t.md
- **Context:** Initial task creation

### 2026-03-08T19:40:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
