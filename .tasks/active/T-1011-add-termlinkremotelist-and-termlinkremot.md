---
id: T-1011
name: "Add termlink_remote_list and termlink_remote_exec MCP tools"
description: >
  Add termlink_remote_list and termlink_remote_exec MCP tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-13T09:54:29Z
last_update: 2026-04-15T13:47:07Z
date_finished: 2026-04-13T09:57:26Z
---

# T-1011: Add termlink_remote_list and termlink_remote_exec MCP tools

## Context

Add termlink_remote_list (discover sessions on remote hub) and termlink_remote_exec (run commands on remote sessions) MCP tools. These are high-value for agent orchestration — agents can discover and interact with sessions on other machines. MCP tool count: 59 -> 61.

## Acceptance Criteria

### Agent
- [x] Add termlink_remote_list MCP tool with hub, name, tags, roles params
- [x] Add termlink_remote_exec MCP tool with hub, session, command params
- [x] MCP tool count increases from 59 to 61
- [x] cargo clippy --workspace passes (0 warnings)
- [x] cargo test --workspace passes (1003 tests)

### Human
- [x] [RUBBER-STAMP] Verify MCP tool count in `termlink doctor` — ticked by user direction 2026-04-23. Evidence: Live: cargo run -- doctor reports 75 MCP tools (≥ 61 baseline). Verified live 2026-04-23T17:30Z (termlink 0.9.354, 3387b084).
  **Steps:**
  1. `cd /opt/termlink && cargo run -- doctor`
  **Expected:** Shows 61 MCP tools
  **If not:** Check tool registration

  **Agent evidence (2026-04-15T17:40Z, commit 5d0eb9b9):** doctor reports `"67 MCP tools"` (≥61; subsequent tasks added more). Registration works. Human may tick + close.


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, remote-list-exec-mcp):** `termlink_remote_list` and `termlink_remote_exec` both present in crates/termlink-mcp/src/tools.rs. `termlink doctor` reports `69 MCP tools`. RUBBER-STAMPable.

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

### 2026-04-13T09:54:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1011-add-termlinkremotelist-and-termlinkremot.md
- **Context:** Initial task creation

### 2026-04-13T09:57:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T19:00:39Z — programmatic-evidence [T-1087]
- **Evidence:** termlink doctor reports 67 MCP tools (includes termlink_remote_list, termlink_remote_exec)
- **Verified by:** automated command execution

