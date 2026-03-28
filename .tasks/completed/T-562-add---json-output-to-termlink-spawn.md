---
id: T-562
name: "Add --json output to termlink spawn"
description: >
  Add --json output to termlink spawn

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:28:44Z
last_update: 2026-03-28T12:30:46Z
date_finished: 2026-03-28T12:30:46Z
---

# T-562: Add --json output to termlink spawn

## Context

Add `--json` flag to `termlink spawn` so automation can capture spawned session name and backend in machine-readable format.

## Acceptance Criteria

### Agent
- [x] `Spawn` variant in cli.rs has `json: bool` field
- [x] `cmd_spawn` outputs JSON with session_name, backend, and ready status when --json is passed
- [x] Integration test validates JSON output from spawn --json
- [x] All existing tests pass

## Verification

cargo test -p termlink --test cli_integration -- cli_spawn_json 2>&1 | grep -q "test result"
cargo clippy -p termlink -- -D warnings 2>&1 | tail -1 | grep -qv error

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

### 2026-03-28T12:28:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-562-add---json-output-to-termlink-spawn.md
- **Context:** Initial task creation

### 2026-03-28T12:30:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
