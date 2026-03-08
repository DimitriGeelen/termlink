---
id: T-016
name: "Control plane JSON-RPC handler over Unix socket"
description: >
  Control plane JSON-RPC handler over Unix socket

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T16:32:19Z
last_update: 2026-03-08T16:35:59Z
date_finished: 2026-03-08T16:35:59Z
---

# T-016: Control plane JSON-RPC handler over Unix socket

## Context

Implements JSON-RPC 2.0 control plane over Unix sockets per T-005 protocol spec. Sessions accept connections and dispatch methods: `termlink.ping`, `session.heartbeat`, `query.status`, `query.capabilities`. See `docs/reports/T-005-message-protocol-design.md`.

## Acceptance Criteria

### Agent
- [x] JSON-RPC 2.0 request/response/error types with serde serialization
- [x] Method dispatcher that routes incoming JSON-RPC requests to handlers
- [x] `termlink.ping` handler returns session ID (for liveness verification)
- [x] `query.status` handler returns session state and metadata
- [x] `query.capabilities` handler returns session capabilities
- [x] Session accept loop that reads newline-delimited JSON-RPC from Unix socket
- [x] Tests for JSON-RPC serialization, method dispatch, and end-to-end socket communication
- [x] `cargo test --workspace` passes with no failures

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

### 2026-03-08T16:32:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-016-control-plane-json-rpc-handler-over-unix.md
- **Context:** Initial task creation

### 2026-03-08T16:35:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
