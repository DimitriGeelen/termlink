---
id: T-1095
name: "Add CLI tests for fleet doctor with profiles and fleet reauth bootstrap"
description: >
  Add CLI tests for fleet doctor with profiles and fleet reauth bootstrap

status: work-completed
workflow_type: test
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T21:45:25Z
last_update: 2026-04-16T21:48:47Z
date_finished: 2026-04-16T21:48:47Z
---

# T-1095: Add CLI tests for fleet doctor with profiles and fleet reauth bootstrap

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Test: fleet doctor with unreachable profile shows error in JSON (ok=false, status=error)
- [x] Test: fleet reauth --bootstrap-from file:<path> reads and writes secret (heal complete confirmed)
- [x] Test: fleet reauth --bootstrap-from file:<nonexistent> fails gracefully (exit 1, file not found)
- [x] All 3 new tests pass

### Human
- [x] [RUBBER-STAMP] Verify test count increased — ticked by user direction 2026-04-23. Evidence: cli_integration: 3 passed for fleet_doctor_unreachable/fleet_reauth_bootstrap tests. Verified live via cargo test 2026-04-23T17:25Z.
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- fleet_doctor fleet_reauth 2>&1 | grep "passed"`
  **Expected:** Additional tests beyond baseline
  **If not:** Check test names


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, test-count, t-1095):** Implementation commit `16d83b58` added 3 new test function(s) covering fleet doctor profiles + fleet reauth bootstrap in `crates/termlink-cli/tests/cli_integration.rs`. Current file holds ~168 tests (grep'd test-attribute or fn-test count). Pre-series baseline was lower; test count clearly increased. RUBBER-STAMPable.

## Verification

bash -c 'cargo test -p termlink -- fleet_doctor_unreachable fleet_reauth_bootstrap 2>&1 | grep -q "3 passed"'

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

### 2026-04-16T21:45:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1095-add-cli-tests-for-fleet-doctor-with-prof.md
- **Context:** Initial task creation

### 2026-04-16T21:48:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T22:07:03Z — programmatic-evidence [T-1097]
- **Evidence:** 3 fleet doctor + reauth bootstrap tests passing: cargo test -p termlink -- fleet_doctor_unreachable fleet_reauth_bootstrap (3 passed)
- **Verified by:** automated command execution
