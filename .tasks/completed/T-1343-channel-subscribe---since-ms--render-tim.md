---
id: T-1343
name: "channel subscribe --since <ms> — render-time timestamp filter"
description: >
  channel subscribe --since <ms> — render-time timestamp filter

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T19:22:32Z
last_update: 2026-04-27T19:31:15Z
date_finished: 2026-04-27T19:31:15Z
---

# T-1343: channel subscribe --since <ms> — render-time timestamp filter

## Context

`channel subscribe --since <ms>` — render-time filter that drops envelopes whose `ts < <ms>` from the printed output. Mirrors `channel info --since` (T-1331) and `channel ack --since` (T-1337). Pure render-side filter applied per page; cursor / pagination unaffected. Useful for "what's happened on this topic since 5 minutes ago" without needing to know the offset.

## Acceptance Criteria

### Agent
- [x] `channel subscribe <topic> --since <ms>` only prints envelopes whose ts >= `<ms>`
- [x] Pre-since envelopes are dropped from BOTH text and JSON-lines output (render-side filter only — pagination unchanged)
- [x] Pure helper `should_emit_for_since(env, since)` unit-tested across present/absent ts, ts/ts_unix_ms aliasing, and >=  boundary
- [x] Build/test/clippy all clean (307 tests passing); e2e `agent-conversation.sh` step 20 added and passes

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

### 2026-04-27T19:22:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1343-channel-subscribe---since-ms--render-tim.md
- **Context:** Initial task creation

### 2026-04-27T19:31:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
