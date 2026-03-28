---
id: T-600
name: "Add JSON-aware error output to request command timeout and emit failure"
description: >
  Add JSON-aware error output to request command timeout and emit failure

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T16:49:32Z
last_update: 2026-03-28T16:50:53Z
date_finished: 2026-03-28T16:50:53Z
---

# T-600: Add JSON-aware error output to request command timeout and emit failure

## Context

The `cmd_request` function in execution.rs has two error paths that use `anyhow::bail!()` without JSON-aware output: emit failure (line 194) and timeout (line 254). When `--json` is passed, these should output structured JSON errors.

## Acceptance Criteria

### Agent
- [x] `cmd_request` emit failure path outputs JSON error when `--json` is passed
- [x] `cmd_request` timeout path outputs JSON error when `--json` is passed
- [x] Project compiles with `cargo check`

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

### 2026-03-28T16:49:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-600-add-json-aware-error-output-to-request-c.md
- **Context:** Initial task creation

### 2026-03-28T16:50:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
