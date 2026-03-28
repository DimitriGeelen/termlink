---
id: T-613
name: "Add JSON-aware error output to inject timeout and inject error"
description: >
  Add JSON-aware error output to inject timeout and inject error

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:13:31Z
last_update: 2026-03-28T17:14:33Z
date_finished: 2026-03-28T17:14:33Z
---

# T-613: Add JSON-aware error output to inject timeout and inject error

## Context

cmd_inject has a bail! on timeout and on unwrap_result error without JSON-aware output.

## Acceptance Criteria

### Agent
- [x] inject timeout outputs JSON error when --json is passed
- [x] inject unwrap_result error outputs JSON error when --json is passed (already existed)
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

### 2026-03-28T17:13:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-613-add-json-aware-error-output-to-inject-ti.md
- **Context:** Initial task creation

### 2026-03-28T17:14:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
