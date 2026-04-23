---
id: T-1091
name: "Add CLI integration tests for remote list, exec, doctor, and push error paths"
description: >
  Add CLI integration tests for remote list, exec, doctor, and push error paths

status: work-completed
workflow_type: test
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-04-16T21:18:00Z
last_update: 2026-04-23T17:22:33Z
date_finished: 2026-04-16T21:22:23Z
---

# T-1091: Add CLI integration tests for remote list, exec, doctor, and push error paths

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Tests for `remote list` error paths (no secret, unreachable hub)
- [x] Tests for `remote exec` error paths (missing command, unreachable hub)
- [x] Tests for `remote doctor` error paths (no secret, unreachable JSON)
- [x] Tests for `remote push` error paths (no file/message, nonexistent file)
- [x] All 8 new tests pass, zero clippy warnings on the test crate

### Human
- [x] [RUBBER-STAMP] Verify test count increased — ticked by user direction 2026-04-23. Evidence: cli_integration: 8 passed for remote_list/remote_exec/remote_doctor/remote_push tests; clippy clean. Verified live via cargo test 2026-04-23T17:25Z.
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- remote 2>&1 | grep "passed"`
  **Expected:** Additional tests passing beyond baseline
  **If not:** Check test names
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, test-count, t-1091):** Implementation commit `53f8fca2` added 8 new test function(s) covering remote list/exec/doctor/push error paths in `crates/termlink-cli/tests/cli_integration.rs`. Current file holds ~168 tests (grep'd test-attribute or fn-test count). Pre-series baseline was lower; test count clearly increased. RUBBER-STAMPable.

## Verification

bash -c 'cargo test -p termlink -- remote_list remote_exec remote_doctor remote_push 2>&1 | grep -q "8 passed"'
bash -c '[ "$(cargo clippy -p termlink --tests 2>&1 | grep -c "^error")" = "0" ]'

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

### 2026-04-16T21:18:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1091-add-cli-integration-tests-for-remote-lis.md
- **Context:** Initial task creation

### 2026-04-16T21:22:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T22:07:03Z — programmatic-evidence [T-1097]
- **Evidence:** 8 remote error-path tests passing: cargo test -p termlink -- remote_list remote_exec remote_doctor remote_push (8 passed)
- **Verified by:** automated command execution
