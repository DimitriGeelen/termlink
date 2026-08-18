---
id: T-021
name: "End-to-end integration tests — multi-session communication"
description: >
  End-to-end integration tests — multi-session communication

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T17:52:58Z
last_update: '2026-08-18T18:58:40Z'
date_finished: 2026-03-08T18:17:19Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:39Z'
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
  - ts: '2026-08-18T18:58:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-021: End-to-end integration tests — multi-session communication

## Context

Integration tests validating the full stack built in T-015 through T-019: two sessions communicating via Unix sockets, client/server JSON-RPC, command execution, discovery, and lifecycle.

## Acceptance Criteria

### Agent
- [x] Integration test file created in termlink-session crate
- [x] Test: two sessions register, one pings the other
- [x] Test: session A executes a command on session B
- [x] Test: discovery lists both sessions
- [x] Test: deregister cleans up, session no longer discoverable
- [x] All existing + new tests pass (`cargo test --workspace`) — 83 tests

## Verification

PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace

## Updates

### 2026-03-08T17:52:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-021-end-to-end-integration-tests--multi-sess.md
- **Context:** Initial task creation

### 2026-03-08T18:17:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
