---
id: T-837
name: "Add unit tests for CLI token and infrastructure commands"
description: >
  Add unit tests for CLI token and infrastructure commands

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/execution.rs]
related_tasks: []
created: 2026-04-03T23:20:57Z
last_update: 2026-04-03T23:29:08Z
date_finished: 2026-04-03T23:29:08Z
---

# T-837: Add unit tests for CLI token and infrastructure commands

## Context

Add unit tests for `build_spawn_shell_cmd` and `resolve_spawn_backend` in execution.rs — the largest untested pure functions in the CLI crate.

## Acceptance Criteria

### Agent
- [x] Unit tests added for `build_spawn_shell_cmd` covering: shell-only, with command, with roles/tags/cap
- [x] Unit tests added for `resolve_spawn_backend` non-Auto paths
- [x] `cargo test --workspace` passes
- [x] Total test count increases

## Verification

cargo test --workspace 2>&1 | tail -3
cargo test -p termlink -- execution::tests 2>&1 | grep "test result" | head -1

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

### 2026-04-03T23:20:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-837-add-unit-tests-for-cli-token-and-infrast.md
- **Context:** Initial task creation

### 2026-04-03T23:29:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
