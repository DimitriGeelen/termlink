---
id: T-835
name: "Add termlink_hub_start and termlink_hub_stop MCP tools — hub lifecycle management for AI agents"
description: >
  Add termlink_hub_start and termlink_hub_stop MCP tools — hub lifecycle management for AI agents

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-03T22:27:55Z
last_update: 2026-04-03T22:33:28Z
date_finished: 2026-04-03T22:33:28Z
---

# T-835: Add termlink_hub_start and termlink_hub_stop MCP tools — hub lifecycle management for AI agents

## Context

AI agents need to start/stop the hub before using collect/broadcast tools. hub_start spawns the hub in a background tokio task. hub_stop sends SIGTERM. 36th+37th MCP tools.

## Acceptance Criteria

### Agent
- [x] `termlink_hub_start` tool spawns hub in background, returns pid/socket
- [x] `termlink_hub_stop` tool sends SIGTERM to running hub, cleans up stale
- [x] Integration test for start then status (hub running) + already_running + stale cleanup
- [x] Integration test for stop when not running
- [x] All tests pass (705), zero clippy warnings
- [x] ARCHITECTURE.md and CHANGELOG.md updated

## Verification

cargo test --workspace 2>&1 | tail -5
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c '^warning\[')" = "0"
grep -q "termlink_hub_start" crates/termlink-mcp/src/tools.rs
grep -q "termlink_hub_stop" crates/termlink-mcp/src/tools.rs

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

### 2026-04-03T22:27:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-835-add-termlinkhubstart-and-termlinkhubstop.md
- **Context:** Initial task creation

### 2026-04-03T22:33:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
