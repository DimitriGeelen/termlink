---
id: T-563
name: "Add --json output to termlink kv"
description: >
  Add --json output to termlink kv

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:31:09Z
last_update: 2026-03-28T12:33:14Z
date_finished: 2026-03-28T12:33:14Z
---

# T-563: Add --json output to termlink kv

## Context

Add `--json` flag to `termlink kv` so automation can machine-parse kv operations.

## Acceptance Criteria

### Agent
- [x] `Kv` variant in cli.rs has `json: bool` field
- [x] `cmd_kv` outputs raw RPC JSON result for all subactions when --json is passed
- [x] Integration test validates JSON output from kv set/get/list/del --json
- [x] All existing tests pass

## Verification

cargo test -p termlink --test cli_integration -- cli_kv_json 2>&1 | grep -q "test result"
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

### 2026-03-28T12:31:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-563-add---json-output-to-termlink-kv.md
- **Context:** Initial task creation

### 2026-03-28T12:33:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
