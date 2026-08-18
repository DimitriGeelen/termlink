---
id: T-070
name: "Failure-mode e2e tests — crash recovery, orchestrator death, event loss"
description: >
  E2e tests for failure scenarios: specialist crash, orchestrator death mid-task,
  event ordering under load, graceful degradation.

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:43Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-10T20:01:42Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:40Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-070: Failure-mode e2e tests — crash recovery, orchestrator death, event loss

## Context

Testing gap found by reflection fleet e2e-suite and test-coverage agents. No tests for failure scenarios: specialist crash, orchestrator death, event loss. See [docs/reports/reflection-result-e2e.md] and [docs/reports/reflection-result-testcov.md].

## Acceptance Criteria

### Agent
- [x] E2e test: specialist watcher crash mid-task — orchestrator receives `task.failed` event
- [x] E2e test: orchestrator process killed mid-coordination — sessions continue, no orphan locks
- [x] E2e test: event ordering verified under concurrent emitters (3+ agents emitting simultaneously)
- [x] E2e test: `--since` cursor with stale/invalid value returns error or empty (not crash)
- [x] E2e test: session deregistration during active event polling — poller handles gracefully
- [x] All failure-mode tests have cleanup (no leftover processes or sockets)

## Verification

# Failure-mode test files exist
test -f tests/e2e/level7-failure-modes.sh
# Test script is executable
test -x tests/e2e/level7-failure-modes.sh

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

### 2026-03-10T08:44:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-070-failure-mode-e2e-tests--crash-recovery-o.md
- **Context:** Initial task creation

### 2026-03-10T18:06:23Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T20:01:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
