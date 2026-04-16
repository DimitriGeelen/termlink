---
id: T-1078
name: "Re-apply T-1066 review-queue patch — clobbered by framework upgrade"
description: >
  Re-apply T-1066 review-queue patch — clobbered by framework upgrade

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T05:28:59Z
last_update: 2026-04-16T05:31:42Z
date_finished: 2026-04-16T05:31:42Z
---

# T-1078: Re-apply T-1066 review-queue patch — clobbered by framework upgrade

## Context

Framework upgrade clobbered the T-1066 `fw task review-queue` patch. Re-apply from `docs/patches/T-1066-fw-task-review-queue.md`.

## Acceptance Criteria

### Agent
- [x] `fw task review-queue --count` returns an integer
- [x] `fw task review-queue` produces formatted output
- [x] `fw task help` mentions review-queue
- [x] T-1068 handover partial-complete section re-applied with tags + age sort

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

### 2026-04-16T05:28:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1078-re-apply-t-1066-review-queue-patch--clob.md
- **Context:** Initial task creation

### 2026-04-16T05:31:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
