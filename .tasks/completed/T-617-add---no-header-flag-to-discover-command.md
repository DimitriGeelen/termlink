---
id: T-617
name: "Add --no-header flag to discover command to suppress table header"
description: >
  Add --no-header flag to discover command to suppress table header

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:21:31Z
last_update: 2026-03-28T17:22:57Z
date_finished: 2026-03-28T17:22:57Z
---

# T-617: Add --no-header flag to discover command to suppress table header

## Context

Same as T-616 for `list`. Suppress header/footer in discover output for piping.

## Acceptance Criteria

### Agent
- [x] `--no-header` flag added to Discover in cli.rs
- [x] Header, separator, and footer suppressed when --no-header
- [x] cmd_discover and main.rs updated
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

### 2026-03-28T17:21:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-617-add---no-header-flag-to-discover-command.md
- **Context:** Initial task creation

### 2026-03-28T17:22:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
