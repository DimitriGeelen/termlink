---
id: T-1020
name: "Add termlink remote doctor — health check remote hubs via RPC"
description: >
  Add 'termlink remote doctor' command that runs health checks on a remote hub via termlink RPC, without SSH. Currently you must SSH or remote exec to check hub health. A dedicated remote doctor command provides a cleaner UX and can be automated.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T12:16:11Z
last_update: 2026-04-15T13:47:08Z
date_finished: 2026-04-13T12:19:55Z
---

# T-1020: Add termlink remote doctor — health check remote hubs via RPC

## Context

Adds `termlink remote doctor <hub>` which queries a remote hub via RPC to report health status: connectivity, session count, inbox status, hub version. Uses hub.ping, session.list, inbox.status, and remote exec for version.

## Acceptance Criteria

### Agent
- [x] `RemoteAction::Doctor` variant added to CLI
- [x] `cmd_remote_doctor` implemented using hub RPC calls (connect, session.list, inbox.status)
- [x] JSON output supported via `--json` flag
- [x] Human-readable output shows pass/warn/fail checks
- [x] Builds clean with no clippy warnings

### Human
- [ ] [REVIEW] Test against live hubs
  **Steps:**
  1. `cd /opt/termlink && cargo run -- remote doctor ring20-management`
  2. `cd /opt/termlink && cargo run -- remote doctor ring20-dashboard --json`
  **Expected:** Health check output showing connectivity, sessions, inbox
  **If not:** Check cmd_remote_doctor implementation in remote.rs

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
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

### 2026-04-13T12:16:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1020-add-termlink-remote-doctor--health-check.md
- **Context:** Initial task creation

### 2026-04-13T12:19:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T23:07:17Z — e2e-evidence [T-1097]
- **Evidence:** termlink remote doctor local-test --json produces structured health report (connectivity, sessions, inbox, version)
- **Verified by:** termlink remote doctor local-test
