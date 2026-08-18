---
id: T-844
name: "Add MCP integration tests for untested tools — signal, output, broadcast, emit_to,
  inject"
description: >
  Add MCP integration tests for untested tools — signal, output, broadcast, emit_to,
  inject

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-04T09:09:42Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T09:14:05Z
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
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-844: Add MCP integration tests for untested tools — signal, output, broadcast, emit_to, inject

## Context

7 of 38 MCP tools have no integration tests: broadcast, emit_to, exec, inject, output, signal, spawn. Add error-path and basic happy-path tests for the feasible ones (signal, output, broadcast, emit_to, inject).

## Acceptance Criteria

### Agent
- [x] Integration test for `termlink_signal` with nonexistent session
- [x] Integration test for `termlink_output` with nonexistent session
- [x] Integration test for `termlink_broadcast` when hub not running
- [x] Integration test for `termlink_emit_to` when hub not running
- [x] Integration test for `termlink_inject` with nonexistent session
- [x] Integration test for `termlink_output` with non-PTY session
- [x] Integration test for `termlink_inject` with registered session
- [x] `cargo test --workspace` passes (787 tests)
- [x] `cargo clippy --workspace --all-targets` has no warnings

## Verification

cargo test -p termlink-mcp --test mcp_integration 2>&1 | tail -5
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c 'warning:')" = "0"

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

### 2026-04-04T09:09:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-844-add-mcp-integration-tests-for-untested-t.md
- **Context:** Initial task creation

### 2026-04-04T09:14:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
