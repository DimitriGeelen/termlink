---
id: T-1092
name: "Add CLI tests for dispatch-status and mcp commands"
description: >
  Add CLI tests for dispatch-status and mcp commands

status: work-completed
workflow_type: test
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T21:23:16Z
last_update: 2026-04-16T21:25:52Z
date_finished: 2026-04-16T21:25:52Z
---

# T-1092: Add CLI tests for dispatch-status and mcp commands

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Tests for `dispatch-status` (no manifest, --check exit 0, --json with expected fields)
- [x] Tests for `mcp serve` error path (stdin closed → connection closed)
- [x] All 4 new tests pass, zero clippy warnings

### Human
- [ ] [RUBBER-STAMP] Verify test count increased
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- dispatch_status mcp 2>&1 | grep "passed"`
  **Expected:** New tests passing
  **If not:** Check test names

## Verification

bash -c 'cargo test -p termlink -- dispatch_status mcp_serve 2>&1 | grep -q "4 passed"'

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

### 2026-04-16T21:23:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1092-add-cli-tests-for-dispatch-status-and-mc.md
- **Context:** Initial task creation

### 2026-04-16T21:25:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T22:07:03Z — programmatic-evidence [T-1097]
- **Evidence:** 4 dispatch-status + mcp tests passing: cargo test -p termlink -- dispatch_status mcp_serve (4 passed)
- **Verified by:** automated command execution
