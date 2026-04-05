---
id: T-898
name: "Fix misplaced doc comments in protocol crate — format_age docs on wrong constant"
description: >
  Fix misplaced doc comments in protocol crate — format_age docs on wrong constant

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T08:41:09Z
last_update: 2026-04-05T08:41:09Z
date_finished: null
---

# T-898: Fix misplaced doc comments in protocol crate — format_age docs on wrong constant

## Context

T-881 moved constants before `#[cfg(test)]` but left `format_age` doc comment merged with `DATA_PLANE_VERSION` doc. Fix so each item has its own correct doc comment.

## Acceptance Criteria

### Agent
- [x] `DATA_PLANE_VERSION` has its own correct doc comment
- [x] `format_age` function has its own correct doc comment
- [x] All tests pass, zero clippy warnings

## Verification

cargo test -p termlink-protocol
cargo clippy -p termlink-protocol --all-targets

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

### 2026-04-05T08:41:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-898-fix-misplaced-doc-comments-in-protocol-c.md
- **Context:** Initial task creation
