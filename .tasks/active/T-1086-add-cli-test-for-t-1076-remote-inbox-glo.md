---
id: T-1086
name: "Add CLI test for T-1076 remote inbox global args fix"
description: >
  Add CLI test for T-1076 remote inbox global args fix

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T18:48:40Z
last_update: 2026-04-16T18:48:40Z
date_finished: null
---

# T-1086: Add CLI test for T-1076 remote inbox global args fix

## Context

T-1076 fixed remote inbox CLI (global args + optional subcommand). T-1077 fixed kv. Add regression tests to prevent re-break.

## Acceptance Criteria

### Agent
- [x] Test: `remote inbox <hub>` defaults to status (no subcommand required)
- [x] Test: `remote inbox <hub> status --timeout 5` parses options after subcommand
- [x] Test: `kv <session>` defaults to list (no subcommand required)
- [x] Test: `kv <session> list --json` parses options after subcommand
- [x] `cargo test` passes (4 new tests)

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

### 2026-04-16T18:48:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1086-add-cli-test-for-t-1076-remote-inbox-glo.md
- **Context:** Initial task creation
