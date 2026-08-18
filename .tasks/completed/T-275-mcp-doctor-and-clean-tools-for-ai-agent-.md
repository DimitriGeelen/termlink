---
id: T-275
name: "MCP doctor and clean tools for AI agent self-healing"
description: >
  MCP doctor and clean tools for AI agent self-healing

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-25T13:07:00Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-03-25T13:12:36Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:58Z'
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
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-275: MCP doctor and clean tools for AI agent self-healing

## Context

Add `termlink_doctor` and `termlink_clean` MCP tools so AI agents can self-diagnose and remediate TermLink environment issues without CLI access. Complements T-273/T-274 CLI doctor.

## Acceptance Criteria

### Agent
- [x] `termlink_doctor` tool exists and returns structured JSON health report (checks, summary with pass/warn/fail counts)
- [x] `termlink_clean` tool exists and removes stale sessions + orphaned sockets, returns count of cleaned items
- [x] MCP integration tests for both tools pass (4 new tests)
- [x] All existing tests pass (cargo test --workspace — 463 pass, 0 fail)
- [x] `termlink_list_tools` test updated to expect 22+ tools

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace
grep -q "termlink_doctor" crates/termlink-mcp/src/tools.rs
grep -q "termlink_clean" crates/termlink-mcp/src/tools.rs
grep -q "test_doctor" crates/termlink-mcp/tests/mcp_integration.rs
grep -q "test_clean" crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-03-25T13:07:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-275-mcp-doctor-and-clean-tools-for-ai-agent-.md
- **Context:** Initial task creation

### 2026-03-25T13:12:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
