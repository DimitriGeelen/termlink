---
id: T-277
name: "MCP request tool — request-reply pattern for inter-session coordination"
description: >
  MCP request tool — request-reply pattern for inter-session coordination

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T13:52:02Z
last_update: 2026-03-25T13:55:16Z
date_finished: 2026-03-25T13:55:16Z
---

# T-277: MCP request tool — request-reply pattern for inter-session coordination

## Context

Add `termlink_request` MCP tool — emit an event to a session and wait for a reply on a specified topic. Enables request-reply coordination between AI agent sessions. Mirrors CLI `termlink request` command.

## Acceptance Criteria

### Agent
- [x] `termlink_request` tool emits event with request_id and polls for matching reply
- [x] MCP integration tests pass (3 new: nonexistent, with-reply, timeout)
- [x] All tests pass (cargo test --workspace — 470 pass, 0 fail)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace
grep -q "termlink_request" crates/termlink-mcp/src/tools.rs
grep -q "test_request" crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-03-25T13:52:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-277-mcp-request-tool--request-reply-pattern-.md
- **Context:** Initial task creation

### 2026-03-25T13:55:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
