---
id: T-699
name: "Add --names and --ids flags to discover command for parity with list"
description: >
  The list command has --names and --ids flags for piping session display names or IDs one per line. The discover command lacks these, forcing users to parse table output. Add both flags for scripting parity.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/metadata.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-29T08:14:11Z
last_update: 2026-03-29T08:16:34Z
date_finished: 2026-03-29T08:16:34Z
---

# T-699: Add --names and --ids flags to discover command for parity with list

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `--names` flag added to Discover in cli.rs, prints display names one per line
- [x] `--ids` flag added to Discover in cli.rs, prints session IDs one per line
- [x] Both flags wired through main.rs dispatch to cmd_discover
- [x] cmd_discover handles names/ids output modes (same pattern as list)
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
termlink discover --help 2>&1 | grep -q '\-\-names'
termlink discover --help 2>&1 | grep -q '\-\-ids'

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

### 2026-03-29T08:14:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-699-add---names-and---ids-flags-to-discover-.md
- **Context:** Initial task creation

### 2026-03-29T08:16:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
