---
id: T-784
name: "Add handler unit tests — KV store, session update, ping, dispatch routing"
description: >
  Add handler unit tests — KV store, session update, ping, dispatch routing

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: [crates/termlink-session/src/handler.rs]
related_tasks: []
created: 2026-03-30T07:05:57Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T07:11:05Z
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
      blast_radius: 1
      tier: 1
      effort: 5
    rationale: blast_radius=1 (no-signal); tier=1 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-784: Add handler unit tests — KV store, session update, ping, dispatch routing

## Context

`handler.rs` has 41 tests but misses: KV error parameter validation, session.update with roles, dispatch_mut fallthrough to immutable dispatch, and dispatch_mut notification handling.

## Acceptance Criteria

### Agent
- [x] KV error cases: kv.set/get/delete with missing key parameter
- [x] Session update with roles test
- [x] dispatch_mut falls through to immutable dispatch for read methods
- [x] dispatch_mut returns None for notifications
- [x] All tests pass with `cargo test -p termlink-session`

## Verification

grep -q "kv_set_missing_key_returns_error" crates/termlink-session/src/handler.rs
grep -q "dispatch_mut_falls_through_to_immutable" crates/termlink-session/src/handler.rs
grep -q "session_update_roles" crates/termlink-session/src/handler.rs

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

### 2026-03-30T07:05:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-784-add-handler-unit-tests--kv-store-session.md
- **Context:** Initial task creation

### 2026-03-30T07:11:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
