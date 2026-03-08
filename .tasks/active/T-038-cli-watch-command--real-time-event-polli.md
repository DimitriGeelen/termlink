---
id: T-038
name: "CLI watch command — real-time event polling"
description: >
  CLI watch command — real-time event polling

status: started-work
workflow_type: build
owner: claude-code
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T21:12:40Z
last_update: 2026-03-08T21:12:40Z
date_finished: null
---

# T-038: CLI watch command — real-time event polling

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Watch subcommand added to CLI with --interval, --topic, and optional target args
- [x] Watches all sessions when no targets specified
- [x] Delta polling with per-session sequence cursors
- [x] All 126 tests pass

### Human
- [ ] [RUBBER-STAMP] Watch command shows live events from multiple sessions
  **Steps:**
  1. Register two sessions: `termlink register --name s1 --shell` and `termlink register --name s2 --shell`
  2. In another terminal: `termlink watch`
  3. Emit events: `termlink emit s1 build.done` and `termlink emit s2 test.pass`
  **Expected:** Watch output shows events from both sessions with session name prefixes
  **If not:** Note which sessions are missing events

## Verification

grep -q "Watch" crates/termlink-cli/src/main.rs
grep -q "cmd_watch" crates/termlink-cli/src/main.rs
/Users/dimidev32/.cargo/bin/cargo build 2>&1 | tail -1 | grep -q "Finished"

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

### 2026-03-08T21:12:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-038-cli-watch-command--real-time-event-polli.md
- **Context:** Initial task creation
