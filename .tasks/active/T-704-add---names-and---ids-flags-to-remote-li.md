---
id: T-704
name: "Add --names and --ids flags to remote list command for scripting parity"
description: >
  Add --names and --ids flags to remote list command for scripting parity

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T08:27:13Z
last_update: 2026-03-29T08:27:13Z
date_finished: null
---

# T-704: Add --names and --ids flags to remote list command for scripting parity

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `--names` and `--ids` flags added to RemoteAction::List in cli.rs
- [x] Flags wired through main.rs to cmd_remote_list
- [x] cmd_remote_list outputs names/IDs one per line when specified
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
termlink remote list --help 2>&1 | grep -q '\-\-names'
termlink remote list --help 2>&1 | grep -q '\-\-ids'

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

### 2026-03-29T08:27:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-704-add---names-and---ids-flags-to-remote-li.md
- **Context:** Initial task creation
