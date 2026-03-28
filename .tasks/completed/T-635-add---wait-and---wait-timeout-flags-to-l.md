---
id: T-635
name: "Add --wait and --wait-timeout flags to list command"
description: >
  Add --wait and --wait-timeout flags to list command

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T18:06:32Z
last_update: 2026-03-28T18:07:46Z
date_finished: 2026-03-28T18:07:46Z
---

# T-635: Add --wait and --wait-timeout flags to list command

## Context

`discover --wait` polls until sessions match but `list` has no wait mode. Converting list to async for consistency.

## Acceptance Criteria

### Agent
- [x] `--wait` and `--wait-timeout` flags added to `Command::List` in cli.rs
- [x] `cmd_list` converted to async and polls with 250ms interval when `--wait` is set
- [x] Timeout produces JSON error when `--json` and exits 1
- [x] main.rs uses `.await` for list dispatch
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

### 2026-03-28T18:06:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-635-add---wait-and---wait-timeout-flags-to-l.md
- **Context:** Initial task creation

### 2026-03-28T18:07:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
