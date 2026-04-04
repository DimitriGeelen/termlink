---
id: T-877
name: "Add since_default to MCP termlink_collect tool — expose hub history replay to MCP consumers"
description: >
  Add since_default to MCP termlink_collect tool — expose hub history replay to MCP consumers

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-04T23:17:24Z
last_update: 2026-04-04T23:20:25Z
date_finished: 2026-04-04T23:20:25Z
---

# T-877: Add since_default to MCP termlink_collect tool — expose hub history replay to MCP consumers

## Context

T-876 added `since_default` to the hub's event.collect RPC. Expose it in the MCP
`termlink_collect` tool so AI agents can replay event history without knowing session IDs.

## Acceptance Criteria

### Agent
- [x] `since_default` field added to `CollectParams` struct
- [x] MCP tool passes `since_default` to hub RPC when provided
- [x] Unit test for CollectParams with since_default (858 tests total)
- [x] `cargo clippy --workspace` passes with no warnings
- [x] `cargo test --workspace` passes (858 tests, 0 failures)

## Verification

cargo clippy --workspace 2>&1 | grep -v "^$" | tail -5 | grep -q "warning generated\|could not compile" && exit 1 || true
cargo test --workspace 2>&1 | tail -3 | grep -q "0 failed"

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

### 2026-04-04T23:17:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-877-add-sincedefault-to-mcp-termlinkcollect-.md
- **Context:** Initial task creation

### 2026-04-04T23:20:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
