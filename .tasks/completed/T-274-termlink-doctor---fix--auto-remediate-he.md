---
id: T-274
name: "termlink doctor --fix — auto-remediate health issues"
description: >
  termlink doctor --fix — auto-remediate health issues

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-03-25T12:15:58Z
last_update: 2026-03-25T12:19:42Z
date_finished: 2026-03-25T12:19:42Z
---

# T-274: termlink doctor --fix — auto-remediate health issues

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `--fix` flag added to `termlink doctor`
- [x] Fixes: stale sessions (clean), stale hub pidfile+socket, orphaned sockets
- [x] All 459 workspace tests pass


## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

grep -q "Auto-fix" crates/termlink-cli/src/cli.rs

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

### 2026-03-25T12:15:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-274-termlink-doctor---fix--auto-remediate-he.md
- **Context:** Initial task creation

### 2026-03-25T12:19:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
