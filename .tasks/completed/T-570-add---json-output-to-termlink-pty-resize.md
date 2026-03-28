---
id: T-570
name: "Add --json output to termlink pty resize"
description: >
  Add --json output to termlink pty resize

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:46:09Z
last_update: 2026-03-28T12:47:45Z
date_finished: 2026-03-28T12:47:45Z
---

# T-570: Add --json output to termlink pty resize

## Context

Add `--json` flag to `termlink pty resize` and hidden `termlink resize` for structured output.

## Acceptance Criteria

### Agent
- [x] `PtyCommand::Resize` and hidden `Resize` have `json: bool` field
- [x] `cmd_resize` outputs JSON with cols, rows when --json is passed
- [x] All existing tests pass

## Verification

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

### 2026-03-28T12:46:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-570-add---json-output-to-termlink-pty-resize.md
- **Context:** Initial task creation

### 2026-03-28T12:47:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
