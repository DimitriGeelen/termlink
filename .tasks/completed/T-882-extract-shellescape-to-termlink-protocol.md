---
id: T-882
name: "Extract shell_escape to termlink-protocol — remove 3-way duplication"
description: >
  Extract shell_escape to termlink-protocol — remove 3-way duplication

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/push.rs, crates/termlink-cli/src/util.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-protocol/src/lib.rs]
related_tasks: []
created: 2026-04-05T06:48:35Z
last_update: 2026-04-05T06:54:24Z
date_finished: 2026-04-05T06:54:24Z
---

# T-882: Extract shell_escape to termlink-protocol — remove 3-way duplication

## Context

`shell_escape` duplicated in CLI util.rs, CLI push.rs, and MCP tools.rs. Same pattern as T-874 (format_age). Move to protocol crate.

## Acceptance Criteria

### Agent
- [x] `pub fn shell_escape()` exists in `termlink-protocol/src/lib.rs`
- [x] CLI `util.rs` uses `termlink_protocol::shell_escape` instead of local definition
- [x] CLI `push.rs` uses `termlink_protocol::shell_escape` instead of local definition
- [x] MCP `tools.rs` uses `termlink_protocol::shell_escape` instead of `mcp_shell_escape`
- [x] Tests for `shell_escape` exist in protocol crate
- [x] `cargo test --workspace` passes
- [x] `cargo clippy --workspace --all-targets` produces no new warnings

## Verification

cargo test --workspace
cargo clippy --workspace --all-targets

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

### 2026-04-05T06:48:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-882-extract-shellescape-to-termlink-protocol.md
- **Context:** Initial task creation

### 2026-04-05T06:54:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
