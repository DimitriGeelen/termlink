---
id: T-1237
name: "T-1229c migrate cmd_inbox_status to status_with_fallback"
description: >
  T-1229c migrate cmd_inbox_status to status_with_fallback

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T10:33:32Z
last_update: 2026-04-25T10:35:06Z
date_finished: null
---

# T-1237: T-1229c migrate cmd_inbox_status to status_with_fallback

## Context

T-1229c per inception (`docs/reports/T-1229-inception.md`): migrate
`cmd_inbox_status` (`crates/termlink-cli/src/commands/infrastructure.rs:766`)
from legacy `inbox.status` RPC to `status_with_fallback` helper shipped in
T-1235. Follows the T-1226 pattern that migrated `cmd_inbox_list` to
`list_with_fallback`.

## Acceptance Criteria

### Agent
- [x] `cmd_inbox_status` calls `status_with_fallback(&addr, cache, &mut ctx)` instead of bare `inbox.status` RPC
- [x] Renderer reads `InboxStatus` struct fields directly (no JSON Value indexing)
- [x] JSON output mode preserves shape (serde Serialize on InboxStatus)
- [x] cargo build -p termlink-cli clean

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

### 2026-04-25T10:33:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1237-t-1229c-migrate-cmdinboxstatus-to-status.md
- **Context:** Initial task creation
