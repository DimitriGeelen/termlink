---
id: T-776
name: "Update ARCHITECTURE.md stale test counts and command count"
description: >
  Update ARCHITECTURE.md stale test counts and command count

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:59:06Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T00:00:20Z
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
      tier: 3
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-776: Update ARCHITECTURE.md stale test counts and command count

## Context

ARCHITECTURE.md test coverage table shows 223 total tests (stale since early development). Actual count is 585. The CLI command count in the ASCII diagram says 28 but should be 30.

## Acceptance Criteria

### Agent
- [x] Test coverage table matches actual per-crate test counts from `cargo test --workspace`
- [x] CLI command count in ASCII diagram updated from 28 to 30
- [x] Total test count updated from 223 to current value

## Verification

grep -q "585" docs/ARCHITECTURE.md
grep -q "30 commands" docs/ARCHITECTURE.md

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

### 2026-03-29T23:59:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-776-update-architecturemd-stale-test-counts-.md
- **Context:** Initial task creation

### 2026-03-30T00:00:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
