---
id: T-783
name: "Add missing std::error::Error impl for NegotiateError"
description: >
  Add missing std::error::Error impl for NegotiateError

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-protocol/src/events.rs]
related_tasks: []
created: 2026-03-30T06:58:43Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T07:00:44Z
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
      tier: 2
      effort: 3
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-783: Add missing std::error::Error impl for NegotiateError

## Context

`NegotiateError` in `termlink-protocol/src/events.rs` implements `Display` but not `std::error::Error`. All other error types in the codebase implement `Error` (most via thiserror). This prevents using `NegotiateError` with `?` in contexts expecting `Box<dyn Error>`.

## Acceptance Criteria

### Agent
- [x] `NegotiateError` implements `std::error::Error`
- [x] Test verifies `NegotiateError` can be used as `Box<dyn std::error::Error>`
- [x] All `cargo test -p termlink-protocol` tests pass

## Verification

grep -q "impl std::error::Error for NegotiateError" crates/termlink-protocol/src/events.rs
cargo test -p termlink-protocol 2>&1 | tail -3

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

### 2026-03-30T06:58:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-783-add-missing-stderrorerror-impl-for-negot.md
- **Context:** Initial task creation

### 2026-03-30T07:00:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
