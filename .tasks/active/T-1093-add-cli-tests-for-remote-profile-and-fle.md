---
id: T-1093
name: "Add CLI tests for remote profile and fleet reauth commands"
description: >
  Add CLI tests for remote profile and fleet reauth commands

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T21:26:47Z
last_update: 2026-04-16T21:26:47Z
date_finished: null
---

# T-1093: Add CLI tests for remote profile and fleet reauth commands

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Tests for `remote profile list` (empty, after add)
- [x] Tests for `remote profile add` (add + verify in list)
- [x] Tests for `remote profile remove` (existing removed, nonexistent reports not found)
- [x] Tests for `fleet reauth` (no config → error, valid profile → heal steps)
- [x] All 6 new tests pass, zero clippy warnings

### Human
- [ ] [RUBBER-STAMP] Verify test count increased
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- remote_profile fleet_reauth 2>&1 | grep "passed"`
  **Expected:** New tests passing
  **If not:** Check test names

## Verification

bash -c 'cargo test -p termlink -- remote_profile fleet_reauth 2>&1 | grep -q "6 passed"'
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-16T21:26:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1093-add-cli-tests-for-remote-profile-and-fle.md
- **Context:** Initial task creation
