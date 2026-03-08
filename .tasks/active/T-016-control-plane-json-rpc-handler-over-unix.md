---
id: T-016
name: "Control plane JSON-RPC handler over Unix socket"
description: >
  Control plane JSON-RPC handler over Unix socket

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T16:32:19Z
last_update: 2026-03-08T16:32:19Z
date_finished: null
---

# T-016: Control plane JSON-RPC handler over Unix socket

## Context

Implements JSON-RPC 2.0 control plane over Unix sockets per T-005 protocol spec. Sessions accept connections and dispatch methods: `termlink.ping`, `session.heartbeat`, `query.status`, `query.capabilities`. See `docs/reports/T-005-message-protocol-design.md`.

## Acceptance Criteria

### Agent
- [ ] JSON-RPC 2.0 request/response/error types with serde serialization
- [ ] Method dispatcher that routes incoming JSON-RPC requests to handlers
- [ ] `termlink.ping` handler returns session ID (for liveness verification)
- [ ] `query.status` handler returns session state and metadata
- [ ] `query.capabilities` handler returns session capabilities
- [ ] Session accept loop that reads newline-delimited JSON-RPC from Unix socket
- [ ] Tests for JSON-RPC serialization, method dispatch, and end-to-end socket communication
- [ ] `cargo test --workspace` passes with no failures

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
