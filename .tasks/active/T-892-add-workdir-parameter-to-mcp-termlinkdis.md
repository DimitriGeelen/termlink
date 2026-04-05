---
id: T-892
name: "Add workdir parameter to MCP termlink_dispatch — worker directory control"
description: >
  Add workdir parameter to MCP termlink_dispatch — worker directory control

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T07:57:03Z
last_update: 2026-04-05T07:57:03Z
date_finished: null
---

# T-892: Add workdir parameter to MCP termlink_dispatch — worker directory control

## Context

The CLI `termlink dispatch --workdir <path>` lets workers cd into a directory before executing. The MCP `termlink_dispatch` tool lacks this parameter. Adding it for parity.

## Acceptance Criteria

### Agent
- [x] `DispatchParams` has `workdir: Option<String>` field
- [x] MCP dispatch prepends `cd <workdir> &&` to shell command when workdir is set
- [x] Unit test for DispatchParams with workdir field
- [x] All tests pass (`cargo test --workspace`)
- [x] Zero clippy warnings

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

### 2026-04-05T07:57:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-892-add-workdir-parameter-to-mcp-termlinkdis.md
- **Context:** Initial task creation
