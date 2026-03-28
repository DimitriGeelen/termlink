---
id: T-657
name: "Add JSON error output to register PTY spawn and signal parse errors"
description: >
  Add JSON error output to register PTY spawn and signal parse errors

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T20:14:34Z
last_update: 2026-03-28T20:16:32Z
date_finished: 2026-03-28T20:16:32Z
---

# T-657: Add JSON error output to register PTY spawn and signal parse errors

## Context

Register PTY spawn, persist registration, list sessions, and signal parse use `.context()` without JSON output.

## Acceptance Criteria

### Agent
- [x] PTY spawn, persist, list, and signal parse errors have JSON error output
- [x] `cargo check -p termlink` passes

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

### 2026-03-28T20:14:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-657-add-json-error-output-to-register-pty-sp.md
- **Context:** Initial task creation

### 2026-03-28T20:16:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
