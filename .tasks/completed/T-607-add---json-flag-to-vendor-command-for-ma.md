---
id: T-607
name: "Add --json flag to vendor command for machine-readable output"
description: >
  Add --json flag to vendor command for machine-readable output

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T16:59:49Z
last_update: 2026-03-28T17:01:21Z
date_finished: 2026-03-28T17:01:21Z
---

# T-607: Add --json flag to vendor command for machine-readable output

## Context

`termlink vendor` only has human-readable output. Adding `--json` enables automated vendor workflows.

## Acceptance Criteria

### Agent
- [x] `--json` flag added to Vendor command in cli.rs
- [x] cmd_vendor accepts json param and outputs structured JSON on success
- [x] main.rs updated to pass json through
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

### 2026-03-28T16:59:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-607-add---json-flag-to-vendor-command-for-ma.md
- **Context:** Initial task creation

### 2026-03-28T17:01:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
