---
id: T-609
name: "Add JSON-aware error output to spawn command backend failures"
description: >
  Add JSON-aware error output to spawn command backend failures

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:03:27Z
last_update: 2026-03-28T17:04:25Z
date_finished: 2026-03-28T17:04:25Z
---

# T-609: Add JSON-aware error output to spawn command backend failures

## Context

The `spawn_via_terminal` and `spawn_via_tmux` helper functions bail! without JSON output. Since `cmd_spawn` has `--json`, these errors should be wrapped.

## Acceptance Criteria

### Agent
- [x] spawn_via_terminal error wrapped with JSON output when json is true
- [x] spawn_via_tmux error wrapped with JSON output when json is true
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

### 2026-03-28T17:03:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-609-add-json-aware-error-output-to-spawn-com.md
- **Context:** Initial task creation

### 2026-03-28T17:04:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
