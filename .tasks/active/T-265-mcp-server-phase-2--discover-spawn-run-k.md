---
id: T-265
name: "MCP server phase 2 — discover, spawn, run, kv, broadcast, wait tools"
description: >
  MCP server phase 2 — discover, spawn, run, kv, broadcast, wait tools

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [mcp, cli]
components: []
related_tasks: []
created: 2026-03-24T17:52:21Z
last_update: 2026-03-24T17:52:21Z
date_finished: null
---

# T-265: MCP server phase 2 — discover, spawn, run, kv, broadcast, wait tools

## Context

Extends T-264 MCP server with 6 additional tools for AI agent workflows: discover (find sessions), spawn (create sessions), run (ephemeral execution), kv (key-value operations), broadcast (multi-session events), wait (event blocking). Total tools: 16.

## Acceptance Criteria

### Agent
- [x] `termlink_discover` tool — find sessions by tag, role, name filter
- [x] `termlink_spawn` tool — create new TermLink session (background backend for MCP)
- [x] `termlink_run` tool — execute command in ephemeral session, return output
- [x] `termlink_kv_get` / `termlink_kv_set` / `termlink_kv_list` / `termlink_kv_del` tools — per-session key-value store
- [x] `termlink_broadcast` tool — emit event to multiple sessions via hub
- [x] `termlink_wait` tool — block until event topic appears (with timeout)
- [x] `cargo check -p termlink-mcp` compiles clean
- [x] All param types derive JsonSchema for auto-schema generation

### Human
- [ ] [REVIEW] New MCP tools work with Claude Code
  **Steps:**
  1. Restart Claude Code with termlink MCP server configured
  2. Ask Claude to "discover sessions with tag 'agent'"
  3. Ask Claude to "set key 'status' to 'active' on session X"
  **Expected:** Tools appear in tool list, structured responses
  **If not:** Check `termlink mcp serve` runs without error

## Verification

/Users/dimidev32/.cargo/bin/cargo check -p termlink-mcp
grep -q "termlink_discover" crates/termlink-mcp/src/tools.rs
grep -q "termlink_spawn" crates/termlink-mcp/src/tools.rs
grep -q "termlink_broadcast" crates/termlink-mcp/src/tools.rs
grep -q "termlink_wait" crates/termlink-mcp/src/tools.rs
grep -q "termlink_kv_set" crates/termlink-mcp/src/tools.rs
grep -q "termlink_run" crates/termlink-mcp/src/tools.rs

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

### 2026-03-24T17:52:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-265-mcp-server-phase-2--discover-spawn-run-k.md
- **Context:** Initial task creation
