---
id: T-1338
name: "channel dm --list --unread — DM inbox with per-topic unread counts"
description: >
  channel dm --list --unread — DM inbox with per-topic unread counts

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T18:31:24Z
last_update: 2026-04-27T18:39:54Z
date_finished: 2026-04-27T18:39:54Z
---

# T-1338: channel dm --list --unread — DM inbox with per-topic unread counts

## Context

Compose T-1320 (`channel dm --list`) with T-1332 (`channel unread`) into an "inbox" view. Adds `--unread` flag to `channel dm --list`: for each DM topic the caller participates in, fetches the caller's last receipt and counts content envelopes since it. Output adds `unread=N first=offset` columns. JSON adds `unread` and `first_unread` fields per record. Sorts unread-first by default. Read-only client-side aggregation.

## Acceptance Criteria

### Agent
- [x] `channel dm --list --unread` adds an `unread=N first=offset` column suffix per DM line
- [x] DMs with `unread > 0` sort before zero-unread DMs (stable within each group)
- [x] `--json` adds `unread` (u64) and `first_unread` (Option<u64>) fields per record
- [x] Pure helper `sort_dm_inbox(rows)` + `DmInboxRow::to_json` unit-tested (4 tests)
- [x] Build/test/clippy all clean (291 tests passing); e2e `agent-conversation.sh` step 16 added and passes

## Verification

cargo build --release -p termlink
cargo test -p termlink --bins --quiet
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5

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

### 2026-04-27T18:31:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1338-channel-dm---list---unread--dm-inbox-wit.md
- **Context:** Initial task creation

### 2026-04-27T18:39:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
