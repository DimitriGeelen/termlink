---
id: T-084
name: "Per-method permission enforcement in session handler"
description: >
  Wire T-078 PermissionScope into session handler dispatch — check method_scope against connection's granted scope before executing. Addresses G-002.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T22:29:22Z
last_update: 2026-03-10T22:32:52Z
date_finished: 2026-03-10T22:32:52Z
---

# T-084: Per-method permission enforcement in session handler

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Per-method scope check in handle_connection before dispatch
- [x] Observe-scoped connections denied Execute/Control/Interact methods
- [x] Execute-scoped connections allowed all methods
- [x] Permission denied returns JSON-RPC error (-32603) with scope details
- [x] Same-UID connections granted Execute scope (backward compatible)
- [x] 2 new tests: scope denial + scope allowance
- [x] All 195 workspace tests pass
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

### 2026-03-10T22:29:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-084-per-method-permission-enforcement-in-ses.md
- **Context:** Initial task creation

### 2026-03-10T22:32:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
