---
id: T-736
name: "Add ok:true to event watch, wait, topics, collect JSON responses"
description: >
  Add ok:true to event watch, wait, topics, collect JSON responses

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T13:16:42Z
last_update: 2026-03-29T13:18:40Z
date_finished: 2026-03-29T13:18:40Z
---

# T-736: Add ok:true to event watch, wait, topics, collect JSON responses

## Context

Add `"ok": true` to streaming JSON outputs (watch, collect per-event lines), `cmd_wait` matched/timeout responses, and `cmd_topics` JSON output.

## Acceptance Criteria

### Agent
- [x] `cmd_watch` streaming JSON events include `"ok": true`
- [x] `cmd_wait` matched response includes `"ok": true`, timeout/interrupted include `"ok": false`
- [x] `cmd_topics` JSON success output includes `"ok": true` (both empty and non-empty cases)
- [x] `cmd_collect` streaming JSON events include `"ok": true`
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

### 2026-03-29T13:16:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-736-add-oktrue-to-event-watch-wait-topics-co.md
- **Context:** Initial task creation

### 2026-03-29T13:18:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
