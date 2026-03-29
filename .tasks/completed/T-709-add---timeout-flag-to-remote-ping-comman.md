---
id: T-709
name: "Add --timeout flag to remote ping command for configurable RPC timeout"
description: >
  Add --timeout flag to remote ping command for configurable RPC timeout

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-29T08:44:16Z
last_update: 2026-03-29T10:16:01Z
date_finished: 2026-03-29T10:16:01Z
---

# T-709: Add --timeout flag to remote ping command for configurable RPC timeout

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `--timeout` flag added to RemoteAction::Ping in cli.rs (default: 10s)
- [x] Flag wired through main.rs to cmd_remote_ping
- [x] cmd_remote_ping wraps the entire operation in tokio timeout
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
termlink remote ping --help 2>&1 | grep -q '\-\-timeout'

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

### 2026-03-29T08:44:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-709-add---timeout-flag-to-remote-ping-comman.md
- **Context:** Initial task creation

### 2026-03-29T08:45:15Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** Budget gate hit at 95% context — CLI changes in progress but not committed

### 2026-03-29T10:14:33Z — status-update [task-update-agent]
- **Change:** status: issues → started-work

### 2026-03-29T10:16:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
