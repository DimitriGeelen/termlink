---
id: T-154
name: "Update tl-dispatch.sh for multi-backend spawn"
description: >
  Delegate spawn to termlink spawn command, simplify cleanup per backend

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-16T05:45:36Z
last_update: '2026-08-18T18:58:51Z'
date_finished: 2026-03-16T06:45:30Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:02Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=1 (body:hand-wired-dispatch)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:51Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-154: Update tl-dispatch.sh for multi-backend spawn

## Context

Replace osascript-only spawn in tl-dispatch.sh with `termlink spawn --backend auto`. Simplify cleanup.

## Acceptance Criteria

### Agent
- [x] cmd_spawn uses `termlink spawn --backend auto` instead of direct osascript
- [x] cmd_cleanup detects backend and uses appropriate cleanup (tmux kill-session / kill PID / osascript 3-phase)
- [x] --backend flag added to tl-dispatch.sh for explicit override (+ TL_DISPATCH_BACKEND env)
- [x] Script still works on macOS (backward compatible — Terminal.app cleanup preserved for window_id)
- [x] Script syntax valid (bash -n check)

## Verification

bash -n scripts/tl-dispatch.sh
grep -q "termlink spawn" scripts/tl-dispatch.sh
grep -q "backend" scripts/tl-dispatch.sh

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

### 2026-03-16T05:45:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-154-update-tl-dispatchsh-for-multi-backend-s.md
- **Context:** Initial task creation

### 2026-03-16T06:35:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-16T06:45:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
