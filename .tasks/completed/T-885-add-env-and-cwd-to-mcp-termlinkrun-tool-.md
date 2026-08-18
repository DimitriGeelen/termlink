---
id: T-885
name: "Add env and cwd to MCP termlink_run tool — run commands with custom environment"
description: >
  Add env and cwd to MCP termlink_run tool — run commands with custom environment

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T07:14:28Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-05T07:17:21Z
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
      effort: 7
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-885: Add env and cwd to MCP termlink_run tool — run commands with custom environment

## Context

MCP `termlink_run` lacks `env` and `cwd` parameters that exist on `termlink_exec`. AI agents using `run` for ephemeral commands can't set environment or working directory.

## Acceptance Criteria

### Agent
- [x] `RunParams` has `env: Option<HashMap<String, String>>` field
- [x] `RunParams` has `cwd: Option<String>` field
- [x] `termlink_run` passes env vars to executor
- [x] `termlink_run` passes cwd to executor
- [x] Unit test for RunParams with new fields
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

### 2026-04-05T07:14:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-885-add-env-and-cwd-to-mcp-termlinkrun-tool-.md
- **Context:** Initial task creation

### 2026-04-05T07:17:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
