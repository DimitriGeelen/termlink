---
id: T-683
name: "Add --strict flag to doctor that exits non-zero on warnings"
description: >
  Add --strict flag to doctor that exits non-zero on warnings

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T23:03:22Z
last_update: 2026-03-28T23:03:22Z
date_finished: null
---

# T-683: Add --strict flag to doctor that exits non-zero on warnings

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `--strict` flag added to Doctor command in cli.rs
- [ ] `strict` param threaded to cmd_doctor (partially done — main.rs dispatch + infrastructure.rs function signature + exit logic needed)
- [ ] Exit 1 when warnings present and --strict is set
- [ ] Without --strict, warnings don't cause non-zero exit
- [ ] Project compiles cleanly

### Human
<!-- Remove if not needed.
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

### 2026-03-28T23:03:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-683-add---strict-flag-to-doctor-that-exits-n.md
- **Context:** Initial task creation
