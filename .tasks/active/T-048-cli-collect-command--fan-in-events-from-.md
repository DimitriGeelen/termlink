---
id: T-048
name: "CLI collect command — fan-in events from multiple sessions via hub"
description: >
  CLI collect command — fan-in events from multiple sessions via hub

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T23:00:53Z
last_update: 2026-03-08T23:00:53Z
date_finished: null
---

# T-048: CLI collect command — fan-in events from multiple sessions via hub

## Context

CLI interface for the hub's `event.collect` RPC — fan-in events from all (or targeted) sessions through the hub with continuous polling and cursor tracking.

## Acceptance Criteria

### Agent
- [x] `termlink collect` polls hub for events from all sessions
- [x] `--targets` filters to specific sessions
- [x] `--topic` filters by event topic
- [x] `--count N` exits after N events
- [x] `--interval` controls poll frequency
- [x] Cursor tracking prevents duplicate events across polls
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

### 2026-03-08T23:00:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-048-cli-collect-command--fan-in-events-from-.md
- **Context:** Initial task creation
