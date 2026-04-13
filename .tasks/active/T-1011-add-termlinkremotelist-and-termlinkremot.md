---
id: T-1011
name: "Add termlink_remote_list and termlink_remote_exec MCP tools"
description: >
  Add termlink_remote_list and termlink_remote_exec MCP tools

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T09:54:29Z
last_update: 2026-04-13T09:54:29Z
date_finished: null
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
- [ ] [RUBBER-STAMP] Verify MCP tool count in `termlink doctor`
  **Steps:**
  1. `cd /opt/termlink && cargo run -- doctor`
  **Expected:** Shows 61 MCP tools
  **If not:** Check tool registration

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
