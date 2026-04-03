---
id: T-827
name: "Add termlink_pty_mode MCP tool — query terminal mode for AI agent interaction decisions"
description: >
  Add termlink_pty_mode MCP tool — query terminal mode for AI agent interaction decisions

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-03T20:37:43Z
last_update: 2026-04-03T20:37:43Z
date_finished: null
---

# T-827: Add termlink_pty_mode MCP tool — query terminal mode for AI agent interaction decisions

## Context

Expose `pty.mode` RPC as MCP tool. Lets AI agents check terminal state (canonical/echo/raw/alternate_screen) before interacting with sessions. 32nd MCP tool.

## Acceptance Criteria

### Agent
- [x] `termlink_pty_mode` MCP tool calls `pty.mode` RPC on target session
- [x] Returns JSON with canonical, echo, raw, alternate_screen boolean fields
- [x] Returns clear error for non-PTY sessions
- [x] Integration test: pty_mode on non-PTY session returns error + nonexistent session test
- [x] `cargo test -p termlink-mcp` passes (54 tests)
- [x] `cargo clippy --workspace` clean (0 warnings)

## Verification

cargo test -p termlink-mcp
cargo clippy --workspace
grep -q 'termlink_pty_mode' crates/termlink-mcp/src/tools.rs

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

### 2026-04-03T20:37:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-827-add-termlinkptymode-mcp-tool--query-term.md
- **Context:** Initial task creation
