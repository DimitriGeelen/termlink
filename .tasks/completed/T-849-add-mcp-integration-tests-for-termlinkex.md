---
id: T-849
name: "Add MCP integration tests for termlink_exec and termlink_spawn"
description: >
  Add MCP integration tests for termlink_exec and termlink_spawn

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-04T15:13:56Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T15:22:18Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:11Z'
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
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-849: Add MCP integration tests for termlink_exec and termlink_spawn

## Context

The only 2 MCP tools without call() integration tests. termlink_exec runs a command on a session, termlink_spawn creates a new terminal session.

## Acceptance Criteria

### Agent
- [x] Integration test for termlink_exec: runs `echo hello-mcp-exec` on a session, verifies stdout contains output
- [x] Integration test for termlink_exec: nonexistent session returns error with "not found"
- [x] Integration test for termlink_spawn: spawns with background backend, verifies it returns non-empty response
- [x] All 40 MCP tools now have call() integration tests (81 integration tests total)
- [x] All tests pass: cargo test -p termlink-mcp (109 tests)
- [x] Zero clippy warnings: cargo clippy -p termlink-mcp

## Verification

cargo test -p termlink-mcp 2>&1 | tail -3
cargo clippy -p termlink-mcp -- -D warnings 2>&1 | tail -3

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

### 2026-04-04T15:13:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-849-add-mcp-integration-tests-for-termlinkex.md
- **Context:** Initial task creation

### 2026-04-04T15:22:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
