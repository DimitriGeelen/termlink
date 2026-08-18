---
id: T-799
name: "Add E2E test level10 for worktree dispatch isolation"
description: >
  Add E2E test level10 for worktree isolation dispatch

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-30T14:57:09Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T16:06:57Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:10Z'
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
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-799: Add E2E test level10 for worktree dispatch isolation

## Context

Level 10 E2E test exercises the worktree isolation dispatch feature (T-789): `--isolate` creates git worktrees per worker, `--auto-merge` merges branches back, `dispatch-status` reports manifest state.

## Acceptance Criteria

### Agent
- [x] tests/e2e/level10-worktree-isolation.sh exists and is executable
- [x] Test creates a temp git repo, dispatches with --isolate, verifies worktrees
- [x] Test verifies --auto-merge merges worker branches
- [x] Test verifies dispatch-status --json reports correct counts
- [x] Test follows the level9 pattern (setup.sh, report, cleanup)

## Verification

test -x tests/e2e/level10-worktree-isolation.sh
grep -q "isolate" tests/e2e/level10-worktree-isolation.sh
grep -q "auto-merge" tests/e2e/level10-worktree-isolation.sh
grep -q "dispatch-status" tests/e2e/level10-worktree-isolation.sh

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

### 2026-03-30T14:57:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-799-add-e2e-test-level10-for-worktree-dispat.md
- **Context:** Initial task creation

### 2026-03-30T16:04:20Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T16:06:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
