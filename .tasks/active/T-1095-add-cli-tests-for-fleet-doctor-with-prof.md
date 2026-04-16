---
id: T-1095
name: "Add CLI tests for fleet doctor with profiles and fleet reauth bootstrap"
description: >
  Add CLI tests for fleet doctor with profiles and fleet reauth bootstrap

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T21:45:25Z
last_update: 2026-04-16T21:45:25Z
date_finished: null
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
- [ ] [RUBBER-STAMP] Verify test count increased
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- fleet_doctor fleet_reauth 2>&1 | grep "passed"`
  **Expected:** Additional tests beyond baseline
  **If not:** Check test names

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
