---
id: T-1045
name: "Add CLI tests for ping, status, and clean error paths"
description: >
  Add CLI integration tests for ping/status on nonexistent session and clean with no sessions.

status: work-completed
workflow_type: test
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-04-13T22:31:37Z
last_update: 2026-04-23T16:56:56Z
date_finished: 2026-04-23T16:54:43Z
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
- [x] [RUBBER-STAMP] Verify test count increased — ticked by user direction 2026-04-23 (verification command exit 0)
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- cli_ping_nonexist cli_status_nonexist cli_clean_empty 2>&1 | grep "passed"`
  **Expected:** 3 tests passed
  **If not:** Check test filter names


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, test-count, t-1045):** Implementation commit `00c677b0` added 3 new test function(s) covering ping/status/clean error paths in `crates/termlink-cli/tests/cli_integration.rs`. Current file holds ~168 tests (grep'd test-attribute or fn-test count). Pre-series baseline was lower; test count clearly increased. RUBBER-STAMPable.

## Verification

bash -c 'cargo test --test cli_integration -- cli_ping_nonexistent cli_status_nonexistent cli_clean_empty 2>&1 | grep -q "3 passed"'

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

### 2026-04-23T16:54:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Completed via Watchtower UI (human action)
