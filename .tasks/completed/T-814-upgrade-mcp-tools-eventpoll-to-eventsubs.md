---
id: T-814
name: "Upgrade MCP tools event.poll to event.subscribe"
description: >
  Upgrade MCP tools event.poll to event.subscribe

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-03-30T19:39:46Z
last_update: 2026-03-30T19:47:19Z
date_finished: 2026-03-30T19:47:19Z
---

# T-814: Upgrade MCP tools event.poll to event.subscribe

## Context

MCP tools `termlink_request` and `termlink_wait` use `event.poll` + sleep(500ms) loops. Upgrade to `event.subscribe` for near-zero latency, consistent with CLI commands (T-811, T-812, T-813). Keep `termlink_event_poll` as-is (it wraps the poll RPC intentionally).

## Acceptance Criteria

### Agent
- [x] `termlink_request` cursor snapshot uses `event.subscribe` with timeout_ms=1
- [x] `termlink_request` reply poll loop uses `event.subscribe` instead of poll+sleep
- [x] `termlink_wait` poll loop uses `event.subscribe` instead of poll+sleep
- [x] `cargo check -p termlink-mcp` passes
- [x] All MCP tests pass (43/43)

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

### 2026-03-30T19:39:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-814-upgrade-mcp-tools-eventpoll-to-eventsubs.md
- **Context:** Initial task creation

### 2026-03-30T19:47:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
