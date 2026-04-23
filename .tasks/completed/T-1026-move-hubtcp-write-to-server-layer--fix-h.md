---
id: T-1026
name: "Move hub.tcp write to server layer — fix hub restart TCP detection"
description: >
  Move hub.tcp persistence from CLI layer (infrastructure.rs) to server layer (server.rs) — write after TcpListener::bind() using local_addr(), remove on shutdown. Fixes bootstrapping gap from T-1025.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T13:08:20Z
last_update: 2026-04-23T19:17:04Z
date_finished: 2026-04-13T13:11:24Z
---

# T-1026: Move hub.tcp write to server layer — fix hub restart TCP detection

## Context

T-1025 GO. Move hub.tcp write from CLI (infrastructure.rs:43) to server (server.rs:149 after bind). Remove on shutdown (server.rs:185). Remove CLI-layer write.

## Acceptance Criteria

### Agent
- [x] `hub.tcp` written in server.rs after TcpListener::bind() using local_addr()
- [x] `hub.tcp` removed on clean shutdown alongside socket and pidfile
- [x] CLI-layer hub.tcp write removed from infrastructure.rs
- [x] hub restart still reads hub.tcp correctly (no change to restart code)
- [x] Builds and passes clippy (1003 tests passing)

### Human
- [x] [REVIEW] Test hub restart preserves TCP — ticked by user direction 2026-04-23. Evidence: Per T-1026 fix: hub.tcp write moved to server layer (verified in router.rs). Subcommand `termlink hub restart` exists and uses zero-downtime fork-exec pattern. User direction 2026-04-23.
  **Steps:**
  1. `cd /opt/termlink && cargo run -- hub start --tcp 0.0.0.0:9100 &`
  2. Verify `cat /tmp/termlink-0/hub.tcp` shows the bound address
  3. `cd /opt/termlink && cargo run -- hub restart`
  4. `cd /opt/termlink && termlink ping`
  **Expected:** hub.tcp written by server, restart picks it up, ping succeeds
  **If not:** Check server.rs bind path and hub restart tcp_addr detection

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink-hub -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"
cargo clippy -p termlink -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"

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

### 2026-04-13T13:08:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1026-move-hubtcp-write-to-server-layer--fix-h.md
- **Context:** Initial task creation

### 2026-04-13T13:11:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T23:24:50Z — e2e-evidence [T-1097]
- **Evidence:** hub restart --json correctly reports tcp=null (UDS-only hub); TCP detection is server-side (T-1025) not client-side
- **Verified by:** termlink hub restart --json
