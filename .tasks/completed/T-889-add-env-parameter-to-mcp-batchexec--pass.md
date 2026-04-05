---
id: T-889
name: "Add env parameter to MCP batch_exec — pass environment variables to batch command execution"
description: >
  Add env parameter to MCP batch_exec — pass environment variables to batch command execution

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T07:28:01Z
last_update: 2026-04-05T07:30:03Z
date_finished: 2026-04-05T07:30:03Z
---

# T-889: Add env parameter to MCP batch_exec — pass environment variables to batch command execution

## Context

All other exec/run/spawn/dispatch tools now have env support. batch_exec is the remaining gap.

## Acceptance Criteria

### Agent
- [x] `BatchExecParams` has `env: Option<HashMap<String, String>>` field
- [x] `termlink_batch_exec` passes env to each session's RPC call
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

### 2026-04-05T07:28:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-889-add-env-parameter-to-mcp-batchexec--pass.md
- **Context:** Initial task creation

### 2026-04-05T07:30:03Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
