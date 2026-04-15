---
id: T-879
name: "Add termlink_dispatch MCP tool — atomic spawn+tag+collect for AI agent orchestration"
description: >
  Add termlink_dispatch MCP tool — atomic spawn+tag+collect for AI agent orchestration

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T06:03:46Z
last_update: 2026-04-15T15:27:20Z
date_finished: 2026-04-05T06:32:42Z
---

# T-879: Add termlink_dispatch MCP tool — atomic spawn+tag+collect for AI agent orchestration

## Context

CLI `termlink dispatch` exists but no MCP tool equivalent. AI agents using MCP can't orchestrate multi-worker dispatch. This adds `termlink_dispatch` MCP tool that spawns N workers, tags them, and collects results — the same atomic pattern as the CLI command but accessible to AI agents via MCP.

## Acceptance Criteria

### Agent
- [x] `termlink_dispatch` MCP tool exists in `crates/termlink-mcp/src/tools.rs`
- [x] Tool accepts: count, command, timeout, topic, name_prefix, roles, tags
- [x] Tool spawns N background workers with dispatch metadata tags
- [x] Tool collects events via hub and returns structured JSON results
- [x] Tool validates inputs (count >= 1, command non-empty, hub running)
- [x] Unit test for DispatchParams deserialization
- [x] `cargo test --workspace` passes
- [x] `cargo clippy --workspace --all-targets` passes with no warnings

### Human
- [ ] [REVIEW] Dispatch 3 workers via MCP tool and verify results collected
  **Steps:**
  1. Start hub: `cd /opt/termlink && cargo run -- hub start`
  2. Use MCP tool: call `termlink_dispatch` with `{"count": 3, "command": ["echo", "hello"], "timeout": 30}`
  3. Verify JSON response has `ok: true`, 3 results
  **Expected:** All 3 workers spawn, register, emit, and results collected
  **If not:** Check hub logs and worker registration

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

### 2026-04-05T06:03:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-879-add-termlinkdispatch-mcp-tool--atomic-sp.md
- **Context:** Initial task creation

### 2026-04-05T06:32:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
