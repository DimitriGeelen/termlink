---
id: T-884
name: "Add --env flag to spawn CLI and MCP tool — pass environment variables to sessions"
description: >
  Add --env flag to spawn CLI and MCP tool — pass environment variables to sessions

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/execution.rs, 
      crates/termlink-cli/src/main.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T07:06:06Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-05T07:13:22Z
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
      blast_radius: 5
      tier: 2
      effort: 6
    rationale: blast_radius=5 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-884: Add --env flag to spawn CLI and MCP tool — pass environment variables to sessions

## Context

Dispatch got `--env` in T-883 but `spawn` CLI and MCP tool lack it. AI agents need to pass env vars to spawned sessions for configuration.

## Acceptance Criteria

### Agent
- [x] CLI `Spawn` has `--env KEY=VALUE` flag (repeatable)
- [x] CLI spawn injects env vars as `export KEY=VALUE;` prefix in shell command
- [x] MCP `SpawnParams` has `env: Option<HashMap<String, String>>` field
- [x] MCP spawn injects env vars into shell command
- [x] `cargo build` succeeds
- [x] `cargo clippy --workspace --all-targets` has no new warnings

## Verification

cargo build
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

### 2026-04-05T07:06:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-884-add---env-flag-to-spawn-cli-and-mcp-tool.md
- **Context:** Initial task creation

### 2026-04-05T07:13:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
