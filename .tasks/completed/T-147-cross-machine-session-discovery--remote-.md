---
id: T-147
name: "Cross-machine session discovery + remote liveness"
description: >
  Cross-machine discovery combining local FS and remote sessions

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [tcp, hub]
components: []
related_tasks: []
created: 2026-03-15T22:06:27Z
last_update: '2026-08-18T18:58:50Z'
date_finished: 2026-03-15T22:50:11Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:00Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 4
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=4 
      (body:cross-machine); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:50Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-147: Cross-machine session discovery + remote liveness

## Context

Final piece of TCP hub story. Hub can forward requests to remote (TCP) sessions,
router resolves remote entries, integration test proves full E2E flow.

## Acceptance Criteria

### Agent
- [x] `forward_to_target` resolves remote sessions from the store (not just local FS)
- [x] `resolve_target` looks up remote entries by ID or display name
- [x] Integration test: register remote → discover → forward ping via TCP proxy
- [x] Integration test: register/heartbeat/deregister lifecycle
- [x] All 264 tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace

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

### 2026-03-15T22:06:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-147-cross-machine-session-discovery--remote-.md
- **Context:** Initial task creation

### 2026-03-15T22:16:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-15T22:50:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
