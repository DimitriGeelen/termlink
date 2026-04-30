---
id: T-1428
name: "Foundation soak audit — T-1426 + T-1427 ship status (2-week check)"
description: >
  Scheduled audit fire date: 2026-05-14. Check whether T-1426 (deprecation print on legacy primitives) and T-1427 (termlink whoami + identity binding) have shipped, and gather T-1166 cut-readiness signal from any deprecation telemetry the picks may have produced. This is a foundation-soak sentinel — created at the same time as T-1425 inception RFC + T-1426/T-1427 captures so the system has a structural reminder to re-check that the pre-cut foundation actually got built.

status: captured
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T21:21:07Z
last_update: 2026-04-30T21:21:07Z
date_finished: null
---

# T-1428: Foundation soak audit — T-1426 + T-1427 ship status (2-week check)

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

### 2026-04-30T21:21:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1428-foundation-soak-audit--t-1426--t-1427-sh.md
- **Context:** Initial task creation
