---
id: T-1041
name: "Add CLI tests for hub restart and fleet doctor error paths"
description: >
  Add CLI integration tests for hub restart (not running, stale pid) and fleet doctor (no hubs configured). Follows T-1033 pattern.

status: started-work
workflow_type: test
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-04-13T20:53:38Z
last_update: 2026-04-17T08:11:24Z
date_finished: null
---

# T-1041: Add CLI tests for hub restart and fleet doctor error paths

## Context

Add CLI integration tests for hub restart and fleet doctor error paths. No existing tests cover these commands. Follows T-1033 test pattern (TestDir isolation, JSON output parsing).

## Acceptance Criteria

### Agent
- [x] Test: hub restart when no hub running reports error (not-running text and JSON)
- [x] Test: hub restart with stale pidfile reports stale error (text and JSON)
- [x] Test: fleet doctor with no hubs.toml reports empty config (JSON)
- [x] Test: fleet doctor JSON output has expected structure (ok=true, hubs=[])
- [x] All 5 new tests pass, zero clippy warnings

### Human
- [ ] [RUBBER-STAMP] Verify test count increased
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- hub_restart fleet_doctor 2>&1 | grep "passed"`
  **Expected:** 4+ tests passed
  **If not:** Check test names match the filter


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, test-count, t-1041):** Implementation commit `015a4aa8` added 5 new test function(s) covering hub restart + fleet doctor error paths in `crates/termlink-cli/tests/cli_integration.rs`. Current file holds ~168 tests (grep'd test-attribute or fn-test count). Pre-series baseline was lower; test count clearly increased. RUBBER-STAMPable.

## Verification

bash -c 'cargo test --test cli_integration -- hub_restart fleet_doctor_no 2>&1 | grep -q "5 passed"'

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

### 2026-04-13T20:53:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1041-add-cli-tests-for-hub-restart-and-fleet-.md
- **Context:** Initial task creation
