---
id: T-603
name: "Add JSON-aware error output to token create and inspect validation errors"
description: >
  Add JSON-aware error output to token create and inspect validation errors

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T16:54:11Z
last_update: 2026-03-28T16:54:57Z
date_finished: 2026-03-28T16:54:57Z
---

# T-603: Add JSON-aware error output to token create and inspect validation errors

## Context

Token create has a bail! for invalid secret length and token inspect has a bail! for invalid format — both without JSON-aware output.

## Acceptance Criteria

### Agent
- [x] `cmd_token_create` invalid secret length outputs JSON error when `--json` is passed
- [x] `cmd_token_inspect` invalid format outputs JSON error when `--json` is passed
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

### 2026-03-28T16:54:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-603-add-json-aware-error-output-to-token-cre.md
- **Context:** Initial task creation

### 2026-03-28T16:54:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
