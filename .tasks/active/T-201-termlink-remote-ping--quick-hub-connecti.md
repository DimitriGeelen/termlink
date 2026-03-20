---
id: T-201
name: "termlink remote ping — quick hub connectivity check"
description: >
  termlink remote ping — quick hub connectivity check

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [cli, cross-machine, remote]
components: []
related_tasks: []
created: 2026-03-20T23:49:02Z
last_update: 2026-03-20T23:52:48Z
date_finished: 2026-03-20T23:52:48Z
---

# T-201: termlink remote ping — quick hub connectivity check

## Context

Quick connectivity/health check for remote hubs and sessions. `remote ping <hub>` tests
TOFU TLS + auth. `remote ping <hub> <session>` forwards ping to a specific session via hub routing.

## Acceptance Criteria

### Agent
- [x] `RemoteAction::Ping` variant with args: hub, optional session, secret-file/secret, scope
- [x] Hub-only ping: connects, authenticates, calls `session.discover` as health check, reports latency
- [x] Session ping: forwards `termlink.ping` with `target` param, reports PONG with latency
- [x] Latency measurement: reports total, auth, and rpc round-trip time in ms
- [x] `cargo build --package termlink` compiles clean
- [x] `cargo test --package termlink` passes (297 passed, 0 failed)
- [x] Help text: `termlink remote ping --help`

### Human
- [ ] [REVIEW] Cross-machine test: ping hub and session on .107
  **Steps:**
  1. `termlink remote ping 192.168.10.107:9100 --secret-file /tmp/termlink-107-secret.txt`
  2. `termlink remote ping 192.168.10.107:9100 fw-agent --secret-file /tmp/termlink-107-secret.txt`
  **Expected:** Both report PONG with latency in ms
  **If not:** Check hub is running, verify auth

## Verification

/Users/dimidev32/.cargo/bin/cargo build --package termlink
grep -q "RemoteAction::Ping" crates/termlink-cli/src/main.rs

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

### 2026-03-20T23:49:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-201-termlink-remote-ping--quick-hub-connecti.md
- **Context:** Initial task creation

### 2026-03-20T23:52:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
