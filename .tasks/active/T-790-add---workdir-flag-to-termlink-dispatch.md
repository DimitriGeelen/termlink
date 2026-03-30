---
id: T-790
name: "Add --workdir flag to termlink dispatch"
description: >
  Phase 1 stepping stone for T-789 worktree isolation

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-789, T-791]
created: 2026-03-30T13:35:04Z
last_update: 2026-03-30T13:36:01Z
date_finished: null
---

# T-790: Add --workdir flag to termlink dispatch

## Context

Phase 1 of T-789 (worktree isolation). Adds `--workdir` flag to `termlink dispatch` so workers can be spawned in a specified directory. This is the stepping stone for `--isolate` (T-791) — useful on its own for manual worktree setups. See `docs/reports/T-789-worktree-isolation-research.md`.

## Acceptance Criteria

### Agent
- [x] `--workdir <path>` flag added to Dispatch CLI struct in `cli.rs`
- [x] `cmd_dispatch()` accepts and propagates workdir parameter
- [x] Workers spawned with `cd <workdir> &&` prepended to shell command when --workdir is set
- [x] `TERMLINK_WORKDIR` env var injected into worker environment
- [x] `--workdir` without a valid directory path returns clear error (not panic)
- [x] `--workdir` appears in `termlink dispatch --help` output
- [x] JSON output includes `workdir` field when --workdir is used
- [x] Unit test: workdir validation rejects non-existent path
- [x] Unit test: dispatch with --workdir validates and accepts valid directory
- [x] All existing dispatch tests still pass
- [x] `cargo clippy --workspace` has zero warnings
- [x] `cargo test --workspace` passes (635 tests, 0 failures)

## Verification

cargo test -p termlink dispatch 2>&1 | grep -q "0 failed"
cargo clippy --workspace 2>&1 | grep -c "warning" | grep -q "^0$"
grep -q "workdir" crates/termlink-cli/src/cli.rs
grep -q "TERMLINK_WORKDIR" crates/termlink-cli/src/commands/dispatch.rs

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

### 2026-03-30T13:35:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-790-add---workdir-flag-to-termlink-dispatch.md
- **Context:** Initial task creation

### 2026-03-30T13:36:01Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
