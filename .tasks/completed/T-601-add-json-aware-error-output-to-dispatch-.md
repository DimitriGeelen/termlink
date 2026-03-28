---
id: T-601
name: "Add JSON-aware error output to dispatch command validation errors"
description: >
  Add JSON-aware error output to dispatch command validation errors

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T16:51:05Z
last_update: 2026-03-28T16:51:48Z
date_finished: 2026-03-28T16:51:48Z
---

# T-601: Add JSON-aware error output to dispatch command validation errors

## Context

The `cmd_dispatch` function in dispatch.rs has three early validation bail! calls (count==0, empty command, hub not running) that don't output JSON when `--json` is passed.

## Acceptance Criteria

### Agent
- [x] `cmd_dispatch` count==0 validation outputs JSON error when `--json` is passed
- [x] `cmd_dispatch` empty command validation outputs JSON error when `--json` is passed
- [x] `cmd_dispatch` hub-not-running validation outputs JSON error when `--json` is passed
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

### 2026-03-28T16:51:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-601-add-json-aware-error-output-to-dispatch-.md
- **Context:** Initial task creation

### 2026-03-28T16:51:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
