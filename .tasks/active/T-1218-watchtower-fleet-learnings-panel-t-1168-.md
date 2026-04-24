---
id: T-1218
name: "Watchtower fleet-learnings panel (T-1168 B3)"
description: >
  Watchtower panel that reads the cross-project learnings mirror written by T-1217's subscriber. Displays origin_project, learning_id, learning text, task, source, date, received_at per entry. Read-only UI over .context/project/received-learnings.yaml. Split from T-1217 B3 per sizing rule 'one task = one deliverable'. Open question: should this be a new /fleet-learnings page, or integrated into the existing fleet overview? Warrants inception before build.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-04-24T13:09:59Z
last_update: 2026-04-24T13:09:59Z
date_finished: null
---

# T-1218: Watchtower fleet-learnings panel (T-1168 B3)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

### 2026-04-24T13:09:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1218-watchtower-fleet-learnings-panel-t-1168-.md
- **Context:** Initial task creation
