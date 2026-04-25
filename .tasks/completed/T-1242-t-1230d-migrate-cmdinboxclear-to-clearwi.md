---
id: T-1242
name: "T-1230d migrate cmd_inbox_clear to clear_with_fallback"
description: >
  T-1230d migrate cmd_inbox_clear to clear_with_fallback

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-25T10:40:05Z
last_update: 2026-04-25T10:47:10Z
date_finished: 2026-04-25T10:47:10Z
---

# T-1242: T-1230d migrate cmd_inbox_clear to clear_with_fallback

## Context

T-1230d per inception (`docs/reports/T-1230-inception.md`): migrate
`cmd_inbox_clear` (`crates/termlink-cli/src/commands/infrastructure.rs:802`)
from legacy `inbox.clear` RPC to `clear_with_fallback` (T-1236).

## Acceptance Criteria

### Agent
- [x] `cmd_inbox_clear` calls `clear_with_fallback(&addr, scope, cache, &mut ctx)` with `ClearScope::Target(name)` or `ClearScope::All`
- [x] Renderer reads typed `InboxClearResult` (`.cleared`, `.target`)
- [x] JSON output preserves shape via serde Serialize
- [x] cargo build -p termlink clean

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
cargo build -p termlink 2>&1 | tail -3 | grep -q "Finished"

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

### 2026-04-25T10:40:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1242-t-1230d-migrate-cmdinboxclear-to-clearwi.md
- **Context:** Initial task creation

### 2026-04-25T10:47:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
