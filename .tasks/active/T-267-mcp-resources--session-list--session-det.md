---
id: T-267
name: "MCP resources — session list + session detail as read-only data"
description: >
  MCP resources — session list + session detail as read-only data

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [mcp]
components: []
related_tasks: []
created: 2026-03-24T20:57:05Z
last_update: 2026-03-24T20:57:05Z
date_finished: null
---

# T-267: MCP resources — session list + session detail as read-only data

## Context

Extends T-264/T-265 MCP server with read-only resources (MCP protocol feature). Sessions are exposed as `termlink://` URIs for AI agent context.

## Acceptance Criteria

### Agent
- [x] `termlink://sessions` resource — JSON list of all active sessions
- [x] `termlink://sessions/{id}` resource — live session detail via RPC
- [x] Resource template for `{session_id}` URI pattern
- [x] Graceful fallback when session is unreachable (returns registration data)
- [x] Resources enabled in ServerCapabilities
- [x] 6 resource integration tests pass (20 total)
- [x] `cargo check -p termlink-mcp` compiles clean

## Verification

/Users/dimidev32/.cargo/bin/cargo check -p termlink-mcp
grep -q "enable_resources" crates/termlink-mcp/src/server.rs
grep -q "termlink://sessions" crates/termlink-mcp/src/server.rs
/Users/dimidev32/.cargo/bin/cargo test -p termlink-mcp --test mcp_integration -- --test-threads=1 2>&1 | grep -q "20 passed"

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

### 2026-03-24T20:57:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-267-mcp-resources--session-list--session-det.md
- **Context:** Initial task creation
