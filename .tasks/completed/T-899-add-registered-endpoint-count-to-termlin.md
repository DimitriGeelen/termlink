---
id: T-899
name: "Add registered endpoint count to termlink_info MCP tool output"
description: >
  Add registered endpoint count to termlink_info MCP tool output

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T08:45:12Z
last_update: '2026-08-18T18:59:23Z'
date_finished: 2026-04-05T08:47:52Z
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
      blast_radius: 1
      tier: 2
      effort: 3
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-899: Add registered endpoint count to termlink_info MCP tool output

## Context

The `termlink_info` tool shows system state but doesn't report how many MCP-registered endpoints are running (via termlink_register). Also add `mcp_tools` count to info output.

## Acceptance Criteria

### Agent
- [x] info output includes `registered_endpoints` count from shared state
- [x] info output includes `mcp_tools` count
- [x] All tests pass (881 total), zero clippy warnings

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

### 2026-04-05T08:45:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-899-add-registered-endpoint-count-to-termlin.md
- **Context:** Initial task creation

### 2026-04-05T08:47:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
