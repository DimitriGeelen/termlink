---
id: T-898
name: "Fix misplaced doc comments in protocol crate — format_age docs on wrong constant"
description: >
  Fix misplaced doc comments in protocol crate — format_age docs on wrong constant

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-protocol/src/lib.rs]
related_tasks: []
created: 2026-04-05T08:41:09Z
last_update: '2026-08-18T18:59:23Z'
date_finished: 2026-04-05T08:42:33Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:13Z'
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
  - ts: '2026-08-18T18:59:23Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 3
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-898: Fix misplaced doc comments in protocol crate — format_age docs on wrong constant

## Context

T-881 moved constants before `#[cfg(test)]` but left `format_age` doc comment merged with `DATA_PLANE_VERSION` doc. Fix so each item has its own correct doc comment.

## Acceptance Criteria

### Agent
- [x] `DATA_PLANE_VERSION` has its own correct doc comment
- [x] `format_age` function has its own correct doc comment
- [x] All tests pass, zero clippy warnings

## Verification

cargo test -p termlink-protocol
cargo clippy -p termlink-protocol --all-targets

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

### 2026-04-05T08:41:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-898-fix-misplaced-doc-comments-in-protocol-c.md
- **Context:** Initial task creation

### 2026-04-05T08:42:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
