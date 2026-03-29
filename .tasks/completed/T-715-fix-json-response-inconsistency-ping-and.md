---
id: T-715
name: "Fix JSON response inconsistency: ping and signal use status:ok instead of ok:true"
description: >
  Fix JSON response inconsistency: ping and signal use status:ok instead of ok:true

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-29T10:42:13Z
last_update: 2026-03-29T10:43:54Z
date_finished: 2026-03-29T10:43:54Z
---

# T-715: Fix JSON response inconsistency: ping and signal use status:ok instead of ok:true

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] cmd_ping JSON success response uses `"ok": true` instead of `"status": "ok"`
- [x] cmd_signal JSON success response uses `"ok": true` instead of `"status": "ok"`
- [x] `cargo build --release` succeeds

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

cargo build --release 2>&1 | tail -1
! grep -q '"status": "ok"' crates/termlink-cli/src/commands/session.rs

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

### 2026-03-29T10:42:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-715-fix-json-response-inconsistency-ping-and.md
- **Context:** Initial task creation

### 2026-03-29T10:43:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
