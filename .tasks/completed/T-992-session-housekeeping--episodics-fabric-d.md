---
id: T-992
name: "Session housekeeping — episodics, fabric drift, stale tasks"
description: >
  Session housekeeping — episodics, fabric drift, stale tasks

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-13T06:11:18Z
last_update: '2026-08-18T18:59:24Z'
date_finished: 2026-04-13T06:14:09Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:16Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 4
      D3: 0
      D4: 0
      F-RECALL: 1
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=4 (body:fw-audit-or-doctor); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=1 (body:episodic-only); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:24Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-992: Session housekeeping — episodics, fabric drift, stale tasks

## Context

Address audit warnings: 2 missing episodics (T-990, T-991), fabric drift (3 files), stale/bypassed tasks (T-854, T-931, T-243, T-260).

## Acceptance Criteria

### Agent
- [x] Episodic summaries generated for T-990 and T-991
- [x] Fabric drift resolved (fw fabric drift shows 0 unregistered)
- [x] T-854 and T-931 placeholder ACs backfilled and checked
- [x] T-243 and T-260 reviewed (both already horizon: later; T-260 now has verification section)

## Verification

test -f .context/episodic/T-990.yaml
test -f .context/episodic/T-991.yaml

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

### 2026-04-13T06:11:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-992-session-housekeeping--episodics-fabric-d.md
- **Context:** Initial task creation

### 2026-04-13T06:14:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
