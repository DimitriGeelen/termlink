---
id: T-627
name: "Add --no-header flag to event topics command"
description: >
  Add --no-header flag to event topics command

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/events.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-28T17:54:40Z
last_update: 2026-03-29T10:19:08Z
date_finished: 2026-03-29T10:19:08Z
---

# T-627: Add --no-header flag to event topics command

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `--no-header` flag added to EventCommand::Topics in cli.rs
- [x] Flag wired through main.rs to cmd_topics
- [x] cmd_topics suppresses session name headers and summary footer when --no-header is set
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
termlink event topics --help 2>&1 | grep -q '\-\-no-header'

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

### 2026-03-28T17:54:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-627-add---no-header-flag-to-event-topics-com.md
- **Context:** Initial task creation

### 2026-03-28T17:55:09Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** Topics command has no header/footer to suppress — wrong improvement target

### 2026-03-29T10:16:51Z — status-update [task-update-agent]
- **Change:** status: issues → started-work

### 2026-03-29T10:19:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
