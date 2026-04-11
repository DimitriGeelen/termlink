---
id: T-913
name: "fw doctor check — warn if .agentic-framework is a symlink"
description: >
  Follow-up from T-909. fw doctor should detect when a consumer project has .agentic-framework as a symlink (vs a real vendored directory) and emit a WARN with pointer to fw vendor. This would have caught G-001 immediately instead of letting it sit undetected for weeks. Also consider: fw upgrade pre-flight should refuse to upgrade a consumer project that still has a symlink (suggest vendoring first).

status: captured
workflow_type: build
owner: human
horizon: now
tags: [infrastructure, doctor, symlink]
components: []
related_tasks: []
created: 2026-04-11T12:28:45Z
last_update: 2026-04-11T12:28:45Z
date_finished: null
---

# T-913: fw doctor check — warn if .agentic-framework is a symlink

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

### 2026-04-11T12:28:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-913-fw-doctor-check--warn-if-agentic-framewo.md
- **Context:** Initial task creation
