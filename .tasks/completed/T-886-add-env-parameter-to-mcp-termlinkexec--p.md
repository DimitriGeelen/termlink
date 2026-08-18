---
id: T-886
name: "Add env parameter to MCP termlink_exec — pass environment variables to remote
  command execution"
description: >
  Add env parameter to MCP termlink_exec — pass environment variables to remote command
  execution

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T07:18:23Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-05T07:19:52Z
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
      effort: 3
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-886: Add env parameter to MCP termlink_exec — pass environment variables to remote command execution

## Context

Session handler already supports env in command.execute RPC but MCP ExecParams doesn't expose it.

## Acceptance Criteria

### Agent
- [x] `ExecParams` has `env: Option<HashMap<String, String>>` field
- [x] `termlink_exec` passes env to RPC params
- [x] `cargo build` succeeds

## Verification

cargo build

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

### 2026-04-05T07:18:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-886-add-env-parameter-to-mcp-termlinkexec--p.md
- **Context:** Initial task creation

### 2026-04-05T07:19:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
