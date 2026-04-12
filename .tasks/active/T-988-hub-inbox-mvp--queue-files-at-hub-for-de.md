---
id: T-988
name: "Hub inbox MVP — queue files at hub for delivery when target sessions register"
description: >
  Hub inbox MVP — queue files at hub for delivery when target sessions register

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T21:30:08Z
last_update: 2026-04-12T21:30:08Z
date_finished: null
---

# T-988: Hub inbox MVP — queue files at hub for delivery when target sessions register

## Context

Build task from T-946 GO decision. Currently `send-file` forwards chunks via RPC directly
to the target session. If the target is offline, the transfer fails silently. The hub inbox
stores pending file transfers in a spool directory and delivers them when the target session
registers. MVP: spool storage + delivery-on-register + expiry.

Flow: sender → hub `file.deposit` (new method) → spool to `runtime_dir()/inbox/{target}/` →
target registers → hub delivers via `event.emit` → cleanup on delivery.

## Acceptance Criteria

### Agent
- [ ] New hub module `inbox.rs` with `Inbox` struct managing spool directory
- [ ] `file.deposit` hub method: accepts target session name/ID + file events, spools to disk
- [ ] On session registration, hub checks inbox for pending files and delivers
- [ ] File expiry: pending files older than 24h are cleaned by supervisor sweep
- [ ] `inbox.list` hub method: list pending files for a target (useful for diagnostics)
- [ ] `inbox.status` hub method: show inbox size/count
- [ ] Unit tests: deposit, deliver-on-register, expiry, list
- [ ] All existing hub tests still pass

## Verification

cargo test -p termlink-hub
cargo clippy -p termlink-hub -- -D warnings

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

### 2026-04-12T21:30:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-988-hub-inbox-mvp--queue-files-at-hub-for-de.md
- **Context:** Initial task creation
