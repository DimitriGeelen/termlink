---
id: T-881
name: "Fix clippy warning in termlink-protocol — items after test module"
description: >
  Fix clippy warning in termlink-protocol — items after test module

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: [crates/termlink-protocol/src/lib.rs]
related_tasks: []
created: 2026-04-05T06:42:15Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-05T06:47:21Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:12Z'
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
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 3
      effort: 3
    rationale: blast_radius=1 (no-signal); tier=3 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-881: Fix clippy warning in termlink-protocol — items after test module

## Context

Clippy warns about `items_after_test_module` in protocol crate — constants defined after `#[cfg(test)] mod tests`. Move constants before the test module.

## Acceptance Criteria

### Agent
- [x] Constants `DATA_PLANE_VERSION`, `FRAME_MAGIC`, `FRAME_HEADER_SIZE`, `MAX_PAYLOAD_SIZE` appear before `#[cfg(test)]` in lib.rs
- [x] `cargo clippy -p termlink-protocol --all-targets` produces zero warnings
- [x] `cargo test -p termlink-protocol` passes

## Verification

cargo clippy -p termlink-protocol --all-targets
cargo test -p termlink-protocol

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

### 2026-04-05T06:42:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-881-fix-clippy-warning-in-termlink-protocol-.md
- **Context:** Initial task creation

### 2026-04-05T06:47:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
