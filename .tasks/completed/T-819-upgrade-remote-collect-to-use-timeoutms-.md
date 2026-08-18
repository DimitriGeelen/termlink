---
id: T-819
name: "Upgrade remote collect to use timeout_ms for push-based delivery"
description: >
  Upgrade remote collect to use timeout_ms for push-based delivery

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-03-30T20:18:11Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T20:19:55Z
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
      blast_radius: 1
      tier: 2
      effort: 3
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-819: Upgrade remote collect to use timeout_ms for push-based delivery

## Context

Remote `cmd_remote_collect` uses sleep loop with `event.collect`. Pass `timeout_ms` for push-based delivery.

## Acceptance Criteria

### Agent
- [x] Remote collect passes `timeout_ms` to `event.collect`
- [x] Sleep removed from remote collect loop
- [x] `cargo check -p termlink` passes

## Verification

cargo check -p termlink 2>&1 | grep -q "Finished"

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

### 2026-03-30T20:18:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-819-upgrade-remote-collect-to-use-timeoutms-.md
- **Context:** Initial task creation

### 2026-03-30T20:19:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
