---
id: T-271
name: "MCP prompts — debug_session, orchestrate, session_overview"
description: >
  MCP prompts — debug_session, orchestrate, session_overview

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T11:46:25Z
last_update: 2026-03-25T11:51:52Z
date_finished: 2026-03-25T11:51:52Z
---

# T-271: MCP prompts — debug_session, orchestrate, session_overview

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] 3 MCP prompts: debug_session, session_overview, orchestrate
- [x] Prompts capability enabled in ServerCapabilities
- [x] 6 integration tests for prompts (list, get each, unknown error)
- [x] All 459 workspace tests pass, zero warnings

## Verification

grep -q "enable_prompts" crates/termlink-mcp/src/server.rs
grep -q "test_list_prompts" crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-03-25T11:46:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-271-mcp-prompts--debugsession-orchestrate-se.md
- **Context:** Initial task creation

### 2026-03-25T11:51:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
