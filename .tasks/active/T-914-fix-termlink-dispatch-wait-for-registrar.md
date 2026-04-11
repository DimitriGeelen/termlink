---
id: T-914
name: "Fix termlink dispatch wait-for-registrar hang on fast-failing user_cmd (G-002)"
description: >
  G-002 from concerns.yaml. crates/termlink-cli/src/commands/dispatch.rs:293 builds a sh-c template: 'termlink register ... &; TL_PID=$!; sleep 1; <user_cmd>; wait $TL_PID'. When user_cmd fast-fails (e.g., claude -p --dangerously-skip-permissions refuses root, exit 1 in <100ms), sh falls through to wait $TL_PID which blocks on the still-alive registrar. Dispatch sees 'ready' but times out only after --timeout seconds. Discovered 2026-04-11 during T-909 risk-eval: 3 workers appeared ready in termlink list but pstree showed no bash/claude grandchild — silent failure. Fix: capture user_cmd exit explicitly, kill registrar on non-zero, exit with user_cmd's rc. Regression test: 'termlink dispatch ... -- bash -c "exit 42"' must exit 42 within ~3s.

status: captured
workflow_type: build
owner: human
horizon: now
tags: [termlink, dispatch, bug, observability]
components: []
related_tasks: []
created: 2026-04-11T12:30:39Z
last_update: 2026-04-11T12:30:39Z
date_finished: null
---

# T-914: Fix termlink dispatch wait-for-registrar hang on fast-failing user_cmd (G-002)

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

### 2026-04-11T12:30:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-914-fix-termlink-dispatch-wait-for-registrar.md
- **Context:** Initial task creation
