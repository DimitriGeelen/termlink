---
id: T-1245
name: "T-1230g migrate termlink_remote_inbox_clear MCP to clear_with_fallback_with_client"
description: >
  T-1230g migrate termlink_remote_inbox_clear MCP to clear_with_fallback_with_client

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-25T10:43:00Z
last_update: 2026-04-25T10:47:14Z
date_finished: 2026-04-25T10:47:14Z
---

# T-1245: T-1230g migrate termlink_remote_inbox_clear MCP to clear_with_fallback_with_client

## Context

T-1230g per inception: migrate `termlink_remote_inbox_clear` MCP tool
(`crates/termlink-mcp/src/tools.rs:4755`) to
`clear_with_fallback_with_client` (T-1236). Final inbox.clear migration site.

## Acceptance Criteria

### Agent
- [x] MCP tool calls `clear_with_fallback_with_client(&mut rpc_client, &p.hub, scope, cache, &mut ctx)`
- [x] JSON `{ok, hub, result}` envelope preserved
- [x] cargo build -p termlink-mcp clean

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification
cargo build -p termlink-mcp 2>&1 | tail -3 | grep -q "Finished"

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

### 2026-04-25T10:43:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1245-t-1230g-migrate-termlinkremoteinboxclear.md
- **Context:** Initial task creation

### 2026-04-25T10:47:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
