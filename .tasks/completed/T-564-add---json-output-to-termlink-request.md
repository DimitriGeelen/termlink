---
id: T-564
name: "Add --json output to termlink request"
description: >
  Add --json output to termlink request

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:33:40Z
last_update: 2026-03-28T12:35:08Z
date_finished: 2026-03-28T12:35:08Z
---

# T-564: Add --json output to termlink request

## Context

Add `--json` flag to `termlink request` for machine-readable request-reply output.

## Acceptance Criteria

### Agent
- [x] `Request` variant in cli.rs has `json: bool` field
- [x] `cmd_request` outputs structured JSON for both sent and reply when --json is passed
- [x] All existing tests pass

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

cargo clippy -p termlink -- -D warnings 2>&1 | tail -1 | grep -qv error

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

### 2026-03-28T12:33:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-564-add---json-output-to-termlink-request.md
- **Context:** Initial task creation

### 2026-03-28T12:35:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
