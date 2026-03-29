---
id: T-705
name: "Add --payload-only flag to remote events command for piping raw event payloads"
description: >
  Add --payload-only flag to remote events command for piping raw event payloads

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-29T08:29:57Z
last_update: 2026-03-29T08:32:06Z
date_finished: 2026-03-29T08:32:06Z
---

# T-705: Add --payload-only flag to remote events command for piping raw event payloads

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `--payload-only` flag added to RemoteAction::Events in cli.rs
- [x] Flag wired through main.rs to cmd_remote_events
- [x] When payload_only is true, outputs only event payloads (one JSON per line, skips null)
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
termlink remote events --help 2>&1 | grep -q 'payload.only'

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

### 2026-03-29T08:29:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-705-add---payload-only-flag-to-remote-events.md
- **Context:** Initial task creation

### 2026-03-29T08:32:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
