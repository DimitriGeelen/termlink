---
id: T-886
name: "Add env parameter to MCP termlink_exec — pass environment variables to remote command execution"
description: >
  Add env parameter to MCP termlink_exec — pass environment variables to remote command execution

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T07:18:23Z
last_update: 2026-04-05T07:18:23Z
date_finished: null
---

# T-886: Add env parameter to MCP termlink_exec — pass environment variables to remote command execution

## Context

Session handler already supports env in command.execute RPC but MCP ExecParams doesn't expose it.

## Acceptance Criteria

### Agent
- [x] `ExecParams` has `env: Option<HashMap<String, String>>` field
- [x] `termlink_exec` passes env to RPC params
- [x] `cargo build` succeeds

## Verification

cargo build

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

### 2026-04-05T07:18:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-886-add-env-parameter-to-mcp-termlinkexec--p.md
- **Context:** Initial task creation
