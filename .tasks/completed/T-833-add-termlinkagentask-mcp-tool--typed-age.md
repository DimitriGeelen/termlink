---
id: T-833
name: "Add termlink_agent_ask MCP tool — typed agent-to-agent request/response"
description: >
  Add termlink_agent_ask MCP tool — typed agent-to-agent request/response

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-03T22:18:06Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-04-03T22:24:19Z
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
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-833: Add termlink_agent_ask MCP tool — typed agent-to-agent request/response

## Context

AI agents need to send typed requests to other sessions and wait for responses. Uses the agent protocol (agent.request → agent.response events). 35th MCP tool.

## Acceptance Criteria

### Agent
- [x] `termlink_agent_ask` tool added with AgentAskParams (target, action, params, timeout, from)
- [x] Emits agent.request, subscribes for agent.response with matching request_id
- [x] Integration test for nonexistent target error + timeout test
- [x] All tests pass (702), zero clippy warnings
- [x] ARCHITECTURE.md and CHANGELOG.md updated

## Verification

cargo test --workspace 2>&1 | tail -5
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c '^warning\[')" = "0"
grep -q "termlink_agent_ask" crates/termlink-mcp/src/tools.rs

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

### 2026-04-03T22:18:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-833-add-termlinkagentask-mcp-tool--typed-age.md
- **Context:** Initial task creation

### 2026-04-03T22:24:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
