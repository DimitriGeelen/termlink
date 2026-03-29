---
id: T-714
name: "Add --count flag to clean command for script-friendly stale session count"
description: >
  Add --count flag to clean command for script-friendly stale session count

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/session.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-29T10:38:51Z
last_update: 2026-03-29T10:40:34Z
date_finished: 2026-03-29T10:40:34Z
---

# T-714: Add --count flag to clean command for script-friendly stale session count

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `--count` flag added to Command::Clean in cli.rs
- [x] Flag wired through main.rs to cmd_clean
- [x] cmd_clean outputs only the stale count number when --count is set
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
termlink clean --help 2>&1 | grep -q '\-\-count'

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

### 2026-03-29T10:38:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-714-add---count-flag-to-clean-command-for-sc.md
- **Context:** Initial task creation

### 2026-03-29T10:40:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
