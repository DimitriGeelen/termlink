---
id: T-623
name: "Add --json flag to hub stop command"
description: >
  Add --json flag to hub stop command

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:45:25Z
last_update: 2026-03-28T17:46:39Z
date_finished: 2026-03-28T17:46:39Z
---

# T-623: Add --json flag to hub stop command

## Context

`hub stop` is the only hub subcommand without `--json` support.

## Acceptance Criteria

### Agent
- [x] `--json` flag added to `HubAction::Stop` in cli.rs
- [x] `cmd_hub_stop` accepts `json: bool` and outputs structured JSON for all outcomes
- [x] main.rs wires the json parameter
- [x] `cargo check -p termlink` passes

## Verification

cargo check -p termlink 2>&1 | tail -1

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

### 2026-03-28T17:45:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-623-add---json-flag-to-hub-stop-command.md
- **Context:** Initial task creation

### 2026-03-28T17:46:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
