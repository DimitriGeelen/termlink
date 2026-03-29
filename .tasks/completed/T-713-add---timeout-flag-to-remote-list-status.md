---
id: T-713
name: "Add --timeout flag to remote list, status, and push commands"
description: >
  Add --timeout flag to remote list, status, and push commands

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/push.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-29T10:31:03Z
last_update: 2026-03-29T10:34:09Z
date_finished: 2026-03-29T10:34:09Z
---

# T-713: Add --timeout flag to remote list, status, and push commands

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `--timeout` flag added to RemoteAction::List (default: 10s), Status (default: 10s), Push (default: 30s) in cli.rs
- [x] Flags wired through main.rs
- [x] Each command wraps its operation in tokio timeout with JSON error support
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
termlink remote list --help 2>&1 | grep -q '\-\-timeout'
termlink remote status --help 2>&1 | grep -q '\-\-timeout'
termlink remote push --help 2>&1 | grep -q '\-\-timeout'

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

### 2026-03-29T10:31:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-713-add---timeout-flag-to-remote-list-status.md
- **Context:** Initial task creation

### 2026-03-29T10:34:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
