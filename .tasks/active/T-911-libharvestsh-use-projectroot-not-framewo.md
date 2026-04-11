---
id: T-911
name: "lib/harvest.sh use PROJECT_ROOT not FRAMEWORK_ROOT for learnings"
description: >
  Follow-up from T-909. lib/harvest.sh:74-75,363 writes harvested learnings to $FRAMEWORK_ROOT/.context/ which is currently accidentally-correct (writes to live framework via symlink) but will be wrong after any project vendors its framework. Post-T-909, fw harvest from /opt/termlink writes to the static vendored copy instead of the live framework repo. Fix: use $PROJECT_ROOT for per-project learning capture; optionally support a --upstream flag for pushing back to the framework.

status: captured
workflow_type: build
owner: human
horizon: now
tags: [infrastructure, harvest, path-resolution]
components: []
related_tasks: []
created: 2026-04-11T12:28:37Z
last_update: 2026-04-11T12:28:37Z
date_finished: null
---

# T-911: lib/harvest.sh use PROJECT_ROOT not FRAMEWORK_ROOT for learnings

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

### 2026-04-11T12:28:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-911-libharvestsh-use-projectroot-not-framewo.md
- **Context:** Initial task creation
