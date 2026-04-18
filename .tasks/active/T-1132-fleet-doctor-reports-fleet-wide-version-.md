---
id: T-1132
name: "fleet doctor reports fleet-wide version diversity (piggyback on query.capabilities) (from T-1071 GO)"
description: >
  From T-1071 inception GO. fleet doctor / fleet status should report fleet-wide version diversity, e.g. 'Versions in fleet: 0.9.815 (1 hub), 0.9.99 (1 hub), 0.9.844 (1 hub)'. Cheap — reuses the query.capabilities ping already in fleet doctor probe path. Lets operators see at a glance whether a fleet is homogenous or skewed before a Tier-B typed RPC fails.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [termlink, fleet-doctor, diagnostics, T-1071]
components: []
related_tasks: []
created: 2026-04-18T23:00:06Z
last_update: 2026-04-18T23:00:06Z
date_finished: null
---

# T-1132: fleet doctor reports fleet-wide version diversity (piggyback on query.capabilities) (from T-1071 GO)

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

### 2026-04-18T23:00:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1132-fleet-doctor-reports-fleet-wide-version-.md
- **Context:** Initial task creation
