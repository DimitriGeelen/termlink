---
id: T-604
name: "Add --raw flag to kv get for piping-friendly value output"
description: >
  Add --raw flag to kv get for piping-friendly value output

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T16:55:04Z
last_update: 2026-03-28T16:56:27Z
date_finished: 2026-03-28T16:56:27Z
---

# T-604: Add --raw flag to kv get for piping-friendly value output

## Context

Currently `kv get` outputs pretty-printed JSON. A `--raw` flag would output strings without quotes and other types as compact JSON — better for piping.

## Acceptance Criteria

### Agent
- [x] `--raw` flag added to Kv command in cli.rs (bool, default false)
- [x] When --raw and value is a string, output bare string without JSON quotes
- [x] When --raw and value is not a string, output compact JSON
- [x] main.rs passes raw parameter through
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

### 2026-03-28T16:55:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-604-add---raw-flag-to-kv-get-for-piping-frie.md
- **Context:** Initial task creation

### 2026-03-28T16:56:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
