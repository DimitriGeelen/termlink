---
id: T-910
name: "bin/watchtower.sh use PROJECT_ROOT not FRAMEWORK_ROOT for PID/log"
description: >
  Follow-up from T-909. bin/watchtower.sh:16-22 uses $FRAMEWORK_ROOT for the PID file and log paths, causing cross-project collisions when multiple projects share a framework (the symlink case in T-909 proved this: running watchtower.sh stop from one project could kill another project's instance). Fix: use $PROJECT_ROOT (or PROJECT_ROOT env override) for PID/log; fall back to FRAMEWORK_ROOT only as last resort. Add regression test.

status: captured
workflow_type: build
owner: human
horizon: now
tags: [infrastructure, watchtower, path-resolution]
components: []
related_tasks: []
created: 2026-04-11T12:28:29Z
last_update: 2026-04-11T12:28:29Z
date_finished: null
---

# T-910: bin/watchtower.sh use PROJECT_ROOT not FRAMEWORK_ROOT for PID/log

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

### 2026-04-11T12:28:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-910-binwatchtowersh-use-projectroot-not-fram.md
- **Context:** Initial task creation
