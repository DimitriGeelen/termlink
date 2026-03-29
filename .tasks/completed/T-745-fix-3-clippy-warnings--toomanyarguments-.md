---
id: T-745
name: "Fix 3 clippy warnings — too_many_arguments in agent.rs, collapsible_if in file.rs and vendor.rs"
description: >
  Fix 3 clippy warnings — too_many_arguments in agent.rs, collapsible_if in file.rs and vendor.rs

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/agent.rs, crates/termlink-cli/src/commands/events.rs, crates/termlink-cli/src/commands/execution.rs, crates/termlink-cli/src/commands/file.rs, crates/termlink-cli/src/commands/metadata.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/commands/session.rs, crates/termlink-cli/src/commands/vendor.rs]
related_tasks: []
created: 2026-03-29T14:06:02Z
last_update: 2026-03-29T14:11:02Z
date_finished: 2026-03-29T14:11:02Z
---

# T-745: Fix 3 clippy warnings — too_many_arguments in agent.rs, collapsible_if in file.rs and vendor.rs

## Context

GitHub CI (now removed in T-744) was failing on 3 clippy lints. Fix them so the codebase is clippy-clean.

## Acceptance Criteria

### Agent
- [x] Add `#[allow(clippy::too_many_arguments)]` to `cmd_agent_negotiate` in agent.rs
- [x] Collapse nested `if` in file.rs:236 (collapsible_if)
- [x] Collapse nested `if` in vendor.rs:149 (collapsible_if)
- [x] Fix remaining 10 clippy errors (session.rs, events.rs, metadata.rs, execution.rs, remote.rs)
- [x] `cargo clippy --workspace -- -D warnings` passes

## Verification

cargo clippy --workspace -- -D warnings

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

### 2026-03-29T14:06:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-745-fix-3-clippy-warnings--toomanyarguments-.md
- **Context:** Initial task creation

### 2026-03-29T14:11:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
