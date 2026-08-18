---
id: T-276
name: "MCP tag tool — dynamic session tagging for AI orchestration"
description: >
  MCP tag tool — dynamic session tagging for AI orchestration

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-25T13:33:11Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-03-25T13:36:07Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:59Z'
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
  - ts: '2026-08-18T18:59:16Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-276: MCP tag tool — dynamic session tagging for AI orchestration

## Context

Add `termlink_tag` MCP tool so AI agents can dynamically tag/untag sessions for orchestration. Uses `session.update` RPC with `add_tags`/`remove_tags`/`tags` params.

## Acceptance Criteria

### Agent
- [x] `termlink_tag` tool exists with set/add/remove operations
- [x] MCP integration tests for tag tool pass (4 new tests)
- [x] All tests pass (cargo test --workspace — 467 pass, 0 fail)
- [x] Tool count test updated to 23+

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace
grep -q "termlink_tag" crates/termlink-mcp/src/tools.rs
grep -q "test_tag" crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-03-25T13:33:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-276-mcp-tag-tool--dynamic-session-tagging-fo.md
- **Context:** Initial task creation

### 2026-03-25T13:36:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
