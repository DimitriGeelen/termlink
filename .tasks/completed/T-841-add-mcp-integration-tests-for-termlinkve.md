---
id: T-841
name: "Add MCP integration tests for termlink_version and termlink_token_create tools"
description: >
  Add MCP integration tests for termlink_version and termlink_token_create tools

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T00:16:59Z
last_update: 2026-04-04T00:22:17Z
date_finished: 2026-04-04T00:22:17Z
---

# T-841: Add MCP integration tests for termlink_version and termlink_token_create tools

## Context

T-840 added termlink_version and termlink_token_create MCP tools. Need integration tests verifying they work through the full MCP tool_router pipeline.

## Acceptance Criteria

### Agent
- [x] Integration test for termlink_version returns valid JSON with version/commit/target/mcp_tools fields
- [x] Integration test for termlink_token_create with token-secret-enabled session returns a valid token
- [x] Integration test for termlink_token_create on non-token session returns descriptive error
- [x] test_list_tools updated to include new tool names (38 tools)
- [x] All tests pass (755 total), zero clippy warnings

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo test -p termlink-mcp 2>&1 | tail -5
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c 'warning:')" = "0"

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

### 2026-04-04T00:16:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-841-add-mcp-integration-tests-for-termlinkve.md
- **Context:** Initial task creation

### 2026-04-04T00:22:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
