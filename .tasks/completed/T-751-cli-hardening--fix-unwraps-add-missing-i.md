---
id: T-751
name: "CLI hardening — fix unwraps, add missing integration tests"
description: >
  CLI hardening — fix unwraps, add missing integration tests

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/push.rs, crates/termlink-cli/tests/cli_integration.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-03-29T18:27:48Z
last_update: 2026-03-29T18:38:12Z
date_finished: 2026-03-29T18:38:12Z
---

# T-751: CLI hardening — fix unwraps, add missing integration tests

## Context

Fix unsafe unwrap calls in CLI and MCP code that can panic at runtime, and replace with proper error handling. Also add integration tests for untested CLI commands.

## Acceptance Criteria

### Agent
- [x] No `.unwrap()` calls in non-test CLI/MCP source code (push.rs, tools.rs)
- [x] `cargo clippy --workspace -- -D warnings` passes
- [x] `cargo test --workspace` passes (520 tests, 0 failures)
- [x] Integration tests added for token, signal, file-send, dispatch, info, hub-status, doctor, list filters (12 new tests)

## Verification

cargo clippy --workspace -- -D warnings 2>&1 | tail -1
cargo test --workspace 2>&1 | grep -E "^test result:" | grep -v FAILED

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

### 2026-03-29T18:27:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-751-cli-hardening--fix-unwraps-add-missing-i.md
- **Context:** Initial task creation

### 2026-03-29T18:38:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
