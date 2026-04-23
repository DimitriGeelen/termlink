---
id: T-834
name: "Add termlink_register MCP tool — self-registration for AI agent sessions"
description: >
  Add termlink_register MCP tool — self-registration for AI agent sessions

status: work-completed
workflow_type: build
owner: human
horizon: next
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-03T22:26:54Z
last_update: 2026-04-22T04:52:51Z
date_finished: 2026-04-05T07:54:26Z
---

# T-834: Add termlink_register MCP tool — self-registration for AI agent sessions

## Context

Original blocker: `register --self` is blocking (runs until shutdown). Solved by using `Endpoint::run_background()` which returns an `EndpointHandle` that runs in a tokio background task. The MCP server holds handles in shared state — endpoints stay alive for the MCP server's lifetime and clean up on drop.

Also adds `termlink_deregister` to allow explicit cleanup of registered endpoints.

## Acceptance Criteria

### Agent
- [x] `termlink_register` MCP tool accepts name, roles, tags, cap parameters
- [x] Tool starts endpoint via `Endpoint::run_background()` and returns session ID immediately
- [x] Endpoint handles stored in `Arc<Mutex<Vec<EndpointHandle>>>` on `TermLinkTools`
- [x] `termlink_deregister` MCP tool accepts session ID and stops the matching endpoint
- [x] Unit tests for RegisterParams and DeregisterParams deserialization
- [x] All existing tests pass (`cargo test --workspace`)
- [x] Zero clippy warnings (`cargo clippy --workspace --all-targets`)

### Human
- [x] [REVIEW] Register an endpoint via MCP tool and verify it appears in `termlink list` — ticked by user direction 2026-04-23. Evidence: Live: `termlink doctor` shows 75 MCP tools including termlink_register. Code path: RegisterParams → handle_register → SessionRegistry. User direction 2026-04-23.
  **Steps:**
  1. Call `termlink_register` with `{"name": "test-agent", "tags": ["mcp-test"]}`
  2. Run `cd /opt/termlink && cargo run -- list --tag mcp-test`
  **Expected:** Session appears with name "test-agent" and tag "mcp-test"
  **If not:** Check MCP server logs for endpoint startup errors

## Verification

cargo test --workspace
cargo clippy --workspace --all-targets

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

### 2026-04-03T22:26:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-834-add-termlinkregister-mcp-tool--self-regi.md
- **Context:** Initial task creation

### 2026-04-03T22:27:36Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** register --self is blocking, not suitable for MCP tool

### 2026-04-03T22:27:47Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-05T07:54:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T05:40:15Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-22T04:52:51Z — status-update [task-update-agent]
- **Change:** horizon: later → next
