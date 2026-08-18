---
id: T-877
name: "Add since_default to MCP termlink_collect tool — expose hub history replay
  to MCP consumers"
description: >
  Add since_default to MCP termlink_collect tool — expose hub history replay to MCP
  consumers

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-04T23:17:24Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T23:20:25Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:12Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 2
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=2 (body:default-change); 
      D4=3 (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
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

# T-877: Add since_default to MCP termlink_collect tool — expose hub history replay to MCP consumers

## Context

T-876 added `since_default` to the hub's event.collect RPC. Expose it in the MCP
`termlink_collect` tool so AI agents can replay event history without knowing session IDs.

## Acceptance Criteria

### Agent
- [x] `since_default` field added to `CollectParams` struct
- [x] MCP tool passes `since_default` to hub RPC when provided
- [x] Unit test for CollectParams with since_default (858 tests total)
- [x] `cargo clippy --workspace` passes with no warnings
- [x] `cargo test --workspace` passes (858 tests, 0 failures)

## Verification

cargo clippy --workspace 2>&1 | grep -v "^$" | tail -5 | grep -q "warning generated\|could not compile" && exit 1 || true
cargo test --workspace 2>&1 | tail -3 | grep -q "0 failed"

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

### 2026-04-04T23:17:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-877-add-sincedefault-to-mcp-termlinkcollect-.md
- **Context:** Initial task creation

### 2026-04-04T23:20:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
