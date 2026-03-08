---
id: T-018
name: "Hub message routing between sessions"
description: >
  Hub message routing between sessions

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T16:58:38Z
last_update: 2026-03-08T17:07:24Z
date_finished: 2026-03-08T17:07:24Z
---

# T-018: Hub message routing between sessions

## Context

Hub routes JSON-RPC messages between registered sessions. Discovers sessions via filesystem, forwards requests by resolving target to socket path, returns responses to caller. Implements session.discover method. Per T-003/T-005 architecture.

## Acceptance Criteria

### Agent
- [x] Router that resolves target session and forwards JSON-RPC requests via socket
- [x] `session.discover` method returns list of registered sessions
- [x] Hub accepts connections and routes messages between sessions
- [x] Error handling for unknown targets, dead sessions, forward failures
- [x] Tests for routing, discovery, and error cases
- [x] `cargo test --workspace` passes
- [x] `termlink send` CLI command to send messages through direct session connections

## Verification

PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace
PATH="$HOME/.cargo/bin:$PATH" cargo clippy --workspace -- -D warnings

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

### 2026-03-08T16:58:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-018-hub-message-routing-between-sessions.md
- **Context:** Initial task creation

### 2026-03-08T17:07:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
