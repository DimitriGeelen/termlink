---
id: T-269
name: "Fix hub router compiler warnings + cleanup"
description: >
  Fix hub router compiler warnings + cleanup

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-25T09:53:33Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-03-25T09:57:29Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:56Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 1
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=1 (body:log-or-error-line); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-269: Fix hub router compiler warnings + cleanup

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Zero compiler warnings in `cargo test --workspace`
- [x] All 451 tests pass
- [x] `skipped_count` used in orchestrator.route error message
## Verification

grep -q "skipped_count" crates/termlink-hub/src/router.rs

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

### 2026-03-25T09:53:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-269-fix-hub-router-compiler-warnings--cleanu.md
- **Context:** Initial task creation

### 2026-03-25T09:57:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
