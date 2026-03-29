---
id: T-703
name: "Add --cap flag to run command for setting capabilities on ephemeral sessions"
description: >
  Add --cap flag to run command for setting capabilities on ephemeral sessions

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/execution.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-29T08:25:07Z
last_update: 2026-03-29T08:26:41Z
date_finished: 2026-03-29T08:26:41Z
---

# T-703: Add --cap flag to run command for setting capabilities on ephemeral sessions

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `--cap` flag added to Run in cli.rs with value_delimiter = ','
- [x] Capabilities threaded through main.rs to cmd_run
- [x] cmd_run passes caps to SessionConfig
- [x] `cargo build --release` succeeds

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

cargo build --release 2>&1 | tail -1
termlink run --help 2>&1 | grep -q '\-\-cap'

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

### 2026-03-29T08:25:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-703-add---cap-flag-to-run-command-for-settin.md
- **Context:** Initial task creation

### 2026-03-29T08:26:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
