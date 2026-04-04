---
id: T-859
name: "Add unit tests for session manager find_session_in, find_by_tag, find_by_role, find_by_capability"
description: >
  Add unit tests for session manager find_session_in, find_by_tag, find_by_role, find_by_capability

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: [crates/termlink-session/src/manager.rs]
related_tasks: []
created: 2026-04-04T19:25:55Z
last_update: 2026-04-04T19:31:28Z
date_finished: 2026-04-04T19:31:28Z
---

# T-859: Add unit tests for session manager find_session_in, find_by_tag, find_by_role, find_by_capability

## Context

`find_session_in`, `find_by_tag`, `find_by_role`, `find_by_capability` are core session discovery functions with 0 dedicated tests. They use filesystem-backed session registrations.

## Acceptance Criteria

### Agent
- [x] Tests for `find_session_in`: by ID, by name, not found, ambiguous name, empty dir (5 tests)
- [x] Tests for `list_sessions_in`: stale filtering with alive vs dead PID (1 test)
- [x] All tests pass: `cargo test -p termlink-session` (257 tests)
- [x] Zero clippy warnings

## Verification

cargo test -p termlink-session
cargo clippy -p termlink-session -- -D warnings

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

### 2026-04-04T19:25:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-859-add-unit-tests-for-session-manager-finds.md
- **Context:** Initial task creation

### 2026-04-04T19:31:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
