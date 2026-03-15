---
id: T-145
name: "Hub TCP listener dual select"
description: >
  Hub TCP dual-listen via --tcp flag

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [tcp, hub]
components: []
related_tasks: []
created: 2026-03-15T22:05:51Z
last_update: 2026-03-15T22:49:31Z
date_finished: 2026-03-15T22:49:31Z
---

# T-145: Hub TCP listener dual select

## Context

Add opt-in TCP listening to the hub via `--tcp` flag. See T-144 inception and
docs/reports/T-144-tcp-hub-inception.md for design rationale.

## Acceptance Criteria

### Agent
- [x] `HubAction::Start` has optional `--tcp` arg (e.g., `0.0.0.0:9100`)
- [x] `server::run_with_tcp()` accepts `Option<&str>` for TCP address
- [x] Hub binds TCP listener when `--tcp` provided, Unix-only otherwise
- [x] `handle_connection` is generic (works with both Unix and TCP streams)
- [x] `run_accept_loop` uses `tokio::select!` over both listeners
- [x] TCP connections log info about LAN-only (no auth)
- [x] All existing tests pass (257 total)
- [x] New test: `hub_dual_listen_unix_and_tcp` verifies both transports work

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace

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

### 2026-03-15T22:05:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-145-hub-tcp-listener-dual-select.md
- **Context:** Initial task creation

### 2026-03-15T22:07:08Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-15T22:49:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
