---
id: T-1021
name: "Add termlink_remote_doctor MCP tool — health check remote hubs"
description: >
  Add MCP tool wrapper for the new termlink remote doctor command. Follows the MCP auto-exposure pattern (T-922): every CLI command must be MCP-reachable.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T12:20:13Z
last_update: 2026-04-19T12:12:00Z
date_finished: 2026-04-13T12:23:09Z
---

# T-1021: Add termlink_remote_doctor MCP tool — health check remote hubs

## Context

MCP auto-exposure pattern: wrap `termlink remote doctor` as MCP tool. Follows T-1010/T-1011 pattern.

## Acceptance Criteria

### Agent
- [x] `termlink_remote_doctor` MCP tool added with hub parameter
- [x] Tool uses `connect_remote_hub_mcp` helper for profile resolution
- [x] Returns JSON health check results
- [x] MCP tool count incremented (62 tools)
- [x] Builds and passes clippy

### Human
- [x] [RUBBER-STAMP] Verify MCP tool count in `termlink doctor` — ticked by user direction 2026-04-23. Evidence: Live: cargo run -- doctor reports 75 MCP tools (≥ 62 baseline). Verified live 2026-04-23T17:30Z (termlink 0.9.354, 3387b084).
  **Steps:**
  1. `cd /opt/termlink && cargo run -- doctor --json | python3 -c "import sys,json; print(json.load(sys.stdin)['checks'][-1]['message'])"`
  **Expected:** Shows "62 MCP tools"
  **If not:** Check tool registration in tools.rs

  **Agent evidence (2026-04-15T17:40Z):** Ran the command. Doctor reports `"termlink 0.9.10 (5d0eb9b9), 67 MCP tools"`. Count is 67 (≥62; higher because subsequent tasks T-1038/T-1040 added more remote tools). Registration plumbing works — spirit of AC satisfied. Human may tick the box and close.


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, remote-doctor-mcp):** `termlink_remote_doctor` present in crates/termlink-mcp/src/tools.rs. `termlink doctor` reports `69 MCP tools`. RUBBER-STAMPable.

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink-mcp -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"

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

### 2026-04-13T12:20:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1021-add-termlinkremotedoctor-mcp-tool--healt.md
- **Context:** Initial task creation

### 2026-04-13T12:23:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T19:00:39Z — programmatic-evidence [T-1087]
- **Evidence:** termlink doctor reports 67 MCP tools (includes termlink_remote_doctor)
- **Verified by:** automated command execution


### 2026-04-19T12:12:00Z — status-update [task-update-agent]
- **Change:** owner: agent → human
