---
id: T-1241
name: "T-1229g migrate fleet-doctor inbox check + offline-target visibility regression test"
description: >
  T-1229g migrate fleet-doctor inbox check + offline-target visibility regression test

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, crates/termlink-session/src/inbox_channel.rs]
related_tasks: []
created: 2026-04-25T10:38:33Z
last_update: 2026-04-25T10:47:08Z
date_finished: 2026-04-25T10:47:08Z
---

# T-1241: T-1229g migrate fleet-doctor inbox check + offline-target visibility regression test

## Context

T-1229g per inception (`docs/reports/T-1229-inception.md`): migrate the
fleet-doctor inbox check (`crates/termlink-cli/src/commands/remote.rs:2795`)
from legacy `inbox.status` to `status_with_fallback_with_client`.

**Inception requirement:** must include offline-target visibility regression
test. The point of T-1229 was that channel.list returns ALL `inbox:` topics
including ones for currently-offline targets (since topics are derived from
hub bus records, not live subscribers). The fleet-doctor invariant (G-013
upstream check) depends on counting transfers for offline targets too —
those are the most important ones to surface.

## Acceptance Criteria

### Agent
- [x] Fleet-doctor inbox check calls `status_with_fallback_with_client(&mut rpc_client, conn.hub, cache, &mut ctx)`
- [x] Reads typed `InboxStatus` (no JSON Value indexing)
- [x] Regression test in `inbox_channel.rs`: `aggregate_status_includes_offline_targets` — verifies a topic with `count=N` and no live subscriber still appears in InboxStatus output (the helper is transport-agnostic so this is a pure-fn test on a synthesized channel.list reply with multiple targets)
- [x] cargo build -p termlink clean
- [x] cargo test -p termlink-session --lib inbox_channel:: passes

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
cargo test -p termlink-session --lib inbox_channel:: 2>&1 | tail -3 | grep -q "test result: ok"

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

### 2026-04-25T10:38:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1241-t-1229g-migrate-fleet-doctor-inbox-check.md
- **Context:** Initial task creation

### 2026-04-25T10:47:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
