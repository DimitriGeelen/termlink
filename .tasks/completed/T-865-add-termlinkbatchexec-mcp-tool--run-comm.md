---
id: T-865
name: "Add termlink_batch_exec MCP tool — run command across filtered sessions"
description: >
  Add termlink_batch_exec MCP tool — run command across filtered sessions

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-04T21:16:22Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T21:26:11Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:12Z'
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
      tier: 2
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-865: Add termlink_batch_exec MCP tool — run command across filtered sessions

## Context

Run a command across multiple sessions matching a filter (tag/role/name), returning per-session results with stdout/stderr/exit_code. Combines discovery + exec into one atomic operation for orchestration.

## Acceptance Criteria

### Agent
- [x] New `BatchExecParams` struct with `tag`, `role`, `name` filters, `command`, `timeout`, and `max_parallel`
- [x] `termlink_batch_exec` MCP tool filters sessions, runs command concurrently, returns per-session results
- [x] Returns JSON `{ok: true, results: [{session, display_name, stdout, stderr, exit_code}, ...], total, succeeded, failed}`
- [x] Handles partial failures (some sessions fail, others succeed)
- [x] Unit test for BatchExecParams deserialization
- [x] MCP integration test: batch exec with name filter + empty results
- [x] Zero clippy warnings

## Verification

grep -q 'termlink_batch_exec' crates/termlink-mcp/src/tools.rs
grep -q 'BatchExecParams' crates/termlink-mcp/src/tools.rs
grep -q 'test_batch_exec' crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-04-04T21:16:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-865-add-termlinkbatchexec-mcp-tool--run-comm.md
- **Context:** Initial task creation

### 2026-04-04T21:26:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
