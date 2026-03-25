---
id: T-275
name: "MCP doctor and clean tools for AI agent self-healing"
description: >
  MCP doctor and clean tools for AI agent self-healing

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T13:07:00Z
last_update: 2026-03-25T13:07:00Z
date_finished: null
---

# T-275: MCP doctor and clean tools for AI agent self-healing

## Context

Add `termlink_doctor` and `termlink_clean` MCP tools so AI agents can self-diagnose and remediate TermLink environment issues without CLI access. Complements T-273/T-274 CLI doctor.

## Acceptance Criteria

### Agent
- [x] `termlink_doctor` tool exists and returns structured JSON health report (checks, summary with pass/warn/fail counts)
- [x] `termlink_clean` tool exists and removes stale sessions + orphaned sockets, returns count of cleaned items
- [x] MCP integration tests for both tools pass (4 new tests)
- [x] All existing tests pass (cargo test --workspace — 463 pass, 0 fail)
- [x] `termlink_list_tools` test updated to expect 22+ tools

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | grep -q "0 failed"
grep -q "termlink_doctor" crates/termlink-mcp/src/tools.rs
grep -q "termlink_clean" crates/termlink-mcp/src/tools.rs
grep -q "test_doctor" crates/termlink-mcp/tests/mcp_integration.rs
grep -q "test_clean" crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-03-25T13:07:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-275-mcp-doctor-and-clean-tools-for-ai-agent-.md
- **Context:** Initial task creation
