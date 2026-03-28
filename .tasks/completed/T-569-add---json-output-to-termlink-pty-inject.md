---
id: T-569
name: "Add --json output to termlink pty inject"
description: >
  Add --json output to termlink pty inject

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:43:00Z
last_update: 2026-03-28T12:45:48Z
date_finished: 2026-03-28T12:45:48Z
---

# T-569: Add --json output to termlink pty inject

## Context

Add `--json` flag to `termlink pty inject` and hidden `termlink inject` for structured injection confirmation.

## Acceptance Criteria

### Agent
- [x] `PtyCommand::Inject` and hidden `Inject` have `json: bool` field
- [x] `cmd_inject` outputs JSON confirmation when --json is passed
- [x] Integration test validates JSON output from pty inject --json
- [x] All existing tests pass

## Verification

cargo test -p termlink --test cli_integration -- cli_inject_json 2>&1 | grep -q "test result"
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

### 2026-03-28T12:43:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-569-add---json-output-to-termlink-pty-inject.md
- **Context:** Initial task creation

### 2026-03-28T12:45:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
