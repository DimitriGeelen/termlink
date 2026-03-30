---
id: T-812
name: "Upgrade cmd_request to use event.subscribe for lower latency reply waiting"
description: >
  Upgrade cmd_request to use event.subscribe for lower latency reply waiting

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-30T18:11:49Z
last_update: 2026-03-30T18:13:36Z
date_finished: 2026-03-30T18:13:36Z
---

# T-812: Upgrade cmd_request to use event.subscribe for lower latency reply waiting

## Context

`cmd_request` waits for reply events using `event.poll` in a sleep loop. Upgrading to `event.subscribe` gives near-instant reply detection. Also upgrade the initial cursor fetch to use the EventBus's `next_seq` from subscribe response.

## Acceptance Criteria

### Agent
- [x] Reply wait loop uses `event.subscribe` instead of `event.poll` + sleep
- [x] Initial cursor fetch uses `event.subscribe` with 1ms timeout for next_seq
- [x] request_id matching still works correctly
- [x] Timeout and error handling preserved
- [x] `cargo check -p termlink` passes

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

grep -q "event.subscribe" crates/termlink-cli/src/commands/execution.rs
cargo check -p termlink 2>&1 | grep -q "Finished"

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

### 2026-03-30T18:11:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-812-upgrade-cmdrequest-to-use-eventsubscribe.md
- **Context:** Initial task creation

### 2026-03-30T18:13:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
