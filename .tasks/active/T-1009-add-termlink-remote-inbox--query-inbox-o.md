---
id: T-1009
name: "Add termlink remote inbox — query inbox on remote hubs via RPC"
description: >
  Add termlink remote inbox — query inbox on remote hubs via RPC

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-13T09:34:58Z
last_update: 2026-04-13T09:43:18Z
date_finished: 2026-04-13T09:43:18Z
---

# T-1009: Add termlink remote inbox — query inbox on remote hubs via RPC

## Context

Add `termlink remote inbox` subcommand (status/list/clear) that queries inbox on a remote hub via RPC — works regardless of the remote termlink binary version.

## Acceptance Criteria

### Agent
- [x] Add RemoteInboxAction enum with Status, List, Clear variants
- [x] Wire Inbox into RemoteAction enum in cli.rs
- [x] Add cmd_remote_inbox (status/list/clear) in remote.rs
- [x] All three subcommands call the hub's inbox.status/inbox.list/inbox.clear RPC
- [x] cargo clippy --workspace passes (0 warnings)
- [x] cargo test --workspace passes (1003 tests)
- [x] `termlink remote inbox ring20-management status` connects and calls RPC (remote hub needs inbox routes for full E2E)

### Human
- [ ] [RUBBER-STAMP] Verify `termlink remote inbox --help` shows all three subcommands
  **Steps:**
  1. `cd /opt/termlink && cargo run -- remote inbox --help`
  **Expected:** Shows status, list, clear subcommands
  **If not:** Check RemoteInboxAction enum wiring

## Verification

cargo clippy --workspace -- -D warnings 2>&1 | tail -1
cargo test --workspace 2>&1 | grep "^test result" | grep -v "0 passed"

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

### 2026-04-13T09:34:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1009-add-termlink-remote-inbox--query-inbox-o.md
- **Context:** Initial task creation

### 2026-04-13T09:43:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
