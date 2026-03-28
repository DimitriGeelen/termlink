---
id: T-559
name: "Add --count flag to termlink list for quick session counting"
description: >
  Add --count flag to termlink list for quick session counting

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T10:07:58Z
last_update: 2026-03-28T10:07:58Z
date_finished: null
---

# T-559: Add --count flag to termlink list for quick session counting

## Context

Useful for scripting: `termlink list --count` just outputs the number.

## Acceptance Criteria

### Agent
- [x] `--count` flag added to List command in cli.rs
- [x] When --count is set, only the session count number is printed
- [x] Works with filters (--tag, --name, --role)
- [x] Builds without warnings
- [x] Integration test added (count with 1 session, count with 0 sessions)

### Human
<!-- No human ACs.
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

### 2026-03-28T10:07:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-559-add---count-flag-to-termlink-list-for-qu.md
- **Context:** Initial task creation
