---
id: T-831
name: "Add termlink_hub_status MCP tool — expose hub lifecycle state for AI agents"
description: >
  Add termlink_hub_status MCP tool — expose hub lifecycle state for AI agents

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-03T21:54:30Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-04-03T22:01:02Z
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
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-831: Add termlink_hub_status MCP tool — expose hub lifecycle state for AI agents

## Context

AI agents using MCP tools need to check hub state before calling hub-dependent tools (collect, broadcast). Currently they must use termlink_doctor or guess. Adds 33rd MCP tool.

## Acceptance Criteria

### Agent
- [x] `termlink_hub_status` tool added to tools.rs with name, description
- [x] Returns JSON with ok, status (running/not_running/stale), pid, socket, pidfile
- [x] Integration test for hub not running case
- [x] Integration test for hub running case
- [x] All tests pass, zero clippy warnings
- [x] ARCHITECTURE.md updated with 33 MCP tools and new test count
- [x] CHANGELOG.md updated

## Verification

cargo test --workspace 2>&1 | tail -5
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c '^warning\[')" = "0"
grep -q "termlink_hub_status" crates/termlink-mcp/src/tools.rs
grep -q "hub_status" crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-04-03T21:54:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-831-add-termlinkhubstatus-mcp-tool--expose-h.md
- **Context:** Initial task creation

### 2026-04-03T22:01:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
