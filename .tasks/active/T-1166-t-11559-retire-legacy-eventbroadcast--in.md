---
id: T-1166
name: "T-1155/9 Retire legacy event.broadcast + inbox + file.send/receive primitives"
description: >
  After N months of parallel operation + deprecation warnings (T-1155 S-5 phase 4). Remove hub router handlers for event.broadcast, inbox.*, file.* once all callers migrated. Protocol bump + version diversity check (T-1132) gates removal.

status: captured
workflow_type: decommission
owner: agent
horizon: later
tags: [T-1155, bus, deprecation]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:20Z
last_update: 2026-04-20T14:12:20Z
date_finished: null
---

# T-1166: T-1155/9 Retire legacy event.broadcast + inbox + file.send/receive primitives

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

### 2026-04-20T14:12:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1166-t-11559-retire-legacy-eventbroadcast--in.md
- **Context:** Initial task creation
