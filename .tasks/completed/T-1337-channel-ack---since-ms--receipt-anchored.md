---
id: T-1337
name: "channel ack --since <ms> — receipt anchored to recent slice"
description: >
  channel ack --since <ms> — receipt anchored to recent slice

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T18:18:37Z
last_update: 2026-04-27T18:27:33Z
date_finished: 2026-04-27T18:27:33Z
---

# T-1337: channel ack --since <ms> — receipt anchored to recent slice

## Context

`channel ack --since <ms>` — alternative way to compute the receipt's `up_to`. Walks the topic, finds the highest offset whose envelope has `ts >= <ms>`, posts an `m.receipt` for it. Errors with hint when no envelope satisfies. Mutually exclusive with `--up-to`. Useful for "I just reviewed the last 10 minutes of activity — mark it acked." Reuses existing channel.subscribe walker; no new RPC.

## Acceptance Criteria

### Agent
- [x] `channel ack <topic> --since <ms>` resolves `up_to` to the highest offset with `ts >= <ms>` and posts a receipt
- [x] `--since` and `--up-to` are mutually exclusive (clap-enforced — `error: ... cannot be used with`)
- [x] Errors with friendly message when no envelope on the topic has `ts >= <ms>` (hint includes the topic's latest ts and the gap in ms)
- [x] Pure helpers `latest_offset_since` (5 tests) and `max_ts` (1 test) unit-tested
- [x] Build/test/clippy all clean (287 tests passing); e2e `agent-conversation.sh` step 15 added and passes

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

### 2026-04-27T18:18:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1337-channel-ack---since-ms--receipt-anchored.md
- **Context:** Initial task creation

### 2026-04-27T18:27:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
