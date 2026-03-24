---
id: T-264
name: "MCP server for TermLink — termlink-mcp crate with rmcp"
description: >
  Build MCP server exposing TermLink as structured tools. New crates/termlink-mcp crate, rmcp SDK, stdio transport, 10 tools + 3 resources. From T-261 GO decision.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [mcp, cli, orchestration]
components: []
related_tasks: [T-261, T-233]
created: 2026-03-24T11:52:10Z
last_update: 2026-03-24T12:22:41Z
date_finished: 2026-03-24T12:22:41Z
---

# T-264: MCP server for TermLink — termlink-mcp crate with rmcp

## Context

From T-261 GO decision. See `docs/reports/T-261-mcp-server-inception.md`. New `crates/termlink-mcp` crate using `rmcp` SDK. Stdio transport for v1. Thin adapter: MCP tool calls → TermLink JSON-RPC over Unix sockets.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-mcp/` workspace crate with `rmcp` dependency
- [x] MCP server binary entry point via `termlink mcp serve` subcommand
- [x] Stdio transport (stdin/stdout JSON-RPC)
- [x] 11 tools: ping, list_sessions, status, exec, output, inject, signal, emit, emit_to, event_poll
- [x] Each tool validates params via schemars JsonSchema and returns structured results/errors
- [x] `cargo check -p termlink-mcp` compiles clean
- [x] Param validation via typed structs (JsonSchema-derived schemas, required/optional fields)

### Human
- [ ] [REVIEW] MCP server works with Claude Code
  **Steps:**
  1. Add to `.claude/settings.json`: `{"mcpServers": {"termlink": {"command": "termlink", "args": ["mcp", "serve"]}}}`
  2. Start Claude Code, verify TermLink tools appear in tool list
  3. Ask Claude to "list TermLink sessions" — verify it uses the MCP tool
  **Expected:** Tools discoverable, structured responses
  **If not:** Check `termlink mcp serve` runs without error when invoked manually

## Verification

/Users/dimidev32/.cargo/bin/cargo check -p termlink-mcp
grep -q "termlink_ping" crates/termlink-mcp/src/tools.rs

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

### 2026-03-24T11:52:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-264-mcp-server-for-termlink--termlink-mcp-cr.md
- **Context:** Initial task creation

### 2026-03-24T12:22:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
