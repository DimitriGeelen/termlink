---
id: T-1240
name: "T-1229f migrate termlink_remote_inbox_status MCP to status_with_fallback_with_client"
description: >
  T-1229f migrate termlink_remote_inbox_status MCP to status_with_fallback_with_client

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-session/src/inbox_channel.rs]
related_tasks: []
created: 2026-04-25T10:37:27Z
last_update: 2026-04-25T10:47:07Z
date_finished: 2026-04-25T10:47:07Z
---

# T-1240: T-1229f migrate termlink_remote_inbox_status MCP to status_with_fallback_with_client

## Context

T-1229f per inception (`docs/reports/T-1229-inception.md`): migrate
`termlink_remote_inbox_status` MCP tool (`crates/termlink-mcp/src/tools.rs:4685`)
from legacy `inbox.status` RPC to `status_with_fallback_with_client` helper
(T-1235). Sibling of T-1239 (CLI remote).

## Acceptance Criteria

### Agent
- [x] MCP tool calls `status_with_fallback_with_client(&mut rpc_client, &p.hub, cache, &mut ctx)`
- [x] JSON `{ok, hub, result}` envelope preserved (typed InboxStatus serializes to same shape)
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

### 2026-04-25T10:37:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1240-t-1229f-migrate-termlinkremoteinboxstatu.md
- **Context:** Initial task creation

### 2026-04-25T10:47:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
