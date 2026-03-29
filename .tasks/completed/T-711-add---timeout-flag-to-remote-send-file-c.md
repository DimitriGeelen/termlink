---
id: T-711
name: "Add --timeout flag to remote send-file command (default 60s)"
description: >
  Add --timeout flag to remote send-file command (default 60s)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-29T10:24:49Z
last_update: 2026-03-29T10:26:39Z
date_finished: 2026-03-29T10:26:39Z
---

# T-711: Add --timeout flag to remote send-file command (default 60s)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `--timeout` flag added to RemoteAction::SendFile in cli.rs (default: 60s)
- [x] Flag wired through main.rs to cmd_remote_send_file
- [x] cmd_remote_send_file wraps the entire operation in tokio timeout
- [x] Timeout produces JSON error when --json is set
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
termlink remote send-file --help 2>&1 | grep -q '\-\-timeout'

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

### 2026-03-29T10:24:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-711-add---timeout-flag-to-remote-send-file-c.md
- **Context:** Initial task creation

### 2026-03-29T10:26:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
