---
id: T-887
name: "Add termlink_batch_run MCP tool — parallel ephemeral execution across N sessions"
description: >
  Add termlink_batch_run MCP tool — parallel ephemeral execution across N sessions

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T07:21:51Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-05T07:26:39Z
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
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 6
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-887: Add termlink_batch_run MCP tool — parallel ephemeral execution across N sessions

## Context

`batch_exec` runs on existing sessions. `batch_run` runs commands in parallel ephemeral sessions — useful for parallelizing independent tasks without pre-spawning sessions. Takes multiple commands and returns aggregated results.

## Acceptance Criteria

### Agent
- [x] `BatchRunParams` struct with `commands: Vec<String>`, `timeout`, `cwd`, `env`, `max_parallel`
- [x] `termlink_batch_run` MCP tool spawns ephemeral sessions and collects results
- [x] Results include per-command stdout/stderr/exit_code
- [x] Unit test for BatchRunParams deserialization
- [x] `cargo build` succeeds
- [x] `cargo clippy -p termlink-mcp --all-targets` has no warnings

## Verification

cargo build
cargo clippy -p termlink-mcp --all-targets

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

### 2026-04-05T07:21:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-887-add-termlinkbatchrun-mcp-tool--parallel-.md
- **Context:** Initial task creation

### 2026-04-05T07:26:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
