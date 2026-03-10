---
id: T-067
name: "Session state machine validation — transition guards, Drop impl, TOCTOU fix"
description: >
  SessionState accepts any transition. Add valid_transition guard, Drop impl for cleanup on panic, fix TOCTOU race on display name.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:31Z
last_update: 2026-03-10T08:44:31Z
date_finished: null
---

# T-067: Session state machine validation — transition guards, Drop impl, TOCTOU fix

## Context

Session management issues found by reflection fleet session agent. State machine accepts invalid transitions, no Drop impl, TOCTOU race on display name. See [docs/reports/reflection-result-session.md].

## Acceptance Criteria

### Agent
- [ ] `set_state()` has a `valid_transition()` guard that rejects invalid state transitions (e.g., Terminated → Active)
- [ ] Invalid transition attempts return a typed error (not panic)
- [ ] `Session` implements `Drop` with best-effort cleanup (remove socket file, remove JSON registration)
- [ ] `register_in()` cleans up socket file if JSON write fails (no socket leak on error path)
- [ ] TOCTOU race on display-name uniqueness is documented with a code comment explaining the risk and mitigation path
- [ ] Unit tests cover: valid transitions succeed, invalid transitions are rejected, Drop cleans up files

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-session -- state 2>&1 | tail -5
grep -q "valid_transition\|fn drop" crates/termlink-session/src/session.rs

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

### 2026-03-10T08:44:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-067-session-state-machine-validation--transi.md
- **Context:** Initial task creation
