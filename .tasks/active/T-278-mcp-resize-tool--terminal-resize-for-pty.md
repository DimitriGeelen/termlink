---
id: T-278
name: "MCP resize tool — terminal resize for PTY sessions"
description: >
  MCP resize tool — terminal resize for PTY sessions

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T13:56:52Z
last_update: 2026-03-25T13:56:52Z
date_finished: null
---

# T-278: MCP resize tool — terminal resize for PTY sessions

## Context

Add `termlink_resize` MCP tool — resize PTY terminal dimensions. Simple wrapper around `command.resize` RPC. Useful for AI agents that need specific terminal widths for parsing output.

## Acceptance Criteria

### Agent
- [x] `termlink_resize` tool exists with cols/rows params
- [x] MCP integration tests pass (2 new: non-pty error, nonexistent session)
- [x] All tests pass (cargo test --workspace — 472 pass, 0 fail)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace
grep -q "termlink_resize" crates/termlink-mcp/src/tools.rs

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

### 2026-03-25T13:56:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-278-mcp-resize-tool--terminal-resize-for-pty.md
- **Context:** Initial task creation
