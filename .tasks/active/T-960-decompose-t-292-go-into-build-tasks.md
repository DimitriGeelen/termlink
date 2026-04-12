---
id: T-960
name: "Decompose T-292 GO into build tasks"
description: >
  Decompose T-292 GO into build tasks

status: started-work
workflow_type: specification
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T08:45:52Z
last_update: 2026-04-12T08:45:52Z
date_finished: null
---

# T-960: Decompose T-292 GO into build tasks

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Build tasks created for Fix A (completion gate verifies episodic) — T-961
- [x] Build task created for Fix D (portable date handling for macOS) — T-962
- [x] Each task has real ACs (not placeholders)

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

### 2026-04-12T08:45:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-960-decompose-t-292-go-into-build-tasks.md
- **Context:** Initial task creation
