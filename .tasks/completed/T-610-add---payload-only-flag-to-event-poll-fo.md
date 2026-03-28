---
id: T-610
name: "Add --payload-only flag to event poll for piping just event payloads"
description: >
  Add --payload-only flag to event poll for piping just event payloads

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:04:44Z
last_update: 2026-03-28T17:06:59Z
date_finished: 2026-03-28T17:06:59Z
---

# T-610: Add --payload-only flag to event poll for piping just event payloads

## Context

Add `--payload-only` flag to `event poll` so each event's payload is printed as one JSON line (NDJSON), easy for `jq` piping.

## Acceptance Criteria

### Agent
- [x] `--payload-only` flag added to EventCommand::Poll in cli.rs
- [x] cmd_events outputs one JSON payload per line when --payload-only is set
- [x] Hidden alias Command::Events updated too
- [x] main.rs updated
- [x] Project compiles with `cargo check`

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

### 2026-03-28T17:04:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-610-add---payload-only-flag-to-event-poll-fo.md
- **Context:** Initial task creation

### 2026-03-28T17:06:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
