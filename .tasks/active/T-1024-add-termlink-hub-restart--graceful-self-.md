---
id: T-1024
name: "Add termlink hub restart — graceful self-restart via fork-exec"
description: >
  Add 'termlink hub restart' command and hub.restart RPC method. The hub spawns a new process from the current binary, waits for it to bind the port, then exits. This enables remote hub upgrades via termlink without losing connectivity (solves the chicken-and-egg problem from T-1023).

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T12:47:11Z
last_update: 2026-04-15T13:47:08Z
date_finished: 2026-04-13T12:51:59Z
---

# T-1024: Add termlink hub restart — graceful self-restart via fork-exec

## Context

Killing a hub via `kill PID` or `hub.shutdown` RPC severs all client connections instantly. For remote deployment via termlink, we need the hub to restart itself: spawn a new process from the current binary, let it bind the port, then exit. This enables `termlink remote exec hub restart` without losing connectivity permanently.

Approach: add a `hub.restart` RPC method that spawns `termlink hub start` as a child process, then shuts down the current process after a brief delay. The new process binds the same port (after the old one releases it).

## Acceptance Criteria

### Agent
- [x] `termlink hub restart` CLI subcommand added
- [x] Hub start records TCP address to hub.tcp for restart discovery
- [x] New process inherits runtime dir, TCP address, and secret
- [x] CLI prints restart status (spawned PID, waiting for port)
- [x] Builds and passes clippy

### Human
- [ ] [REVIEW] Test hub restart locally
  **Steps:**
  1. `cd /opt/termlink && termlink hub start &`
  2. `cd /opt/termlink && termlink hub restart`
  3. `cd /opt/termlink && termlink ping`
  **Expected:** Hub restarts, ping succeeds with new PID
  **If not:** Check hub.restart RPC handler in router.rs

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink-hub -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"
cargo clippy -p termlink -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"
cargo clippy -p termlink-hub -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"

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

### 2026-04-13T12:47:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1024-add-termlink-hub-restart--graceful-self-.md
- **Context:** Initial task creation

### 2026-04-13T12:51:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
