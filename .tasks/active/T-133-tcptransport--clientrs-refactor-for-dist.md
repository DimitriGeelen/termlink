---
id: T-133
name: "TcpTransport + client.rs refactor for distributed sessions"
description: >
  TcpTransport + client.rs refactor for distributed sessions

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-14T15:18:46Z
last_update: 2026-03-14T15:33:30Z
date_finished: 2026-03-14T15:33:13Z
---

# T-133: TcpTransport + client.rs refactor for distributed sessions

## Context

Phase 1 from T-011 inception GO. Implement TcpTransport adapter following UnixTransport pattern, refactor client.rs to accept TransportAddr, update hub router. See docs/reports/T-011-distributed-topology-inception.md.

## Acceptance Criteria

### Agent
- [x] `TcpTransport` struct implements `Transport` trait (connect + bind)
- [x] `TcpLivenessProbe` implements `LivenessProbe` (TCP connect with timeout)
- [x] `client::Client::connect_addr` accepts `&TransportAddr` (+ backward-compat `connect(&Path)`)
- [x] `client::rpc_call_addr` accepts `&TransportAddr` (+ backward-compat `rpc_call(&Path)`)
- [x] Hub router uses `reg.addr.to_transport_addr()` instead of `reg.socket_path()` for routing
- [x] All existing tests pass (249 total)
- [x] New tests for TcpTransport (connect, bind, liveness probe) — 4 new tests

### Human
- [ ] [REVIEW] Test TCP session registration + ping across localhost
  **Steps:**
  1. `termlink register --name tcp-test --addr tcp://127.0.0.1:9000`
  2. `termlink ping tcp-test`
  **Expected:** PONG response
  **If not:** Check if TCP listener started, verify port not in use

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | grep -q "test result: ok"
/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | grep -qv "^error"

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

### 2026-03-14T15:18:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-133-tcptransport--clientrs-refactor-for-dist.md
- **Context:** Initial task creation

### 2026-03-14T15:33:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
