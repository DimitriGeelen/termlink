---
id: T-1040
name: "Add termlink_hub_restart and termlink_events MCP tools — T-922 codification"
description: >
  Add MCP tools for hub restart and event history queries. Continues T-922 codification: every CLI command should be MCP-reachable.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-13T20:23:42Z
last_update: 2026-04-13T20:34:53Z
date_finished: null
---

# T-1040: Add termlink_hub_restart and termlink_events MCP tools — T-922 codification

## Context

T-922 codification: every CLI command should be MCP-reachable. The `hub restart` and `events` CLI commands have no MCP equivalents. `hub restart` restarts the local hub (process management). `events` queries event history from a local session via `event.poll` RPC.

## Acceptance Criteria

### Agent
- [x] `termlink_hub_restart` MCP tool restarts the local hub and returns JSON status
- [x] `termlink_events` MCP tool queries event history from a named session
- [x] MCP unit tests for both tools (params parsing, missing required fields)
- [x] Builds with zero clippy warnings

### Human
- [ ] [RUBBER-STAMP] Verify MCP tool count increased in `termlink doctor`
  **Steps:** `cd /opt/termlink && cargo run -- doctor 2>&1 | grep -i tool`
  **Expected:** Tool count increased by 2 compared to previous build
  **If not:** Check MCP tool registration in tools.rs tool_router macro


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, hub-restart-events-mcp):** `termlink_hub_restart` and `termlink_events` present in crates/termlink-mcp/src/tools.rs. `termlink doctor` reports `69 MCP tools`. RUBBER-STAMPable.

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink-mcp -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"
cargo test -p termlink-mcp hub_restart 2>&1 | grep "passed"
cargo test -p termlink-mcp events_params 2>&1 | grep "passed"

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

### 2026-04-13T20:23:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1040-add-termlinkhubrestart-and-termlinkevent.md
- **Context:** Initial task creation
