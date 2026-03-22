---
id: T-214
name: "Fix remaining clippy warnings"
description: >
  Fix remaining clippy warnings

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-22T17:24:41Z
last_update: 2026-03-22T17:24:41Z
date_finished: null
---

# T-214: Fix remaining clippy warnings

## Context

2 clippy warnings from Rust 1.94 (unnecessary_map_or) in main.rs and termlink-session.

## Acceptance Criteria

### Agent
- [x] Zero clippy warnings across workspace
- [x] All tests pass (297 pass, 0 fail)

## Verification

cargo clippy --workspace 2>&1 | grep -q "warning:" && exit 1 || exit 0
cargo test --workspace 2>&1 | grep -q "FAILED" && exit 1 || exit 0

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
