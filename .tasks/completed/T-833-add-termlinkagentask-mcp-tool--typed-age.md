---
id: T-833
name: "Add termlink_agent_ask MCP tool — typed agent-to-agent request/response"
description: >
  Add termlink_agent_ask MCP tool — typed agent-to-agent request/response

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-03T22:18:06Z
last_update: 2026-04-03T22:24:19Z
date_finished: 2026-04-03T22:24:19Z
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
