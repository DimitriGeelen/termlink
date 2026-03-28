---
id: T-574
name: "Wire --json flag for event topics command"
description: >
  Wire --json flag for event topics command

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/events.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-28T15:28:30Z
last_update: 2026-03-28T15:29:53Z
date_finished: 2026-03-28T15:29:53Z
---

# T-574: Wire --json flag for event topics command

## Context

The --json flag exists in cli.rs for EventCommand::Topics and the hidden Topics alias, but the dispatch in main.rs discards it (`json: _`) and cmd_topics doesn't accept it.

## Acceptance Criteria

### Agent
- [x] main.rs passes json flag to cmd_topics for both EventCommand::Topics and hidden Topics alias
- [x] cmd_topics accepts json: bool parameter and outputs structured JSON when set
- [x] cargo build succeeds with no warnings

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

### 2026-03-28T15:28:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-574-wire---json-flag-for-event-topics-comman.md
- **Context:** Initial task creation

### 2026-03-28T15:29:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
