---
id: T-620
name: "Add JSON error output to file receive errors"
description: >
  Add JSON error output to file receive errors

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:39:06Z
last_update: 2026-03-28T17:40:39Z
date_finished: 2026-03-28T17:40:39Z
---

# T-620: Add JSON error output to file receive errors

## Context

`cmd_file_receive` has several `anyhow::bail!` calls (missing chunk, SHA-256 mismatch, transfer error) without JSON error output.

## Acceptance Criteria

### Agent
- [x] Missing chunk bails (2 locations) have JSON error output
- [x] SHA-256 mismatch bails (2 locations) have JSON error output
- [x] Transfer error bail has JSON error output
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

### 2026-03-28T17:39:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-620-add-json-error-output-to-file-receive-er.md
- **Context:** Initial task creation

### 2026-03-28T17:40:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
