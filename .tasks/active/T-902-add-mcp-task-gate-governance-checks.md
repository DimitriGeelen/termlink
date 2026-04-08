---
id: T-902
name: Add MCP task-gate governance checks
description: >
  Add optional task_id parameter to termlink_exec, termlink_spawn, termlink_dispatch,
  and termlink_interact MCP tools. When TERMLINK_TASK_GOVERNANCE=1, tools without
  task_id return a structured MCP error. Default: no enforcement (backward compatible).
status: started-work
workflow_type: build
owner: agent
priority: medium
tags: [mcp, governance]
agents:
  primary: coder
  supporting: []
created: 2026-04-08T07:50:00Z
last_update: 2026-04-08T07:50:00Z
date_finished: null
---

# T-902: Add MCP task-gate governance checks

## Acceptance Criteria

### Agent
- [x] `task_id: Option<String>` added to ExecParams, SpawnParams, InteractParams, DispatchParams
- [x] Governance check function reads `TERMLINK_TASK_GOVERNANCE` env var
- [x] When governance enabled and task_id missing: returns structured JSON error
- [x] When task_id provided: passed through to session tags on spawn/dispatch
- [x] Default behavior: no enforcement (backward compatible)
- [x] Existing tests pass (`cargo test`)
- [x] New tests cover: with task_id passes, without task_id in strict mode errors, default allows all
- [x] Param deserialization tests for task_id field

## Verification

cargo test --manifest-path /opt/termlink/Cargo.toml -p termlink-mcp 2>&1 | tail -5

## Design Record

Governance gate is implemented as a standalone function `check_task_governance(task_id: &Option<String>, tool_name: &str) -> Result<(), String>`
that returns Ok(()) when allowed, Err(json_error_string) when blocked.

The function checks:
1. Is `TERMLINK_TASK_GOVERNANCE` set to "1"? If not, allow everything.
2. Is task_id provided? If yes, allow. If no, return structured error.

Task ID is passed through to session tags as `task:<task_id>` on spawn and dispatch.

## Updates

- 2026-04-08: Implementation started
