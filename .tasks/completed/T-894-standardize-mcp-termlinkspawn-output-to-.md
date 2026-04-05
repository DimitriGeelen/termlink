---
id: T-894
name: "Standardize MCP termlink_spawn output to structured JSON — consistent with other tools"
description: >
  Standardize MCP termlink_spawn output to structured JSON — consistent with other tools

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T08:07:50Z
last_update: 2026-04-05T08:14:23Z
date_finished: 2026-04-05T08:14:23Z
---

# T-894: Standardize MCP termlink_spawn output to structured JSON — consistent with other tools

## Context

`termlink_spawn` returns plain text strings like `"Spawned session 'xxx' (ready)"` while most other tools return structured JSON with `ok` field. Standardizing for consistency.

## Acceptance Criteria

### Agent
- [x] Spawn returns JSON with ok, session_name, status fields
- [x] Spawn failure returns JSON error via json_err()
- [x] kv_set, kv_get, kv_del return structured JSON with ok field
- [x] Integration test updated for new JSON output format
- [x] All tests pass, zero clippy warnings

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

### 2026-04-05T08:07:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-894-standardize-mcp-termlinkspawn-output-to-.md
- **Context:** Initial task creation

### 2026-04-05T08:14:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
