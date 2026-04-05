---
id: T-883
name: "Add --env flag to dispatch CLI and MCP tool — pass environment variables to workers"
description: >
  Add --env flag to dispatch CLI and MCP tool — pass environment variables to workers

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/dispatch.rs, crates/termlink-cli/src/main.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T06:58:33Z
last_update: 2026-04-05T07:04:47Z
date_finished: 2026-04-05T07:04:47Z
---

# T-883: Add --env flag to dispatch CLI and MCP tool — pass environment variables to workers

## Context

AI agents dispatching workers often need to pass configuration via environment variables. Currently there's no way to pass custom env vars to dispatch workers via CLI or MCP.

## Acceptance Criteria

### Agent
- [x] CLI `Dispatch` has `--env KEY=VALUE` flag (repeatable)
- [x] `DispatchOpts` struct has `env: Vec<String>` field
- [x] CLI dispatch passes env vars as `export KEY=VALUE;` in worker shell command
- [x] MCP `DispatchParams` has `env: Option<HashMap<String, String>>` field
- [x] MCP dispatch passes env vars to worker shell commands
- [x] Unit test for env var injection
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

### 2026-04-05T06:58:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-883-add---env-flag-to-dispatch-cli-and-mcp-t.md
- **Context:** Initial task creation

### 2026-04-05T07:04:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
