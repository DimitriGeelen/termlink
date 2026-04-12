---
id: T-963
name: "Session-start check for open concerns and recurring failure patterns"
description: >
  Fix B from T-283: context init (session start) should read concerns.yaml for open gaps and warn. Prevents cross-session failure blindness by surfacing known issues at start of every session.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T09:09:42Z
last_update: 2026-04-12T09:12:13Z
date_finished: 2026-04-12T09:12:13Z
---

# T-963: Session-start check for open concerns and recurring failure patterns

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `context.sh init` reads concerns.yaml and warns about open (non-closed) gaps
- [x] Warning includes gap ID, summary, and age
- [x] No warning when all concerns are closed (backward compatible)
- [x] Pickup sent to framework agent

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

### 2026-04-12T09:09:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-963-session-start-check-for-open-concerns-an.md
- **Context:** Initial task creation

### 2026-04-12T09:12:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
