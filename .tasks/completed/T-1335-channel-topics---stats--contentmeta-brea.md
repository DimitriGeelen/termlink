---
id: T-1335
name: "channel topics --stats — content/meta breakdown per topic"
description: >
  channel topics --stats — content/meta breakdown per topic

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T17:48:00Z
last_update: 2026-04-27T18:02:05Z
date_finished: 2026-04-27T18:02:05Z
---

# T-1335: channel topics --stats — content/meta breakdown per topic

## Context

Add `channel topics --stats` flag: for each topic, report total envelopes split into content vs meta (receipt/reaction/redaction/edit/topic_metadata) plus distinct sender count and timestamp range. Read-only client-side aggregation reusing the existing `channel topics` walker — no new RPC needed.

## Acceptance Criteria

### Agent
- [x] `channel list --stats` prints one line per topic: `topic | content=N | meta=M | senders=S | first..last`
- [x] `--json` returns array of `{topic, content, meta, senders, first_ts, last_ts}` records
- [x] CLI unit test covers the breakdown helper (content vs meta classification matches `UNREAD_META_TYPES`) — 9 tests
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test -p termlink --bins` green (270 passed)
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean
- [x] e2e `tests/e2e/agent-conversation.sh` step 13 added and passes

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

### 2026-04-27T17:48:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1335-channel-topics---stats--contentmeta-brea.md
- **Context:** Initial task creation

### 2026-04-27T18:02:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
