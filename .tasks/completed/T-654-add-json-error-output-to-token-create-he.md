---
id: T-654
name: "Add JSON error output to token create hex and scope errors"
description: >
  Add JSON error output to token create hex and scope errors

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T20:09:23Z
last_update: 2026-03-28T20:10:15Z
date_finished: 2026-03-28T20:10:15Z
---

# T-654: Add JSON error output to token create hex and scope errors

## Context

Token create has hex parsing and scope parsing errors without JSON output.

## Acceptance Criteria

### Agent
- [x] token create hex and scope errors have JSON error output
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

### 2026-03-28T20:09:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-654-add-json-error-output-to-token-create-he.md
- **Context:** Initial task creation

### 2026-03-28T20:10:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
