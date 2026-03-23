---
id: T-244
name: "Fix termlink attach output freezing — delta calculation bug"
description: >
  Fix termlink attach output freezing — delta calculation bug

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-23T15:34:16Z
last_update: 2026-03-23T22:00:22Z
date_finished: 2026-03-23T16:13:48Z
---

# T-244: Fix termlink attach output freezing — delta calculation bug

## Context

Fix delta calculation bug in attach_loop (pty.rs) where scrollback > 8192 bytes causes output to freeze.

## Acceptance Criteria

### Agent
- [x] Delta calculation handles case where delta >= output buffer length
- [x] All workspace tests pass (297 tests)


## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink
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

### 2026-03-23T15:34:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-244-fix-termlink-attach-output-freezing--del.md
- **Context:** Initial task creation

### 2026-03-23T16:13:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
