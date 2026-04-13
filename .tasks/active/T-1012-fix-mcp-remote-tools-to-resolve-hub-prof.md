---
id: T-1012
name: "Fix MCP remote tools to resolve hub profile names"
description: >
  Fix MCP remote tools to resolve hub profile names

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T10:04:14Z
last_update: 2026-04-13T10:04:14Z
date_finished: null
---

# T-1012: Fix MCP remote tools to resolve hub profile names

## Context

`connect_remote_hub_mcp` only accepts `host:port` format — profile names like "ring20-management" fail with "Invalid hub address". All MCP remote tool param docs say "host:port or profile name" but profile resolution is missing. Fix by adding profile lookup before address parsing.

## Acceptance Criteria

### Agent
- [x] connect_remote_hub_mcp resolves profile names from ~/.termlink/hubs.toml
- [x] Profile secret_file/secret are used when MCP params don't provide them
- [x] Direct host:port still works (backward compatible)
- [x] cargo clippy --workspace passes (0 warnings)
- [x] cargo test --workspace passes (1003 tests)

### Human
- [ ] [RUBBER-STAMP] Verify via MCP that profile resolution works
  **Steps:**
  1. Use termlink_remote_ping with hub="ring20-management" (no secret params)
  **Expected:** Returns pong from .109
  **If not:** Check config.rs imports in tools.rs

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

### 2026-04-13T10:04:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1012-fix-mcp-remote-tools-to-resolve-hub-prof.md
- **Context:** Initial task creation
