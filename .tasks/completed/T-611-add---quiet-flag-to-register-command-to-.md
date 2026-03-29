---
id: T-611
name: "Add --quiet flag to register command to suppress startup output"
description: >
  Add --quiet flag to register command to suppress startup output

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/session.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-28T17:07:07Z
last_update: 2026-03-29T11:31:16Z
date_finished: 2026-03-29T11:31:16Z
---

# T-611: Add --quiet flag to register command to suppress startup output

## Context

When `register` runs in background/scripts, the startup messages clutter logs. `--quiet` suppresses all non-error output (or `--json` already works, but `--quiet` is simpler for no-output use cases).

## Acceptance Criteria

### Agent
- [x] `--quiet` flag added to Register command in cli.rs
- [x] When --quiet, suppress all println! in cmd_register (but not errors)
- [x] main.rs passes quiet through
- [x] Project compiles with `cargo check`

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

cargo check 2>&1 | grep -q 'Finished'
grep -q 'quiet' crates/termlink-cli/src/cli.rs
grep -q 'quiet' crates/termlink-cli/src/main.rs
grep -q 'verbose' crates/termlink-cli/src/commands/session.rs

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

### 2026-03-28T17:07:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-611-add---quiet-flag-to-register-command-to-.md
- **Context:** Initial task creation

### 2026-03-28T17:07:39Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** Redundant with --json; register already has structured output that suppresses human messages

### 2026-03-29T11:28:48Z — status-update [task-update-agent]
- **Change:** status: issues → started-work

### 2026-03-29T11:31:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
