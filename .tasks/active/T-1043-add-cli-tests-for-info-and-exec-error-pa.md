---
id: T-1043
name: "Add CLI tests for info and exec error paths"
description: >
  Add CLI integration tests for info on nonexistent session and exec on nonexistent session. Quick error-path coverage.

status: started-work
workflow_type: test
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T22:15:21Z
last_update: 2026-04-13T22:16:23Z
date_finished: null
---

# T-1043: Add CLI tests for info and exec error paths

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Test: `termlink info` on nonexistent session returns error
- [x] Test: `termlink exec` on nonexistent session returns error
- [x] Test: `termlink signal` on nonexistent session returns error
- [x] All 3 tests pass, zero clippy warnings

### Human
- [x] [RUBBER-STAMP] Verify test count increased — ticked by user direction 2026-04-23 (verification command exit 0)
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- cli_info_nonexist cli_exec_nonexist cli_signal_nonexist 2>&1 | grep "passed"`
  **Expected:** 3+ tests passed
  **If not:** Check test filter names


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, test-count, t-1043):** Implementation commit `9b1ac0b4` added 3 new test function(s) covering info/exec/signal error paths in `crates/termlink-cli/tests/cli_integration.rs`. Current file holds ~168 tests (grep'd test-attribute or fn-test count). Pre-series baseline was lower; test count clearly increased. RUBBER-STAMPable.

## Verification

bash -c 'cargo test --test cli_integration -- cli_info_nonexistent cli_exec_nonexistent cli_signal_nonexistent 2>&1 | grep -q "3 passed"'

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

### 2026-04-13T22:15:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1043-add-cli-tests-for-info-and-exec-error-pa.md
- **Context:** Initial task creation
