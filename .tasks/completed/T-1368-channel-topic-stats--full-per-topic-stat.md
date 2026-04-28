---
id: T-1368
name: "channel topic-stats — full per-topic statistics dashboard"
description: >
  Add `channel topic-stats <topic>` — a single-shot statistics dashboard
  for one topic. Counts: total envelopes, distinct senders, breakdown by
  msg_type, top-5 senders, distinct + top-5 emojis (reactions), thread
  roots count, active pins, forwards-in, edits, redactions, time span.
  Like `channel digest` (T-1356) but unconstrained by time and focused on
  cumulative totals rather than recent activity.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [agent-conversation, matrix, stats, channel-cli]
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: [T-1356, T-1359, T-1365]
created: 2026-04-28T10:13:00Z
last_update: 2026-04-28T10:32:02Z
date_finished: 2026-04-28T10:32:02Z
---

# T-1368: channel topic-stats — full per-topic statistics dashboard

## Context

`channel digest` (T-1356) shows recent activity. `channel emoji-stats` (T-1359)
shows reaction breakdown. `channel members` shows distinct senders. There's no
single command that gives a full overview of a topic's lifetime activity.
`topic-stats` rolls up the most useful counters into one read.

Pure helper `compute_topic_stats(envelopes) -> TopicStats` so unit tests
exercise aggregation deterministically. Honors redaction (redacted envelopes
excluded from all counters).

## Acceptance Criteria

### Agent
- [x] CLI variant `Channel TopicStats <topic>` accepted; `--hub`, `--json` flags wired
- [x] `compute_full_topic_stats` (pure helper, name disambiguated from existing T-1335 `compute_topic_stats`) added with 7 unit tests: empty topic, single post, mixed msg types, redacted excluded, active pins LWW, top senders tiebreak, forwards-in via metadata
- [x] Live smoke test against the local hub produces a sensible report (4 envelopes including pin + reaction render correctly)
- [x] e2e step 41 added (positive + JSON shape)
- [x] `cargo build --release -p termlink && cargo test -p termlink --bins --quiet && cargo clippy --all-targets --workspace -- -D warnings` all green

## Updates

### 2026-04-28T10:30Z — name collision avoided
- T-1335 already shipped `TopicStats` + `compute_topic_stats` (a lightweight content/meta/sender/ts breakdown used by `channel list`). To avoid build-time E0428 collision while preserving the existing helper, this work uses `FullTopicStats` + `compute_full_topic_stats` for the dashboard shape. Doc-comment cross-references the two so future readers see the distinction.

## Verification

cargo test -p termlink --bins --quiet 2>&1 | tail -3
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -3

## Decisions

## Updates

### 2026-04-28T10:13:00Z — task scoped
- ACs filled before any source-file edit (G-020 build-readiness gate).

### 2026-04-28T10:32:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
