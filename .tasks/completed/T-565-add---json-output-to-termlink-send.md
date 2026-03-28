---
id: T-565
name: "Add --json output to termlink send"
description: >
  Add --json output to termlink send

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:35:33Z
last_update: 2026-03-28T12:37:17Z
date_finished: 2026-03-28T12:37:17Z
---

# T-565: Add --json output to termlink send

## Context

Add `--json` flag to `termlink send` (raw JSON-RPC call) for structured output of the RPC response.

## Acceptance Criteria

### Agent
- [x] `Send` variant in cli.rs has `json: bool` field
- [x] `cmd_send` outputs raw JSON-RPC result when --json is passed
- [x] Integration test validates JSON output from send --json
- [x] All existing tests pass

## Verification

cargo test -p termlink --test cli_integration -- cli_send_json 2>&1 | grep -q "test result"
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

### 2026-03-28T12:35:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-565-add---json-output-to-termlink-send.md
- **Context:** Initial task creation

### 2026-03-28T12:37:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
