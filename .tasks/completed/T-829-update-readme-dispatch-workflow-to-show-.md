---
id: T-829
name: "Update README dispatch workflow to show termlink dispatch command"
description: >
  Update README dispatch workflow to show termlink dispatch command

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-03T20:56:09Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-04-03T20:57:26Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:11Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 5
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=5 (body:substrate-expand)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-829: Update README dispatch workflow to show termlink dispatch command

## Context

README "Dispatch parallel workers" section shows old manual spawn+wait pattern. Update to show `termlink dispatch` (atomic spawn+tag+collect) and `--isolate` for worktree isolation.

## Acceptance Criteria

### Agent
- [x] README dispatch section shows `termlink dispatch` as the primary approach
- [x] Shows `--isolate` option for git worktree isolation
- [x] Old manual pattern kept as collapsible alternative

## Verification

grep -q 'termlink dispatch' README.md
grep -q 'isolate' README.md

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

### 2026-04-03T20:56:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-829-update-readme-dispatch-workflow-to-show-.md
- **Context:** Initial task creation

### 2026-04-03T20:57:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
