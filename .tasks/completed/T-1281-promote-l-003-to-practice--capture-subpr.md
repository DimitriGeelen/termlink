---
id: T-1281
name: "Promote L-003 to practice — capture subprocess exit codes explicitly"
description: >
  Promote L-003 to practice — capture subprocess exit codes explicitly

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-25T21:32:44Z
last_update: '2026-08-18T18:58:47Z'
date_finished: 2026-04-25T21:33:33Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:52Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=2 (body:learning-ref); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:47Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-1281: Promote L-003 to practice — capture subprocess exit codes explicitly

## Context

L-003 (watcher scripts must capture subprocess exit codes explicitly — `||` swallows non-zero exits and causes false success reporting). 3 apps, D2 Reliability.

## Acceptance Criteria

### Agent
- [x] PP-010 entry created via `fw promote L-003 --directive D2`
- [x] L-003 application field references PP-010

## Verification

test -n "$(grep 'promoted_from: L-003' .context/project/practices.yaml)"
test -n "$(grep -A6 '^- id: L-003' .context/project/learnings.yaml | grep 'application:' | grep -oE 'PP-[0-9]+')"

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

### 2026-04-25T21:32:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1281-promote-l-003-to-practice--capture-subpr.md
- **Context:** Initial task creation

### 2026-04-25T21:33:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
