---
id: T-893
name: "Add cwd parameter to MCP termlink_spawn — initial working directory for spawned
  sessions"
description: >
  Add cwd parameter to MCP termlink_spawn — initial working directory for spawned
  sessions

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T08:02:50Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-05T08:05:23Z
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
      effort: 4
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-893: Add cwd parameter to MCP termlink_spawn — initial working directory for spawned sessions

## Context

MCP `termlink_run` and `termlink_dispatch` both support `cwd`/`workdir` but `termlink_spawn` doesn't. Adding for parity.

## Acceptance Criteria

### Agent
- [x] `SpawnParams` has `cwd: Option<String>` field
- [x] Spawn prepends `cd <cwd> &&` to shell command when cwd is set
- [x] Unit test for SpawnParams with cwd
- [x] All tests pass, zero clippy warnings

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

### 2026-04-05T08:02:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-893-add-cwd-parameter-to-mcp-termlinkspawn--.md
- **Context:** Initial task creation

### 2026-04-05T08:05:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
