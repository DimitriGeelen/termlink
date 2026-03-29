---
id: T-741
name: "Fix pty interact --json not propagating non-zero exit code and always reporting ok:true"
description: >
  Fix pty interact --json not propagating non-zero exit code and always reporting ok:true

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T13:32:29Z
last_update: 2026-03-29T13:33:45Z
date_finished: 2026-03-29T13:33:45Z
---

# T-741: Fix pty interact --json not propagating non-zero exit code and always reporting ok:true

## Context

`pty interact --json` always reports `"ok": true` and always exits 0, even when the executed command returns a non-zero exit code. Both should be conditional on exit_code.

## Acceptance Criteria

### Agent
- [x] `ok` field reflects exit code: `true` when 0 or null, `false` when non-zero
- [x] Process exits with the command's exit code in JSON mode when non-zero
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

### 2026-03-29T13:32:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-741-fix-pty-interact---json-not-propagating-.md
- **Context:** Initial task creation

### 2026-03-29T13:33:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
