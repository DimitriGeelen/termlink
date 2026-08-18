---
id: T-162
name: "Add Agent tool vs TermLink dispatch guidance to framework specs"
description: >
  Add decision matrix for when to use Agent tool vs TermLink dispatch
  to framework pickup specs. Triggered by framework agent confusion.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [framework, docs]
components: []
related_tasks: [T-148, T-157]
created: 2026-03-17T23:20:33Z
last_update: '2026-08-18T18:58:53Z'
date_finished: 2026-03-17T23:21:27Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:06Z'
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
  - ts: '2026-08-18T18:58:53Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-162: Add Agent tool vs TermLink dispatch guidance to framework specs

## Context

Framework agent used Agent tool instead of TermLink for a task, reasoning that "TermLink doesn't have Edit tools." Partially correct but misses that TermLink can dispatch Claude Code workers that DO have Edit/Write.

## Acceptance Criteria

### Agent
- [x] T-148 spec has Agent tool vs TermLink dispatch decision matrix
- [x] Guidance added to CLAUDE.md section proposed in T-148

## Verification

grep -q "Agent tool" docs/specs/T-148-termlink-framework-integration.md
grep -q "TermLink dispatch" docs/specs/T-148-termlink-framework-integration.md

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

### 2026-03-17T23:20:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-162-add-agent-tool-vs-termlink-dispatch-guid.md
- **Context:** Initial task creation

### 2026-03-17T23:21:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
