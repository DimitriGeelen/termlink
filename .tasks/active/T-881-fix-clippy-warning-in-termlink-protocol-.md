---
id: T-881
name: "Fix clippy warning in termlink-protocol — items after test module"
description: >
  Fix clippy warning in termlink-protocol — items after test module

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T06:42:15Z
last_update: 2026-04-05T06:44:04Z
date_finished: null
---

# T-881: Fix clippy warning in termlink-protocol — items after test module

## Context

Clippy warns about `items_after_test_module` in protocol crate — constants defined after `#[cfg(test)] mod tests`. Move constants before the test module.

## Acceptance Criteria

### Agent
- [x] Constants `DATA_PLANE_VERSION`, `FRAME_MAGIC`, `FRAME_HEADER_SIZE`, `MAX_PAYLOAD_SIZE` appear before `#[cfg(test)]` in lib.rs
- [x] `cargo clippy -p termlink-protocol --all-targets` produces zero warnings
- [x] `cargo test -p termlink-protocol` passes

## Verification

cargo clippy -p termlink-protocol --all-targets
cargo test -p termlink-protocol

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

### 2026-04-05T06:42:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-881-fix-clippy-warning-in-termlink-protocol-.md
- **Context:** Initial task creation
