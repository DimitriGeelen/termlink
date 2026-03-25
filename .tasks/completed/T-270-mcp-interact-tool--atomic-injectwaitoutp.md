---
id: T-270
name: "MCP interact tool — atomic inject+wait+output for AI agents"
description: >
  MCP interact tool — atomic inject+wait+output for AI agents

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T09:58:25Z
last_update: 2026-03-25T10:03:50Z
date_finished: 2026-03-25T10:03:50Z
---

# T-270: MCP interact tool — atomic inject+wait+output for AI agents

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `termlink_interact` MCP tool added with inject+poll+output logic
- [x] ANSI stripping enabled by default (AI agents want clean text)
- [x] Integration tests for interact tool (non-PTY error + nonexistent session)
- [x] All 453 workspace tests pass, zero warnings

## Verification

grep -q "termlink_interact" crates/termlink-mcp/src/tools.rs
grep -q "test_interact" crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-03-25T09:58:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-270-mcp-interact-tool--atomic-injectwaitoutp.md
- **Context:** Initial task creation

### 2026-03-25T10:03:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
