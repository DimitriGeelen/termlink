---
id: T-560
name: "Add --json output to termlink register"
description: >
  Add --json output to termlink register

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:22:09Z
last_update: 2026-03-28T12:25:50Z
date_finished: 2026-03-28T12:25:50Z
---

# T-560: Add --json output to termlink register

## Context

Add `--json` flag to `termlink register` so scripted workflows can capture session ID, socket path, and PID in machine-readable format.

## Acceptance Criteria

### Agent
- [x] `Register` variant in cli.rs has `json: bool` field
- [x] `cmd_register` outputs JSON with id, display_name, socket_path, pid when --json is passed
- [x] `cmd_register_self` outputs JSON with id, display_name when --json is passed
- [x] Integration test validates JSON output from register --json
- [x] All existing tests pass (cargo test)

## Verification

cargo test -p termlink --test cli_integration -- cli_register_json 2>&1 | grep -q "test result"
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

### 2026-03-28T12:22:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-560-add---json-output-to-termlink-register.md
- **Context:** Initial task creation

### 2026-03-28T12:25:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
