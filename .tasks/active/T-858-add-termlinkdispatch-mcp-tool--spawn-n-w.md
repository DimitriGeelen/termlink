---
id: T-858
name: "Add termlink_send MCP tool — generic JSON-RPC call to any session"
description: >
  Add termlink_send MCP tool — generic JSON-RPC call to any session

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T19:12:57Z
last_update: 2026-04-04T19:12:57Z
date_finished: null
---

# T-858: Add termlink_dispatch MCP tool — spawn N workers and collect results

## Context

CLI has `termlink send <target> <method> -p <params>` for generic JSON-RPC calls to any session. No MCP equivalent exists. This is the lowest-level building block — lets AI agents call any RPC method on any session.

## Acceptance Criteria

### Agent
- [x] `termlink_send` MCP tool added with params: target, method, params (optional JSON string), timeout (optional)
- [x] Returns JSON response from the session (or error JSON on failure)
- [x] Integration test for successful send (using `termlink.ping`)
- [x] Integration test for send to nonexistent session + invalid JSON params
- [x] Tool appears in `termlink_list_tools` (42+ tools)
- [x] All tests pass: `cargo test -p termlink-mcp` (116 tests)
- [x] Zero clippy warnings

## Verification

cargo test -p termlink-mcp
cargo clippy -p termlink-mcp -- -D warnings

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

### 2026-04-04T19:12:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-858-add-termlinkdispatch-mcp-tool--spawn-n-w.md
- **Context:** Initial task creation
