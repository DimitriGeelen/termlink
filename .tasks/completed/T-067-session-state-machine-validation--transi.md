---
id: T-067
name: "Session state machine validation — transition guards, Drop impl, TOCTOU fix"
description: >
  SessionState accepts any transition. Add valid_transition guard, Drop impl for cleanup
  on panic, fix TOCTOU race on display name.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:31Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-10T16:49:02Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:40Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-067: Session state machine validation — transition guards, Drop impl, TOCTOU fix

## Context

Session management issues found by reflection fleet session agent. State machine accepts invalid transitions, no Drop impl, TOCTOU race on display name. See [docs/reports/reflection-result-session.md].

## Acceptance Criteria

### Agent
- [x] `set_state()` has a `valid_transition()` guard that rejects invalid state transitions (e.g., Terminated → Active)
- [x] Invalid transition attempts return a typed error (not panic)
- [x] `Session` implements `Drop` with best-effort cleanup (remove socket file, remove JSON registration)
- [x] `register_in()` cleans up socket file if JSON write fails (no socket leak on error path)
- [x] TOCTOU race on display-name uniqueness is documented with a code comment explaining the risk and mitigation path
- [x] Unit tests cover: valid transitions succeed, invalid transitions are rejected, Drop cleans up files

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-session -- state 2>&1 | tail -5
grep -q "valid_transition" crates/termlink-session/src/lifecycle.rs
grep -q "fn drop" crates/termlink-session/src/manager.rs

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

### 2026-03-10T14:06:10Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T16:49:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
