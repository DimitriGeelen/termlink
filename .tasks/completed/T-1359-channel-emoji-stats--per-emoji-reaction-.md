---
id: T-1359
name: "channel emoji-stats — per-emoji reaction breakdown across topic"
description: >
  channel emoji-stats — per-emoji reaction breakdown across topic

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-28T07:13:44Z
last_update: 2026-04-28T07:25:22Z
date_finished: 2026-04-28T07:25:22Z
---

# T-1359: channel emoji-stats — per-emoji reaction breakdown across topic

## Context

Per-topic emoji breakdown. Walks the topic, tallies every reaction (active = not redacted by m.redaction), and renders sorted-by-count rows. Each row: emoji, total count, distinct reactors, optional `--by-sender` per-reactor expansion.

Distinct from `channel digest` (shows top 3 only) and from `subscribe --reactions` (per-message aggregation, no global view).

## Acceptance Criteria

### Agent
- [x] `channel emoji-stats <topic>` renders rows: `<emoji> ×<total> (<distinct> reactor(s))` sorted by count desc
- [x] `--by-sender` adds per-reactor breakdown beneath each emoji row
- [x] `--top N` truncates to top N emojis
- [x] `--json` returns `[{emoji, count, reactors:[{sender_id, count}]}]`
- [x] Redacted reactions are excluded (use existing redaction logic)
- [x] Pure helper `compute_emoji_stats(envelopes)` unit-tested across: empty, single emoji, multiple emojis, redacted reactions, by-sender count, top-N truncation
- [x] e2e step covering full lifecycle
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

### 2026-04-28T07:13:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1359-channel-emoji-stats--per-emoji-reaction-.md
- **Context:** Initial task creation

### 2026-04-28T07:25:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
