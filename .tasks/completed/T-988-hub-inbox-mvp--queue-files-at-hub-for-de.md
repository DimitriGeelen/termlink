---
id: T-988
name: "Hub inbox MVP — queue files at hub for delivery when target sessions register"
description: >
  Hub inbox MVP — queue files at hub for delivery when target sessions register

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-hub/src/inbox.rs, crates/termlink-hub/src/lib.rs, crates/termlink-hub/src/router.rs, crates/termlink-hub/src/supervisor.rs]
related_tasks: []
created: 2026-04-12T21:30:08Z
last_update: 2026-04-12T21:56:28Z
date_finished: 2026-04-12T21:56:28Z
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
- [x] New hub module `inbox.rs` with spool directory management
- [x] Router intercepts SESSION_NOT_FOUND for file topics, spools to inbox
- [x] Supervisor delivers pending files to newly-online sessions each sweep cycle
- [x] File expiry: pending files older than 24h cleaned by supervisor sweep
- [x] `inbox.list` hub method: list pending files for a target
- [x] `inbox.status` hub method: show inbox overview (targets + counts)
- [x] Unit tests: deposit, list, expiry, topic detection, name sanitization (5 tests)
- [x] All existing hub tests still pass — 184/184 (179 existing + 5 inbox)

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

### 2026-04-12T21:56:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Hub inbox module complete, 184/184 tests pass, clippy clean
