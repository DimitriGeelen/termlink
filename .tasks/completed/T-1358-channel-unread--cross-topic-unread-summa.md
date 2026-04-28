---
id: T-1358
name: "channel unread — cross-topic unread summary using T-1318 cursors"
description: >
  channel unread — cross-topic unread summary using T-1318 cursors

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-28T06:58:59Z
last_update: 2026-04-28T07:12:34Z
date_finished: 2026-04-28T07:12:34Z
---

# T-1358: channel unread — cross-topic unread summary using T-1318 cursors

## Context

Cross-topic "what did I miss?" view using the T-1318 cursor system. Walks `~/.termlink/cursors.json` for entries scoped to the calling identity, queries `channel.list` for current per-topic offset counts, and reports topics where `count - 1 > cursor`.

Distinct from `channel digest` (single-topic time-windowed) and `channel dm --list --unread` (DM-only, receipt-based, not cursor-based).

## Acceptance Criteria

### Agent
- [x] `cursor_store::list_for_fingerprint(fp)` returns `Vec<(topic, cursor)>` for that identity
- [x] `channel unread` walks cursor store + channel.list, prints rows: `<topic> — N unread (latest=X, cursor=Y)`
- [x] `--json` returns `[{topic, unread, latest, cursor}]`
- [x] Empty result prints "No unread topics" (and JSON `[]`)
- [x] Topics where cursor is at-or-ahead-of latest are excluded
- [x] Pure helper `compute_unread_rows(cursors, list_entries)` unit-tested across: empty cursors, all-current, mixed, missing-from-list, cursor ahead
- [x] e2e step covering the lifecycle
- [x] cargo build, test, clippy --all-targets -- -D warnings all pass

## Verification
cargo test -p termlink --bins --quiet 2>&1 | tail -3
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

### 2026-04-28T06:58:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1358-channel-unread--cross-topic-unread-summa.md
- **Context:** Initial task creation

### 2026-04-28T07:12:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
