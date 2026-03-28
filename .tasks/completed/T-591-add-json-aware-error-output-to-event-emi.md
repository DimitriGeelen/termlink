---
id: T-591
name: "Add JSON-aware error output to event emit, emit-to, broadcast, and poll commands"
description: >
  Add JSON-aware error output to event emit, emit-to, broadcast, and poll commands

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/events.rs]
related_tasks: []
created: 2026-03-28T16:19:07Z
last_update: 2026-03-28T16:20:41Z
date_finished: 2026-03-28T16:20:41Z
---

# T-591: Add JSON-aware error output to event emit, emit-to, broadcast, and poll commands

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] cmd_emit error paths emit JSON when --json is passed
- [x] cmd_emit_to error paths emit JSON when --json is passed
- [x] cmd_broadcast error paths emit JSON when --json is passed
- [x] cmd_events (poll) error paths emit JSON when --json is passed
- [x] cargo build succeeds

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

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-03-28T16:19:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-591-add-json-aware-error-output-to-event-emi.md
- **Context:** Initial task creation

### 2026-03-28T16:20:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
