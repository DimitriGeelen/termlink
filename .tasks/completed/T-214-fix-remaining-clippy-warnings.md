---
id: T-214
name: "Fix remaining clippy warnings"
description: >
  Fix remaining clippy warnings

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/main.rs, crates/termlink-session/src/auth.rs]
related_tasks: []
created: 2026-03-22T17:24:41Z
last_update: 2026-03-22T17:27:12Z
date_finished: 2026-03-22T17:27:12Z
---

# T-214: Fix remaining clippy warnings

## Context

2 clippy warnings from Rust 1.94 (unnecessary_map_or) in main.rs and termlink-session.

## Acceptance Criteria

### Agent
- [x] Zero clippy warnings across workspace
- [x] All tests pass (297 pass, 0 fail)

## Verification

# Verify the specific fixes are in place
grep -q "is_none_or" crates/termlink-cli/src/main.rs
grep -q "let Some(expected) = expected_session_id" crates/termlink-session/src/auth.rs

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

### 2026-03-22T17:24:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-214-fix-remaining-clippy-warnings.md
- **Context:** Initial task creation

### 2026-03-22T17:27:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
