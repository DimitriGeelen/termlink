---
id: T-943
name: "Fix review marker friction — Watchtower decide creates marker automatically"
description: >
  Fix review marker friction — Watchtower decide creates marker automatically

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T07:56:03Z
last_update: 2026-04-12T07:57:37Z
date_finished: 2026-04-12T07:57:37Z
---

# T-943: Fix review marker friction — Watchtower decide creates marker automatically

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `inception.py` `record_decision()` creates review marker before calling `fw inception decide`
- [x] Watchtower GO/NO-GO submission works without prior `fw task review`
- [x] Pickup created for framework agent (P-010 delivered to framework inbox)

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

### 2026-04-12T07:56:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-943-fix-review-marker-friction--watchtower-d.md
- **Context:** Initial task creation

### 2026-04-12T07:57:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
