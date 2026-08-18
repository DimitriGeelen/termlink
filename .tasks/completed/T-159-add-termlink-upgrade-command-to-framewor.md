---
id: T-159
name: "Add termlink upgrade command to framework pickup specs"
description: >
  Document the cargo install/upgrade command in framework pickup specs
  (T-148, T-157) so the framework agent knows how to install or upgrade TermLink.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [remote-access, framework]
components: []
related_tasks: [T-148, T-157]
created: 2026-03-17T20:20:50Z
last_update: '2026-08-18T18:58:52Z'
date_finished: 2026-03-17T20:21:49Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:04Z'
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
  - ts: '2026-08-18T18:58:52Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-159: Add termlink upgrade command to framework pickup specs

## Context

Framework agents need to know how to install/upgrade TermLink. Add the cargo command to both pickup specs.

## Acceptance Criteria

### Agent
- [x] T-148 spec has install/upgrade command section
- [x] T-157 pickup prompt has install/upgrade command
- [x] Commands reference both GitHub and local clone paths

## Verification

grep -q "cargo install" docs/specs/T-148-termlink-framework-integration.md
grep -q "cargo install" docs/specs/T-157-claude-fw-termlink-pickup.md

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

### 2026-03-17T20:20:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-159-add-termlink-upgrade-command-to-framewor.md
- **Context:** Initial task creation

### 2026-03-17T20:21:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
