---
id: T-751
name: "CLI hardening — fix unwraps, add missing integration tests"
description: >
  CLI hardening — fix unwraps, add missing integration tests

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T18:27:48Z
last_update: 2026-03-29T18:27:48Z
date_finished: null
---

# T-751: CLI hardening — fix unwraps, add missing integration tests

## Context

Fix unsafe unwrap calls in CLI and MCP code that can panic at runtime, and replace with proper error handling. Also add integration tests for untested CLI commands.

## Acceptance Criteria

### Agent
- [ ] No `.unwrap()` calls in non-test CLI/MCP source code (push.rs, tools.rs)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] Integration tests added for `doctor`, `version`/`info`, and `hub` commands

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
