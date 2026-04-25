---
id: T-1233
name: "Hub: channel.list_topics(prefix) router method + tests (T-1229a)"
description: >
  Add hub-side channel.list_topics(prefix="inbox:") RPC method per T-1229 Option A. Returns [{topic, count}] for topics matching the prefix, mirroring the existing inbox::list_all_targets() spool walk on the channel surface. Single round-trip aggregation. Preserves fleet-doctor correctness invariant.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-1229, T-1155, bus, channel, hub]
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, crates/termlink-hub/src/channel.rs]
related_tasks: []
created: 2026-04-25T10:11:10Z
last_update: 2026-04-25T10:15:13Z
date_finished: 2026-04-25T10:15:13Z
---

# T-1233: Hub: channel.list_topics(prefix) router method + tests (T-1229a)

## Context

T-1229a per docs/reports/T-1229-inception.md. Foundation hub method for the inbox.status migration. Adds a server-side aggregation primitive on the channel surface so clients can replace `inbox.status` (which walks the spool) with a single round-trip channel call. Backward-compatible extension of existing `channel.list` rather than a new method (refinement decision below).

## Acceptance Criteria

### Agent
- [x] `Bus::topic_record_count(topic) -> Result<u64>` added to `crates/termlink-bus/src/lib.rs` + thin `Meta::count_records(topic) -> Result<u64>` in `crates/termlink-bus/src/meta.rs` using `SELECT COUNT(*) FROM records WHERE topic = ?`.
- [x] `handle_channel_list_with` in `crates/termlink-hub/src/channel.rs` extended: each topic entry now includes a `count` field with the result of `bus.topic_record_count(name)`. Errors counting a topic emit `count: 0` (graceful — never fail the whole list because of one topic).
- [x] Unit test in `termlink-bus`: `topic_record_count_reflects_posts_and_unknown_topics` — posts 3 to alice, 0 to bob, asserts counts; unknown topic returns 0 not error.
- [x] Unit test in `termlink-hub` channel.rs: `list_includes_count_per_topic` — posts to one prefix, asserts count=3 vs count=0; prefix excludes other prefixes.
- [x] `cargo build -p termlink-bus -p termlink-hub` clean (0 new warnings) — verified.
- [x] `cargo test -p termlink-bus -p termlink-hub` passes — bus 13/13 + hub channel 16/16 tests green.
- [x] Existing `channel.list` callers (`cmd_channel_list`, `termlink_channel_list` MCP tool) still work — they read `name` + `retention` only, ignore extras.

## Verification

cargo build -p termlink-bus -p termlink-hub 2>&1 | tail -5
cargo test -p termlink-bus topic_record_count 2>&1 | tail -10
cargo test -p termlink-hub channel_list 2>&1 | tail -15

## Decisions

### 2026-04-25 — Extend channel.list instead of new channel.list_topics
- **Chose:** Add `count` field to each topic entry in existing `channel.list(prefix?)` response.
- **Why:** (1) Existing method already takes the same `prefix?` parameter and returns topics — adding `count` is a single-field extension. (2) Backward-compatible: existing callers (cmd_channel_list, termlink_channel_list MCP) read `name` + `retention` and ignore extras. (3) Keeps T-1166 retirement scope unchanged — no new method to retire later. (4) The inception report's "channel.list_topics" was suggestive naming, not normative; the principle (server-side aggregation, single round-trip, preserves fleet-doctor invariant) is satisfied either way.
- **Rejected:** New `channel.list_topics` method — adds protocol surface for no functional gain; a method with `prefix?` and topic-list semantics already exists.

## Updates

### 2026-04-25T10:11:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1233-hub-channellisttopicsprefix-router-metho.md
- **Context:** Initial task creation

### 2026-04-25T10:13:20Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-25T10:15:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
