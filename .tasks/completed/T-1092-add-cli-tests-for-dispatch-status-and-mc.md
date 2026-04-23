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
last_update: 2026-04-23T17:22:34Z
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
- [x] [RUBBER-STAMP] Verify test count increased — ticked by user direction 2026-04-23. Evidence: cli_integration: 4 passed for dispatch_status/mcp_serve tests. Verified live via cargo test 2026-04-23T17:25Z.
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- dispatch_status mcp 2>&1 | grep "passed"`
  **Expected:** New tests passing
  **If not:** Check test names


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, test-count, t-1092):** Implementation commit `a9c5b453` added 4 new test function(s) covering dispatch-status + mcp commands in `crates/termlink-cli/tests/cli_integration.rs`. Current file holds ~168 tests (grep'd test-attribute or fn-test count). Pre-series baseline was lower; test count clearly increased. RUBBER-STAMPable.

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
