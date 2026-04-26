---
id: T-1289
name: "T-243 dep: Verify push delivery is default for channel.subscribe"
description: >
  Per T-243 inception (Agent B priority #2): without push, immediate response is structurally impossible. Verify channel.subscribe currently uses push (event-driven WebSocket-style stream) and not poll-on-interval. If poll-based, escalate to a separate enabling task to flip to push. Quick spike — should be hours not days. Independently testable; runs in parallel with other child tasks.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-243, transport, push]
components: [crates/termlink-bus/src/lib.rs, crates/termlink-hub/src/channel.rs]
related_tasks: []
created: 2026-04-26T09:32:08Z
last_update: 2026-04-26T09:48:33Z
date_finished: 2026-04-26T09:48:33Z
---

# T-1289: T-243 dep: Verify push delivery is default for channel.subscribe

## Context

T-243 inception (Agent B priority #2): without push delivery, "immediate response is structurally impossible" — there's a polling-interval floor on every conversation turn.

## Findings (2026-04-26 spike)

- `channel.subscribe` (`crates/termlink-hub/src/channel.rs:352`) is a one-shot snapshot RPC: reads up to `limit` records from cursor and returns. **No timeout_ms, no notify, no streaming.** Pure poll.
- `event.subscribe` (router.rs:442 `handle_hub_subscribe`) DOES support long-poll via `timeout_ms` (default 5000ms), but operates on the session-event aggregator — does NOT see channel.post writes. The mirror (T-1162) goes the other direction (event.broadcast → channel:broadcast:global).
- `termlink-bus` itself has no notify primitive: `Bus::post` writes to disk, `Bus::subscribe` reads from disk. No inter-poster-subscriber signal.

**Conclusion:** channel.subscribe is poll-only; T-243 dialog liveness needs push-like latency. Smallest fix: add a per-topic `tokio::sync::Notify` to `Bus`, signal on post, expose a long-poll variant `Bus::subscribe_blocking(topic, cursor, timeout)`. Hub adds optional `timeout_ms` parameter to channel.subscribe RPC; existing snapshot semantics preserved when omitted.

## Acceptance Criteria

### Agent
- [x] Spike confirmed channel.subscribe is poll-only (handle_channel_subscribe: snapshot RPC, no timeout/notify)
- [x] Confirmed event.subscribe long-poll exists but doesn't observe channel.post (only session events)
- [x] Add `Notify` per-topic to Bus; `post` calls `notify_waiters()` after successful append (lib.rs `Bus.notifiers`, `notifier_for`, `post`)
- [x] Add `Bus::subscribe_blocking(topic, cursor, timeout) -> Result<SubscribeIter>` (lib.rs) — fast path checks index, slow path registers waiter + re-checks (lost-wakeup safe), times out to empty iterator
- [x] Extend `handle_channel_subscribe` to accept optional `timeout_ms` parameter (channel.rs) — default 0 = snapshot, >0 = long-poll; capped at 60_000 to bound RPC handler lifetime
- [x] Test: subscribe with cursor=N (no records yet) blocks; concurrent post wakes it (`subscribe_blocking_wakes_on_concurrent_post`) — proves push-like wake latency (50ms producer sleep, total <1s)
- [x] Test: subscribe with timeout that expires returns empty, no error (`subscribe_blocking_returns_empty_on_timeout` + RPC-level `subscribe_timeout_ms_returns_empty_on_no_records`)
- [x] cargo test passes — 31/31 termlink-bus, 224/224 termlink-hub (was 222 + 2 new long-poll tests)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
cargo test -p termlink-bus --lib 2>&1 | tail -5 | grep -E "test result: ok|passed"
cargo test -p termlink-hub --lib subscribe_timeout 2>&1 | tail -5 | grep -E "test result: ok|passed"

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

### 2026-04-26T09:32:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1289-t-243-dep-verify-push-delivery-is-defaul.md
- **Context:** Initial task creation

### 2026-04-26T09:43:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-26T09:48:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
