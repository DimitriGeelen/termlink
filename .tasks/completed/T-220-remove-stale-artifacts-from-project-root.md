---
id: T-220
name: "Remove stale artifacts from project root"
description: >
  Remove stale artifacts from project root

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-22T21:23:54Z
last_update: '2026-08-18T18:59:05Z'
date_finished: 2026-03-22T21:24:41Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:33Z'
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
  - ts: '2026-08-18T18:59:05Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-220: Remove stale artifacts from project root

## Context

Stale test artifacts in project root: `sim-spike-test.txt` (T-192 spike, 1 line) and `CLAUDE.md.bak` (old backup from T-140 framework upgrade).

## Acceptance Criteria

### Agent
- [x] `sim-spike-test.txt` removed from tracked files
- [x] `CLAUDE.md.bak` removed from tracked files

## Verification

! test -f sim-spike-test.txt
! test -f CLAUDE.md.bak

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

### 2026-03-22T21:23:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-220-remove-stale-artifacts-from-project-root.md
- **Context:** Initial task creation

### 2026-03-22T21:24:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
