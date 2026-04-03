---
id: T-824
name: "Add termlink_collect MCP tool — multi-session event fan-in via hub"
description: >
  Add termlink_collect MCP tool — multi-session event fan-in via hub

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-03T20:10:08Z
last_update: 2026-04-03T20:10:08Z
date_finished: null
---

# T-824: Add termlink_collect MCP tool — multi-session event fan-in via hub

## Context

Add `termlink_collect` MCP tool — fan-in events from multiple sessions via hub. Single-shot collect with timeout for MCP compatibility. Requires hub running. This is the 31st MCP tool.

## Acceptance Criteria

### Agent
- [ ] `termlink_collect` MCP tool implemented — calls hub `event.collect` with targets, topic, timeout_ms
- [ ] Params: targets (optional list), topic (optional filter), timeout_ms (optional, default 5000), since (optional cursors)
- [ ] Returns: collected events array with session_name, topic, payload, seq, timestamp + cursors for continuation
- [ ] Returns error message when hub is not running
- [ ] Integration test: collect with no hub returns error
- [ ] `cargo test -p termlink-mcp` passes
- [ ] `cargo clippy --workspace` clean

## Verification

cargo test -p termlink-mcp
cargo clippy --workspace
grep -q 'termlink_collect' crates/termlink-mcp/src/tools.rs

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

### 2026-04-03T20:10:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-824-add-termlinkcollect-mcp-tool--multi-sess.md
- **Context:** Initial task creation
