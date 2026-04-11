---
id: T-919
name: "Add unit tests for termlink remote commands (T-186 test gap)"
description: >
  Add unit tests for termlink remote commands (T-186 test gap)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-04-11T14:53:13Z
last_update: 2026-04-11T14:55:53Z
date_finished: 2026-04-11T14:55:53Z
---

# T-919: Add unit tests for termlink remote commands (T-186 test gap)

## Context

`crates/termlink-cli/src/commands/remote.rs` shipped as part of T-186
(`termlink remote inject/ping/list/status/send-file/events` — the cross-machine
CLI family) with 1229 lines of code and zero tests. This task closes the
coverage gap by testing the pure-validation paths of `connect_remote_hub`,
which is the shared entry point for every remote subcommand and does all the
argument/secret/scope parsing before touching the network.

The tests exercise failure paths that error out before any I/O, so they
require no live hub, no TLS setup, and no network fixtures.

## Acceptance Criteria

### Agent
- [x] `commands/remote.rs` has a `#[cfg(test)] mod tests` block with async tests covering `connect_remote_hub` validation
- [x] Tests cover hub-address parsing errors (missing colon, extra colons, non-numeric port)
- [x] Tests cover secret errors (missing, wrong length, invalid hex, missing secret file)
- [x] Tests cover scope validation (unknown scope rejected; all four valid scopes pass validation without reaching the "Invalid scope" error)
- [x] `cargo test -p termlink --bin termlink commands::remote::tests` passes (all new tests green)
- [x] `cargo build --workspace` stays clean

## Verification

cargo build --workspace --quiet
cargo test --package termlink --bin termlink commands::remote::tests -- --quiet

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

### 2026-04-11T14:53:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-919-add-unit-tests-for-termlink-remote-comma.md
- **Context:** Initial task creation

### 2026-04-11T14:55:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
