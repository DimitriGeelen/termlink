---
id: T-961
name: "Completion gate verifies episodic output exists after generation"
description: >
  Fix A/C from T-292: After generate-episodic runs during task completion, verify the .yaml file actually exists. If not, block completion. Prevents silent episodic generation failures.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T08:46:42Z
last_update: 2026-04-12T08:46:42Z
date_finished: null
---

# T-961: Completion gate verifies episodic output exists after generation

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `update-task.sh` checks for `.context/episodic/T-XXX.yaml` after calling `generate-episodic`
- [x] If episodic file missing after generation, warning with actionable error (doesn't block — warns loudly)
- [x] Existing tasks without episodic failures still complete (backward compatible)
- [x] Pickup sent to framework agent (this is a framework-side fix)

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

### 2026-04-12T08:46:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-961-completion-gate-verifies-episodic-output.md
- **Context:** Initial task creation
