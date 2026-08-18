---
id: T-892
name: "Add workdir parameter to MCP termlink_dispatch — worker directory control"
description: >
  Add workdir parameter to MCP termlink_dispatch — worker directory control

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T07:57:03Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-05T08:00:40Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:13Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=3 
      (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=1 
      (body:hand-wired-dispatch)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 5
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-892: Add workdir parameter to MCP termlink_dispatch — worker directory control

## Context

The CLI `termlink dispatch --workdir <path>` lets workers cd into a directory before executing. The MCP `termlink_dispatch` tool lacks this parameter. Adding it for parity.

## Acceptance Criteria

### Agent
- [x] `DispatchParams` has `workdir: Option<String>` field
- [x] MCP dispatch prepends `cd <workdir> &&` to shell command when workdir is set
- [x] Unit test for DispatchParams with workdir field
- [x] All tests pass (`cargo test --workspace`)
- [x] Zero clippy warnings

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

### 2026-04-05T07:57:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-892-add-workdir-parameter-to-mcp-termlinkdis.md
- **Context:** Initial task creation

### 2026-04-05T08:00:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
