---
id: T-1010
name: "Add termlink_remote_inbox_status MCP tool — query inbox on remote hubs"
description: >
  Add termlink_remote_inbox_status MCP tool — query inbox on remote hubs

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-13T09:46:45Z
last_update: 2026-04-15T13:47:07Z
date_finished: 2026-04-13T09:50:15Z
---

# T-1010: Add termlink_remote_inbox_status MCP tool — query inbox on remote hubs

## Context

Expose T-1009's `termlink remote inbox` as MCP tools so AI agents can query inbox on remote hubs. Follows T-922 codification: every CLI command must be MCP-reachable.

## Acceptance Criteria

### Agent
- [x] Add termlink_remote_inbox_status MCP tool with hub param
- [x] Add termlink_remote_inbox_list MCP tool with hub + target params
- [x] Add termlink_remote_inbox_clear MCP tool with hub + target/all params
- [x] MCP tool count increases from 56 to 59
- [x] cargo clippy --workspace passes (0 warnings)
- [x] cargo test --workspace passes (1003 tests)

### Human
- [x] [RUBBER-STAMP] Verify MCP tool count in `termlink doctor` — ticked by user direction 2026-04-23. Evidence: Live: cargo run -- doctor reports 75 MCP tools (≥ 59 baseline). Verified live 2026-04-23T17:30Z (termlink 0.9.354, 3387b084).
  **Steps:**
  1. `cd /opt/termlink && cargo run -- doctor`
  **Expected:** Shows 59 MCP tools
  **If not:** Check tool registration macro

  **Agent evidence (2026-04-15T17:40Z, commit 5d0eb9b9):** doctor reports `"67 MCP tools"` (≥59; grew as T-1011/T-1021/T-1038/T-1040 added more). Registration works. Human may tick + close.


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, remote-inbox-status-mcp):** `termlink_remote_inbox_status` present in crates/termlink-mcp/src/tools.rs. `termlink doctor` reports `69 MCP tools`, up from the pre-series baseline. RUBBER-STAMPable.

## Verification

cargo clippy --workspace -- -D warnings 2>&1 | tail -1
cargo test --workspace 2>&1 | grep "^test result" | grep -v "0 passed"

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

### 2026-04-13T09:46:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1010-add-termlinkremoteinboxstatus-mcp-tool--.md
- **Context:** Initial task creation

### 2026-04-13T09:50:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T19:00:39Z — programmatic-evidence [T-1087]
- **Evidence:** termlink doctor reports 67 MCP tools (includes termlink_remote_inbox_status)
- **Verified by:** automated command execution

