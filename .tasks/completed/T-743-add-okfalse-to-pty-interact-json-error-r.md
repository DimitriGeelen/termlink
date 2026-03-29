---
id: T-743
name: "Add ok:false to pty interact JSON error responses"
description: >
  Add ok:false to pty interact JSON error responses

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T13:38:24Z
last_update: 2026-03-29T13:39:44Z
date_finished: 2026-03-29T13:39:44Z
---

# T-743: Add ok:false to pty interact JSON error responses

## Context

`pty interact` JSON error responses (no PTY, timeout, poll failure) are missing `"ok": false`.

## Acceptance Criteria

### Agent
- [x] "Session has no PTY" error includes `"ok": false`
- [x] Timeout error includes `"ok": false`
- [x] Poll failure error includes `"ok": false`
- [x] Project compiles with `cargo build`

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

### 2026-03-29T13:38:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-743-add-okfalse-to-pty-interact-json-error-r.md
- **Context:** Initial task creation

### 2026-03-29T13:39:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
