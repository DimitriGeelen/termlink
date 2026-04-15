---
id: T-1021
name: "Add termlink_remote_doctor MCP tool — health check remote hubs"
description: >
  Add MCP tool wrapper for the new termlink remote doctor command. Follows the MCP auto-exposure pattern (T-922): every CLI command must be MCP-reachable.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T12:20:13Z
last_update: 2026-04-15T17:41:32Z
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
- [ ] [RUBBER-STAMP] Verify MCP tool count in `termlink doctor`
  **Steps:**
  1. `cd /opt/termlink && cargo run -- doctor --json | python3 -c "import sys,json; print(json.load(sys.stdin)['checks'][-1]['message'])"`
  **Expected:** Shows "62 MCP tools"
  **If not:** Check tool registration in tools.rs

  **Agent evidence (2026-04-15T17:40Z):** Ran the command. Doctor reports `"termlink 0.9.10 (5d0eb9b9), 67 MCP tools"`. Count is 67 (≥62; higher because subsequent tasks T-1038/T-1040 added more remote tools). Registration plumbing works — spirit of AC satisfied. Human may tick the box and close.

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
