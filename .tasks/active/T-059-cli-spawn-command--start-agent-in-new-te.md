---
id: T-059
name: "CLI spawn command — start agent in new terminal with auto-registration"
description: >
  CLI spawn command — start agent in new terminal with auto-registration

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-09T12:01:51Z
last_update: 2026-03-09T12:07:02Z
date_finished: null
---

# T-059: CLI spawn command — start agent in new terminal with auto-registration

## Context

Phase 1 of T-012 agent-to-agent communication. Adds `termlink spawn` command that opens a new terminal window and runs a command with TermLink session auto-registration. Design at `docs/reports/T-012-agent-to-agent-communication.md`.

## Acceptance Criteria

### Agent
- [x] `Spawn` variant added to Command enum with name, roles, tags, wait, and command args
- [x] `cmd_spawn` function opens new terminal via macOS AppleScript
- [x] Spawned command is wrapped to inherit TERMLINK_RUNTIME_DIR
- [x] `--wait` flag polls for session registration before returning
- [x] CLI builds and all existing tests pass (13/13)
- [x] Help text shows spawn command

### Human
- [ ] [REVIEW] `termlink spawn --name test-agent -- echo hello` opens a new terminal window
  **Steps:**
  1. Run `termlink spawn --name test-agent -- echo hello`
  2. Observe new Terminal.app window opens
  3. Run `termlink list` in original terminal
  **Expected:** New terminal window visible, session appears in list
  **If not:** Note error message and whether terminal opened

## Verification

/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | grep -q "Finished"
/Users/dimidev32/.cargo/bin/cargo test -p termlink --test cli_integration 2>&1 | grep -q "passed"

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

### 2026-03-09T12:01:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-059-cli-spawn-command--start-agent-in-new-te.md
- **Context:** Initial task creation
