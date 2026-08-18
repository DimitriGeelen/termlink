---
id: T-277
name: "MCP request tool — request-reply pattern for inter-session coordination"
description: >
  MCP request tool — request-reply pattern for inter-session coordination

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-25T13:52:02Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-03-25T13:55:16Z
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
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-277: MCP request tool — request-reply pattern for inter-session coordination

## Context

Add `termlink_request` MCP tool — emit an event to a session and wait for a reply on a specified topic. Enables request-reply coordination between AI agent sessions. Mirrors CLI `termlink request` command.

## Acceptance Criteria

### Agent
- [x] `termlink_request` tool emits event with request_id and polls for matching reply
- [x] MCP integration tests pass (3 new: nonexistent, with-reply, timeout)
- [x] All tests pass (cargo test --workspace — 470 pass, 0 fail)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace
grep -q "termlink_request" crates/termlink-mcp/src/tools.rs
grep -q "test_request" crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-03-25T13:52:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-277-mcp-request-tool--request-reply-pattern-.md
- **Context:** Initial task creation

### 2026-03-25T13:55:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
