---
id: T-897
name: "Add integration tests for termlink_register and termlink_deregister MCP tools"
description: >
  Add integration tests for termlink_register and termlink_deregister MCP tools

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-05T08:31:49Z
last_update: '2026-08-18T18:59:23Z'
date_finished: 2026-04-05T08:35:04Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:13Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=3 
      (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:23Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-897: Add integration tests for termlink_register and termlink_deregister MCP tools

## Context

T-834 added termlink_register and termlink_deregister MCP tools but only has unit tests for params deserialization. Need integration tests verifying actual endpoint lifecycle.

## Acceptance Criteria

### Agent
- [x] Integration test: register creates an endpoint that appears in list_sessions
- [x] Integration test: deregister removes the endpoint
- [x] Integration test: deregister with invalid ID returns error
- [x] Integration test: register with no parameters (minimal)
- [x] All tests pass (881 total)

## Verification

cargo test --workspace
cargo clippy --workspace --all-targets

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

### 2026-04-05T08:31:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-897-add-integration-tests-for-termlinkregist.md
- **Context:** Initial task creation

### 2026-04-05T08:35:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
