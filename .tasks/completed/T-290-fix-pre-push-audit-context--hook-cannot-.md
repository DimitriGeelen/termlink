---
id: T-290
name: "Fix pre-push audit context — hook cannot find tasks dir, blocks git push"
description: >
  Fix pre-push audit context — hook cannot find tasks dir, blocks git push

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-25T22:49:16Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-25T23:08:03Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:01Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 4
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=4 (body:fw-audit-or-doctor); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-290: Fix pre-push audit context — hook cannot find tasks dir, blocks git push

## Context

Pre-push hook runs audit which fails to find .tasks/ — sources lib/paths.sh which doesn't exist in consumer projects. Blocks git push with false FAILs.

## Acceptance Criteria

### Agent
- [x] `git push origin main` succeeds without `--no-verify`
- [x] Pre-push hook correctly resolves PROJECT_ROOT and finds .tasks/
- [x] `fw audit` still passes

## Verification

# Verify push works (the actual AC)
git ls-remote origin main

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

### 2026-03-25T22:49:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-290-fix-pre-push-audit-context--hook-cannot-.md
- **Context:** Initial task creation

### 2026-03-25T23:08:03Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
