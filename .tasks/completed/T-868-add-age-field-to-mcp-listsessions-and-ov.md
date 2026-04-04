---
id: T-868
name: "Add age field to MCP list_sessions and overview responses"
description: >
  Add age field to MCP list_sessions and overview responses

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-04T21:39:30Z
last_update: 2026-04-04T21:44:09Z
date_finished: 2026-04-04T21:44:09Z
---

# T-868: Add age field to MCP list_sessions and overview responses

## Context

AI agents calling `termlink_list_sessions` and `termlink_overview` get raw Unix timestamps. Adding a computed `age` field (human-readable) helps agents understand session freshness without parsing timestamps.

## Acceptance Criteria

### Agent
- [x] `SessionInfo` struct has `age` field (human-readable string like "3d", "2h")
- [x] `termlink_list_sessions` JSON output includes `age` per session
- [x] `termlink_overview` includes session age in its summary
- [x] `format_age` utility in MCP tools (mirrors CLI's version)
- [x] Existing MCP tests still pass (849 total)
- [x] Zero clippy warnings

## Verification

grep -q 'age' crates/termlink-mcp/src/tools.rs

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

### 2026-04-04T21:39:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-868-add-age-field-to-mcp-listsessions-and-ov.md
- **Context:** Initial task creation

### 2026-04-04T21:44:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
