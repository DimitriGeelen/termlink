---
id: T-1244
name: "T-1230f migrate cmd_remote_inbox_inner Clear arm to clear_with_fallback_with_client"
description: >
  T-1230f migrate cmd_remote_inbox_inner Clear arm to clear_with_fallback_with_client

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T10:41:56Z
last_update: 2026-04-25T10:41:56Z
date_finished: null
---

# T-1244: T-1230f migrate cmd_remote_inbox_inner Clear arm to clear_with_fallback_with_client

## Context

T-1230f per inception: migrate `cmd_remote_inbox_inner` Clear arm
(`crates/termlink-cli/src/commands/remote.rs:1304`) to
`clear_with_fallback_with_client` (T-1236). Sibling of T-1242 (CLI local).

## Acceptance Criteria

### Agent
- [x] Clear arm calls `clear_with_fallback_with_client(&mut rpc_client, conn.hub, scope, cache, &mut ctx)`
- [x] Renderer reads typed `InboxClearResult`
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

### 2026-04-25T10:41:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1244-t-1230f-migrate-cmdremoteinboxinner-clea.md
- **Context:** Initial task creation
