---
id: T-906
name: "Add model param to dispatch"
description: >
  Add model param to dispatch

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-08T07:07:28Z
last_update: 2026-04-08T07:12:19Z
date_finished: 2026-04-08T07:12:19Z
---

# T-906: Add model param to dispatch

## Context

T-902 added task governance, T-903 added task-type routing. This task verifies model-aware dispatch is complete: optional model parameter on MCP tool and CLI, passed as TERMLINK_MODEL env var to workers, recorded in manifest.

## Acceptance Criteria

### Agent
- [x] MCP DispatchParams has model: Option<String> field
- [x] CLI dispatch has --model flag
- [x] Model passed as TERMLINK_MODEL env var in both CLI and MCP spawn paths
- [x] Model recorded in DispatchRecord manifest
- [x] Tests pass for model present, absent, and different model values
- [x] Summary report written to docs/reports/

## Verification

cargo test -p termlink-mcp dispatch_params_with_model
cargo test -p termlink-mcp dispatch_params_without_model
cargo test -p termlink-mcp dispatch_params_model_sonnet

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

### 2026-04-08T07:07:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-906-add-model-param-to-dispatch.md
- **Context:** Initial task creation

### 2026-04-08T07:12:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
