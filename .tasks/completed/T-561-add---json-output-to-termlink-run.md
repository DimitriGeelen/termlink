---
id: T-561
name: "Add --json output to termlink run"
description: >
  Add --json output to termlink run

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:26:16Z
last_update: 2026-03-28T12:28:29Z
date_finished: 2026-03-28T12:28:29Z
---

# T-561: Add --json output to termlink run

## Context

Add `--json` flag to `termlink run` so automation can capture exit code, stdout, stderr, and timing in machine-readable format.

## Acceptance Criteria

### Agent
- [x] `Run` variant in cli.rs has `json: bool` field
- [x] `cmd_run` outputs JSON with exit_code, stdout, stderr, elapsed_ms when --json is passed
- [x] Integration test validates JSON output from run --json (2 tests: success + nonzero exit)
- [x] All existing tests pass

## Verification

cargo test -p termlink --test cli_integration -- cli_run_json 2>&1 | grep -q "test result"
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

### 2026-03-28T12:26:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-561-add---json-output-to-termlink-run.md
- **Context:** Initial task creation

### 2026-03-28T12:28:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
