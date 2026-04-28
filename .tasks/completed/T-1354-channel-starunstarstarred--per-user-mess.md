---
id: T-1354
name: "channel star/unstar/starred — per-user message bookmarks (Matrix m.bookmark)"
description: >
  channel star/unstar/starred — per-user message bookmarks (Matrix m.bookmark)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-28T06:05:59Z
last_update: 2026-04-28T06:24:54Z
date_finished: 2026-04-28T06:24:54Z
---

# T-1354: channel star/unstar/starred — per-user message bookmarks (Matrix m.bookmark)

## Context

Per-user message bookmarks. Sibling of T-1345 pin (per-topic, anyone-can-pin) but scoped to the calling user. Matrix `m.bookmark`/favorite analog. Uses additive metadata pattern: `metadata={star_target: <offset>, star: <true|false>}`. `starred` aggregates by walking topic and computing the latest state per (sender_id, target).

## Acceptance Criteria

### Agent
- [x] `termlink channel star <topic> --offset N` emits an envelope with `metadata.star_target=N`, `metadata.star=true`
- [x] `termlink channel unstar <topic> --offset N` emits same shape but `metadata.star=false`
- [x] `termlink channel starred <topic>` lists current user's starred messages (latest state per target wins)
- [x] `--all` flag on `starred` lists every user's stars (not just caller)
- [x] `--json` flag on `starred` returns structured rows
- [x] Unit tests for `compute_starred_set` covering: empty input, single star, star→unstar, unstar without prior star (ignored), multiple users, --all aggregation
- [x] e2e step added to tests/e2e/agent-conversation.sh
- [x] cargo build, cargo test, cargo clippy --all-targets -- -D warnings all pass

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

### 2026-04-28T06:05:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1354-channel-starunstarstarred--per-user-mess.md
- **Context:** Initial task creation

### 2026-04-28T06:24:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
