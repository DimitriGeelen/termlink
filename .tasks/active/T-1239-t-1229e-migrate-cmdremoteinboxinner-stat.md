---
id: T-1239
name: "T-1229e migrate cmd_remote_inbox_inner Status arm to status_with_fallback_with_client"
description: >
  T-1229e migrate cmd_remote_inbox_inner Status arm to status_with_fallback_with_client

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T10:36:21Z
last_update: 2026-04-25T10:37:20Z
date_finished: null
---

# T-1239: T-1229e migrate cmd_remote_inbox_inner Status arm to status_with_fallback_with_client

## Context

T-1229e per inception (`docs/reports/T-1229-inception.md`): migrate
`cmd_remote_inbox_inner` Status arm (`crates/termlink-cli/src/commands/remote.rs:1253`)
from legacy `inbox.status` RPC to `status_with_fallback_with_client` helper
(T-1235). Sibling List arm (line 1286) was migrated by T-1227 — same pattern.

Legacy renderer reads `t["transfer_count"]` (likely a typo — InboxStatus
struct exposes `pending`). Switch to typed struct fields.

## Acceptance Criteria

### Agent
- [x] Status arm calls `status_with_fallback_with_client(&mut rpc_client, conn.hub, cache, &mut ctx)`
- [x] Renderer reads typed `InboxStatus` struct fields (`status.total_transfers`, `t.target`, `t.pending`)
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

### 2026-04-25T10:36:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1239-t-1229e-migrate-cmdremoteinboxinner-stat.md
- **Context:** Initial task creation
