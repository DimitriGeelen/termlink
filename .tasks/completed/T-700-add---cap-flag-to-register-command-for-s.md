---
id: T-700
name: "Add --cap flag to register command for setting initial capabilities"
description: >
  Register and register --self can set roles and tags but not capabilities. Add --cap for setting initial capabilities at registration time, matching the pattern of --roles and --tags.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/session.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-29T08:16:41Z
last_update: 2026-03-29T08:18:52Z
date_finished: 2026-03-29T08:18:52Z
---

# T-700: Add --cap flag to register command for setting initial capabilities

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `--cap` flag added to Register in cli.rs with value_delimiter = ','
- [x] Capabilities passed through main.rs to cmd_register and cmd_register_self
- [x] cmd_register merges user caps with shell-mode caps (data_plane, stream)
- [x] cmd_register_self passes caps to SessionConfig
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
termlink register --help 2>&1 | grep -q '\-\-cap'

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

### 2026-03-29T08:16:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-700-add---cap-flag-to-register-command-for-s.md
- **Context:** Initial task creation

### 2026-03-29T08:18:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
