---
id: T-1045
name: "Add CLI tests for ping, status, and clean error paths"
description: >
  Add CLI integration tests for ping/status on nonexistent session and clean with no sessions.

status: started-work
workflow_type: test
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T22:31:37Z
last_update: 2026-04-13T22:31:37Z
date_finished: null
---

# T-1045: Add CLI tests for ping, status, and clean error paths

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Test: `termlink ping` on nonexistent session returns error
- [x] Test: `termlink status` on nonexistent session returns error
- [x] Test: `termlink clean --dry-run` with no stale sessions works
- [x] All 3 tests pass, zero clippy warnings

### Human
- [ ] [RUBBER-STAMP] Verify test count increased
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- cli_ping_nonexist cli_status_nonexist cli_clean_empty 2>&1 | grep "passed"`
  **Expected:** 3 tests passed
  **If not:** Check test filter names

## Verification

cargo test -p termlink -- cli_ping_nonexist 2>&1 | grep "passed"
cargo test -p termlink -- cli_status_nonexist 2>&1 | grep "passed"
cargo test -p termlink -- cli_clean_empty 2>&1 | grep "passed"

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

### 2026-04-13T22:31:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1045-add-cli-tests-for-ping-status-and-clean-.md
- **Context:** Initial task creation
