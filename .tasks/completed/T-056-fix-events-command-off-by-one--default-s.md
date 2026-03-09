---
id: T-056
name: "Fix events command off-by-one — default since=0 misses first event"
description: >
  Fix events command off-by-one — default since=0 misses first event

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-09T10:09:42Z
last_update: 2026-03-09T10:15:47Z
date_finished: 2026-03-09T10:15:47Z
---

# T-056: Fix events command off-by-one — default since=0 misses first event

## Context

The `events` CLI command defaulted `--since` to 0. Since the RPC handler uses `seq > since` (exclusive), `since=0` misses the first event at seq=0. The handler already supports omitting `since` to return all events — the CLI just needed to stop always sending it.

## Acceptance Criteria

### Agent
- [x] `--since` is optional in CLI (no default value)
- [x] `events` without `--since` returns all events including seq=0
- [x] `events --since N` still works for delta polling
- [x] CLI integration test `cli_emit_and_events` verifies seq=0 is visible
- [x] All 13 CLI integration tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink --test cli_integration 2>&1 | grep -q "13 passed"

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

### 2026-03-09T10:09:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-056-fix-events-command-off-by-one--default-s.md
- **Context:** Initial task creation

### 2026-03-09T10:15:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
