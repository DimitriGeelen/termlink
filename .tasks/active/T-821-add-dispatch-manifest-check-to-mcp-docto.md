---
id: T-821
name: "Add dispatch manifest check to MCP doctor tool"
description: >
  Add dispatch manifest check to MCP doctor tool

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-30T20:22:32Z
last_update: 2026-03-30T20:22:32Z
date_finished: null
---

# T-821: Add dispatch manifest check to MCP doctor tool

## Context

CLI doctor has dispatch manifest check (#6, added T-806) but MCP doctor is missing it. Add parity.

## Acceptance Criteria

### Agent
- [x] MCP doctor includes dispatch manifest check
- [x] `cargo check -p termlink-mcp` passes
- [x] MCP tests pass (43/43)

## Verification

cargo check -p termlink-mcp 2>&1 | grep -q "Finished"
cargo test -p termlink-mcp

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

### 2026-03-30T20:22:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-821-add-dispatch-manifest-check-to-mcp-docto.md
- **Context:** Initial task creation
