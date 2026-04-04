---
id: T-847
name: "Add termlink_file_receive MCP tool — receive file from session event stream"
description: >
  Add termlink_file_receive MCP tool — receive file from session event stream

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T13:47:09Z
last_update: 2026-04-04T13:47:09Z
date_finished: null
---

# T-847: Add termlink_file_receive MCP tool — receive file from session event stream

## Context

Inverse of termlink_file_send. Polls a target session's event stream for the most recent completed file transfer, reassembles chunks, verifies SHA-256 integrity, and writes the result to disk. Useful for agents receiving files from dispatch workers or cross-session file exchange.

## Acceptance Criteria

### Agent
- [x] MCP tool `termlink_file_receive` added with params: target (session), output_dir (path to write file)
- [x] Tool polls event stream for latest complete file transfer (file.init + file.chunk + file.complete)
- [x] Chunks reassembled in order, SHA-256 verified against file.complete
- [x] File written to output_dir with original filename
- [x] Returns JSON with ok, filename, path, size, sha256, transfer_id
- [x] Returns error if no complete transfer found or SHA-256 mismatch
- [x] Unit test for FileReceiveParams deserialization + 2 integration tests (nonexistent session, no transfer)
- [x] tool_count incremented (40 tools)
- [x] All tests pass: cargo test --workspace (801 tests)
- [x] Zero clippy warnings: cargo clippy --workspace

## Verification

cargo test --workspace 2>&1 | tail -3
cargo clippy --workspace -- -D warnings 2>&1 | tail -3

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

### 2026-04-04T13:47:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-847-add-termlinkfilereceive-mcp-tool--receiv.md
- **Context:** Initial task creation
