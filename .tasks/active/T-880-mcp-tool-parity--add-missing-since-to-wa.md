---
id: T-880
name: "MCP tool parity — add missing since to wait, cap to spawn and dispatch"
description: >
  MCP tool parity — add missing since to wait, cap to spawn and dispatch

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T06:35:54Z
last_update: 2026-04-05T06:35:54Z
date_finished: null
---

# T-880: MCP tool parity — add missing since to wait, cap to spawn and dispatch

## Context

CLI commands gained `--since` (T-878) and `--cap` flags but corresponding MCP tools were not updated. Three parity gaps: WaitParams missing `since`, SpawnParams missing `cap`, DispatchParams missing `cap`.

## Acceptance Criteria

### Agent
- [x] `WaitParams` has `since: Option<u64>` field
- [x] `termlink_wait` uses `since` as initial cursor (matching CLI --since behavior)
- [x] `SpawnParams` has `cap: Option<Vec<String>>` field
- [x] `termlink_spawn` passes `cap` to register args
- [x] `DispatchParams` has `cap: Option<Vec<String>>` field
- [x] `termlink_dispatch` passes `cap` to worker register args
- [x] Unit tests for new params deserialization
- [x] `cargo test -p termlink-mcp` passes
- [x] `cargo clippy -p termlink-mcp --all-targets` passes with no warnings

## Verification

cargo test -p termlink-mcp
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

### 2026-04-05T06:35:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-880-mcp-tool-parity--add-missing-since-to-wa.md
- **Context:** Initial task creation
