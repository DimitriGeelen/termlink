---
id: T-856
name: "Fix dispatch test ordering bug — mutual exclusion check must precede git repo check"
description: >
  Fix dispatch test ordering bug — mutual exclusion check must precede git repo check

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/dispatch.rs]
related_tasks: []
created: 2026-04-04T19:01:38Z
last_update: 2026-04-04T19:04:26Z
date_finished: 2026-04-04T19:04:26Z
---

# T-856: Fix dispatch test ordering bug — mutual exclusion check must precede git repo check

## Context

`isolate_and_workdir_mutually_exclusive` test fails when run in full workspace because `isolate_rejects_non_git_dir` changes CWD to a non-git temp dir. The git repo check fires before the mutual exclusion check, producing wrong error message.

## Acceptance Criteria

### Agent
- [x] Mutual exclusion check (`--isolate` + `--workdir`) runs before git repository check in `cmd_dispatch`
- [x] `cargo test --workspace` passes with 0 failures (including the previously-flaky test)
- [x] Zero clippy warnings: `cargo clippy --workspace`

## Verification

cargo test --workspace
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

### 2026-04-04T19:01:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-856-fix-dispatch-test-ordering-bug--mutual-e.md
- **Context:** Initial task creation

### 2026-04-04T19:04:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
