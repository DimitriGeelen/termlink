---
id: T-1084
name: "Register new scripts in component fabric"
description: >
  Register new scripts in component fabric

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T18:33:02Z
last_update: 2026-04-16T18:33:02Z
date_finished: null
---

# T-1084: Register new scripts in component fabric

## Context

scripts/watchdog.sh and scripts/learnings-exchange.sh were created in recent sessions but not registered in the component fabric.

## Acceptance Criteria

### Agent
- [x] scripts/watchdog.sh registered with enriched card
- [x] scripts/learnings-exchange.sh registered with enriched card
- [x] `fw fabric drift` reports 0 unregistered

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

### 2026-04-16T18:33:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1084-register-new-scripts-in-component-fabric.md
- **Context:** Initial task creation
