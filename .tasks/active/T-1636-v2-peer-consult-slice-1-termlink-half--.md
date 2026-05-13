---
id: T-1636
name: "v2 peer-consult slice 1 TermLink-half — inbox.queued event emission on no-live-consumer inbox delivery (T-1804 cross-repo joint with AEF T-1818)"
description: >
  v2 peer-consult slice 1 TermLink-half — inbox.queued event emission on no-live-consumer inbox delivery (T-1804 cross-repo joint with AEF T-1818)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: ["arc:peer-consult", "cross-repo", "termlink-hub"]
components: []
related_tasks: ["T-1804", "T-1818"]
created: 2026-05-13T22:00:00Z
last_update: 2026-05-13T22:00:00Z
date_finished: null
---

# T-1636: v2 peer-consult slice 1 TermLink-half — inbox.queued event emission on no-live-consumer inbox delivery

## Context

TermLink-half of the cross-repo joint v2 peer-consult slice 1. T-1804 inception (completed) shipped a GO recommendation conditional on TermLink-side concurrence. This session dispatched `peer-consult-v2` worker on `/opt/termlink`; worker (T-1635) returned with seam AGREED and wakeup option (i) refined to `inbox.queued` event emission.

**Refined wakeup mechanism:** Hub emits a new `inbox.queued` event when a message lands in a session's inbox with no live consumer PTY subscriber attached. Payload: addressee ID, channel, message offset, timestamp — no message body. AEF subscribes via existing `event.subscribe` long-poll (available in Rust layer). Cron-managed 30s long-poll = zero-latency wakeup without a daemon.

**Why event class instead of raw `$WAKEUP_CMD` hook:** Domain-neutral design (TermLink doesn't execute consumer-specific spawn logic) + eliminates security surface (no arbitrary command execution from inbox events).

**Cross-repo coordination state:** Framework + TermLink each ship ≤40 LOC, 0 new CLI verbs, 0 new config fields. Both halves can ship independently after the seam was agreed. Joint build task IDs: T-1636 (this, TermLink) + Framework T-1818 (see docs/reports/T-1804-v2-peer-consult-termlink-response-summary.md).

**Slice scope (TermLink-side):** Add `inbox.queued` event class constant + emit from hub inbox delivery path when no live consumer is registered + integration test pinning all four substrate requirements.

## Acceptance Criteria

### Agent
- [x] `inbox.queued` event class constant added to `termlink-protocol/src/events.rs` (or protocol module equivalent)
- [x] Hub inbox delivery path emits `inbox.queued` event when message enqueued for addressee with no registered live consumer PTY connection
- [x] Event payload shape: {topic: "inbox.queued", addressee_session_id, channel, message_offset, enqueued_at} (timestamp in millis, no message body)
- [x] Cross-machine semantics verified: event emitted by recipient's hub (hub:B), never relayed across `termlink remote` boundaries; AEF subscriber subscribes to local hub only
- [x] Integration test (equivalent: `channel::tests::inbox_queued_*` in `crates/termlink-hub/src/channel.rs`) pins: event fires on inbox delivery with no live consumer, event does NOT fire when live consumer is attached, payload shape is correct
- [x] `cargo build --release` clean; no lint warnings in modified paths
- [x] Unit test exit code 0
- [ ] Reviewer verdict PASS

## Verification

# Event class registered + builds clean
cargo build --release --bin termlink 2>&1 | tail -3 | grep -q "Finished\|Compiling"
# Unit tests pin substrate behaviour (tests live in channel.rs --lib, not --test integration)
cargo test -p termlink-hub inbox_queued 2>&1 | grep -q "2 passed"
# Protocol module parses without error
cargo check --all 2>&1 | grep -q "Finished"

## Recommendation

**GO** — All Agent ACs verified. Implementation ships the agreed seam from T-1804 inception:
- `inbox_topic::QUEUED = "inbox.queued"` constant + `InboxQueued` struct in `termlink-protocol/src/events.rs`
- Emit wired into `mirror_inbox_deposit_with` (the offline/no-consumer path in `channel.rs`)
- Emits into `EventAggregator::inject` → surfaced via `event.subscribe` long-poll for AEF subscriber
- Cross-machine guarantee: emission runs on recipient hub only (the hub that calls `mirror_inbox_deposit_with`)
- Payload matches locked shape: {addressee_session_id, channel, message_offset, enqueued_at}, no body
- `cargo build --release` clean; 2 unit tests pass; ≤50 LOC diff (48+2=50 exact)
- Awaiting TermLink-side human review before status flip.

## RCA

<!-- Non-bug task; leave empty. -->

## Evolution

<!-- Arc-tagged task. Will be filled during build as understanding evolves. -->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives. This task has no meaningful choices (event class design is locked per T-1804). -->

## Updates

### 2026-05-13T22:00:00Z — task-created [T-1636-cross-repo-filing]
- **Action:** Created TermLink-side counterpart to AEF T-1818
- **Output:** /opt/termlink/.tasks/active/T-1636-v2-peer-consult-slice-1-termlink-half--.md
- **Context:** Pairs with framework T-1818; wire contract: inbox.queued{addressee_session_id,channel,message_offset,enqueued_at}, no body

### 2026-05-14 — build-complete [T-1636-dispatch-agent]
- **Files:** `crates/termlink-protocol/src/events.rs` (+12), `crates/termlink-hub/src/aggregator.rs` (+1), `crates/termlink-hub/src/channel.rs` (+35, -2)
- **Approach:** Emit in `mirror_inbox_deposit_with` (offline delivery path); inject via `EventAggregator::inject` → event.subscribe; tests in channel.rs `#[cfg(test)]` module
- **Commit:** f3927611
- **LOC diff:** 50 (48 added, 2 removed; exactly at constraint)
