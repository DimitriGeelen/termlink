---
id: T-832
name: "Add termlink_file_send MCP tool — file transfer between sessions for AI agents"
description: >
  Add termlink_file_send MCP tool — file transfer between sessions for AI agents

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-03T22:02:19Z
last_update: 2026-04-03T22:16:56Z
date_finished: 2026-04-03T22:16:56Z
---

# T-832: Add termlink_file_send MCP tool — file transfer between sessions for AI agents

## Context

AI agents need to transfer files between sessions. The file transfer protocol uses 3 phases (init, chunks, complete) over event.emit. This MCP tool wraps the entire pipeline into a single call. 34th MCP tool.

## Acceptance Criteria

### Agent
- [x] `termlink_file_send` tool added to tools.rs with FileSendParams (target, path)
- [x] Reads file, computes SHA256, chunks and sends via init/chunk/complete events
- [x] Integration test for sending a file to a live session
- [x] Integration test for nonexistent target error (+ nonexistent file test)
- [x] All tests pass, zero clippy warnings
- [x] ARCHITECTURE.md and CHANGELOG.md updated with 34 MCP tools

## Verification

cargo test --workspace 2>&1 | tail -5
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c '^warning\[')" = "0"
grep -q "termlink_file_send" crates/termlink-mcp/src/tools.rs
grep -q "file_send" crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-04-03T22:02:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-832-add-termlinkfilesend-mcp-tool--file-tran.md
- **Context:** Initial task creation

### 2026-04-03T22:16:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
