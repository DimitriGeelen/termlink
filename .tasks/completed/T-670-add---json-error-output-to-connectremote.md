---
id: T-670
name: "Add --json error output to connect_remote_hub argument validation"
description: >
  Add --json error output to connect_remote_hub argument validation

status: work-completed
workflow_type: build
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-03-28T21:42:05Z
last_update: 2026-03-29T14:19:50Z
date_finished: 2026-03-29T14:19:50Z
---

# T-670: Add --json error output to connect_remote_hub argument validation

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] All callers of connect_remote_hub wrap errors in JSON when --json is set

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

### 2026-03-28T21:42:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-670-add---json-error-output-to-connectremote.md
- **Context:** Initial task creation

### 2026-03-28T21:42:17Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** connect_remote_hub is a helper — callers already wrap errors with JSON output

### 2026-03-29T11:28:42Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-03-29T14:19:50Z — status-update [task-update-agent]
- **Change:** status: issues → work-completed
