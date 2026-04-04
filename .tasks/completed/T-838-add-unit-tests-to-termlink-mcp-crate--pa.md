---
id: T-838
name: "Add unit tests to termlink-mcp crate — parse_signal, parameter serde, filter extraction"
description: >
  Add unit tests to termlink-mcp crate — parse_signal, parameter serde, filter extraction

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/metadata.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/commands/session.rs, crates/termlink-cli/src/main.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-03T23:47:39Z
last_update: 2026-04-04T00:00:13Z
date_finished: 2026-04-04T00:00:13Z
---

# T-838: Add unit tests to termlink-mcp crate — parse_signal, parameter serde, filter extraction

## Context

The termlink-mcp crate (tools.rs: 2210 lines, server.rs: 412 lines) has zero unit tests. Adding tests for pure functions: parse_signal, parameter struct deserialization, and the session filter logic in the CLI layer.

## Acceptance Criteria

### Agent
- [x] `parse_signal` in tools.rs has tests covering all named signals, SIG-prefixed variants, numeric input, and invalid input
- [x] Parameter struct deserialization tests verify required vs optional fields for at least 3 param types
- [x] `filter_sessions` extracted as standalone function in CLI session.rs with unit tests for tag, name, role, and capability filtering
- [x] All new tests pass: `cargo test -p termlink-mcp` and `cargo test -p termlink -- session::tests`
- [x] Zero clippy warnings: `cargo clippy --workspace --all-targets`

## Verification

cargo test -p termlink-mcp --lib 2>&1 | tail -5
cargo test -p termlink -- session::tests 2>&1 | tail -5
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c 'warning:')" = "0"

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

### 2026-04-03T23:47:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-838-add-unit-tests-to-termlink-mcp-crate--pa.md
- **Context:** Initial task creation

### 2026-04-04T00:00:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
