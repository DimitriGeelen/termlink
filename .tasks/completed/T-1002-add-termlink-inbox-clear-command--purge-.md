---
id: T-1002
name: "Add termlink inbox clear command — purge pending transfers"
description: >
  Add termlink inbox clear command — purge pending transfers

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-cli/src/main.rs, crates/termlink-hub/src/inbox.rs, crates/termlink-hub/src/router.rs]
related_tasks: []
created: 2026-04-13T08:33:50Z
last_update: 2026-04-13T08:37:00Z
date_finished: 2026-04-13T08:37:00Z
---

# T-1002: Add termlink inbox clear command — purge pending transfers

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `termlink inbox clear <target>` removes pending transfers for a target
- [x] `termlink inbox clear --all` removes all pending transfers

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-13T08:33:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1002-add-termlink-inbox-clear-command--purge-.md
- **Context:** Initial task creation

### 2026-04-13T08:37:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
