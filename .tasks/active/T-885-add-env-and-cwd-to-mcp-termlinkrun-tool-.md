---
id: T-885
name: "Add env and cwd to MCP termlink_run tool — run commands with custom environment"
description: >
  Add env and cwd to MCP termlink_run tool — run commands with custom environment

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T07:14:28Z
last_update: 2026-04-05T07:14:28Z
date_finished: null
---

# T-885: Add env and cwd to MCP termlink_run tool — run commands with custom environment

## Context

MCP `termlink_run` lacks `env` and `cwd` parameters that exist on `termlink_exec`. AI agents using `run` for ephemeral commands can't set environment or working directory.

## Acceptance Criteria

### Agent
- [x] `RunParams` has `env: Option<HashMap<String, String>>` field
- [x] `RunParams` has `cwd: Option<String>` field
- [x] `termlink_run` passes env vars to executor
- [x] `termlink_run` passes cwd to executor
- [x] Unit test for RunParams with new fields
- [x] `cargo build` succeeds
- [x] `cargo clippy -p termlink-mcp --all-targets` has no warnings

## Verification

cargo build
cargo clippy -p termlink-mcp --all-targets

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

### 2026-04-05T07:14:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-885-add-env-and-cwd-to-mcp-termlinkrun-tool-.md
- **Context:** Initial task creation
