---
id: T-269
name: "Fix hub router compiler warnings + cleanup"
description: >
  Fix hub router compiler warnings + cleanup

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T09:53:33Z
last_update: 2026-03-25T09:57:29Z
date_finished: 2026-03-25T09:57:29Z
---

# T-269: Fix hub router compiler warnings + cleanup

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Zero compiler warnings in `cargo test --workspace`
- [x] All 451 tests pass
- [x] `skipped_count` used in orchestrator.route error message
## Verification

grep -q "skipped_count" crates/termlink-hub/src/router.rs

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

### 2026-03-25T09:53:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-269-fix-hub-router-compiler-warnings--cleanu.md
- **Context:** Initial task creation

### 2026-03-25T09:57:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
