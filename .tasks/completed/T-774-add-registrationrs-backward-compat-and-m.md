---
id: T-774
name: "Add registration.rs backward-compat and metadata tests"
description: >
  Add registration.rs backward-compat and metadata tests

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:33:23Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-29T23:34:45Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:09Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-774: Add registration.rs backward-compat and metadata tests

## Context

registration.rs has 9 tests. Missing coverage: SessionMetadata serde, token_secret/allowed_commands fields, RegistrationAddr Display, TCP address registration, invalid state transition via set_state.

## Acceptance Criteria

### Agent
- [x] Add test for SessionMetadata serde (all fields, optional omission)
- [x] Add test for token_secret field in registration JSON
- [x] Add test for allowed_commands field in registration JSON
- [x] Add test for RegistrationAddr Display trait
- [x] Add test for TCP address in registration (non-Unix transport)
- [x] Add test for invalid state transition via set_state returns error
- [x] All tests pass: `cargo test -p termlink-session -- registration` (17 passing)

## Verification

cargo test -p termlink-session -- registration --quiet

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

### 2026-03-29T23:33:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-774-add-registrationrs-backward-compat-and-m.md
- **Context:** Initial task creation

### 2026-03-29T23:34:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
