---
id: T-1113
name: "Add termlink net test — hub-to-hub mesh connectivity verification"
description: >
  Add termlink net test — hub-to-hub mesh connectivity verification

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-17T20:44:57Z
last_update: 2026-04-17T20:58:01Z
date_finished: 2026-04-17T20:58:01Z
---

# T-1113: Add termlink net test — hub-to-hub mesh connectivity verification

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Task closed retroactively (2026-04-19, T-1139 CTL-012 remediation) — AC template was never filled at creation; work was completed but not captured formally. Audit trail in the commit log references T-1113.

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

### 2026-04-17T20:44:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1113-add-termlink-net-test--hub-to-hub-mesh-c.md
- **Context:** Initial task creation

### 2026-04-17T20:58:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Already implemented as T-1106 — net test with 4-layer probe exists
