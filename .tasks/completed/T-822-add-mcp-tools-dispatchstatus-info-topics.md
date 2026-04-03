---
id: T-822
name: "Add MCP tools: dispatch_status, info, topics — expand AI agent coverage to 30 tools"
description: >
  Add MCP tools: dispatch_status, info, topics — expand AI agent coverage to 30 tools

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-03T19:55:14Z
last_update: 2026-04-03T20:04:20Z
date_finished: 2026-04-03T20:04:20Z
---

# T-822: Add MCP tools: dispatch_status, info, topics — expand AI agent coverage to 30 tools

## Context

Add 3 new MCP tools to close the gap between CLI commands and MCP API surface. Currently 27 MCP tools; these bring it to 30. All three mirror existing CLI commands.

## Acceptance Criteria

### Agent
- [x] `termlink_dispatch_status` MCP tool — reads dispatch manifest, returns pending/merged/conflict/expired counts
- [x] `termlink_info` MCP tool — returns runtime info (version, sessions, hub status, paths)
- [x] `termlink_topics` MCP tool — lists event topics across all sessions (or a specific one)
- [x] All 3 tools registered in the MCP server tool list
- [x] Integration tests for each new tool (8 new tests: 2 dispatch_status, 2 info, 3 topics, 1 topics nonexistent)
- [x] `cargo test --workspace` passes (692 tests)
- [x] `cargo clippy --workspace` clean (0 warnings)
- [x] CHANGELOG updated with new tools

## Verification

cargo test -p termlink-mcp
cargo clippy --workspace
grep -q 'termlink_dispatch_status' crates/termlink-mcp/src/tools.rs
grep -q 'termlink_info' crates/termlink-mcp/src/tools.rs
grep -q 'termlink_topics' crates/termlink-mcp/src/tools.rs

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

### 2026-04-03T19:55:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-822-add-mcp-tools-dispatchstatus-info-topics.md
- **Context:** Initial task creation

### 2026-04-03T20:04:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
