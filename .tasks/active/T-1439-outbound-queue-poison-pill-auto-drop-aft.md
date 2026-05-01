---
id: T-1439
name: "outbound queue: poison-pill auto-drop after N hub-reject attempts"
description: >
  outbound queue: poison-pill auto-drop after N hub-reject attempts

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T16:38:34Z
last_update: 2026-05-01T16:38:34Z
date_finished: null
---

# T-1439: outbound queue: poison-pill auto-drop after N hub-reject attempts

## Context

`BusClient::flush()` (bus_client.rs:148) currently breaks out of the drain loop
on the FIRST hub-reject, only bumping `attempts` on the head entry. This means
any single permanent-error post (unknown topic, malformed payload, etc.) head-
of-line blocks the entire queue forever. Discovered T-1438 cycle:
`/root/.termlink/outbound.sqlite` had id=1 with attempts=319 on a topic the
hub doesn't recognize, blocking ids 2-6 indefinitely.

Fix: after N attempts (POISON_THRESHOLD = 10), `pop()` the entry instead of
`break`-ing. Log loudly so operators see the drop. Continue draining the
remainder of the queue. Below threshold, keep current break behaviour
(avoid spinning on transient hub-side issues).

This also closes the conceptual half of the T-1429 Phase-2 outbound-queue gap
that's actually a *recovery* concern — the "preserve hub_addr in queued
post_json" half is moot for current binary because TCP --hub posts bypass
the queue entirely (T-1385 + e3f2381f), and unix-socket queue entries are
correctly local-bound.

## Acceptance Criteria

### Agent
- [x] `BusClient::flush()` pops entries whose `attempts + 1 >= POISON_THRESHOLD` (10) — bus_client.rs:148. Logs at WARN with queue_id, attempts, topic, msg_type, error. New `dropped_poison` field on `FlushReport`
- [x] Drain continues past the dropped entry (`continue` rather than `break`) so subsequent entries get a chance — bus_client.rs:170
- [x] Below threshold: bump attempts and break (preserves current behavior for transient errors / race with hub restart) — bus_client.rs:177-184
- [x] Added `OfflineQueue::peek_oldest_with_attempts()` so flush can read attempts without a separate roundtrip — offline_queue.rs:170
- [x] Unit tests: 2 new offline_queue tests (peek_oldest_with_attempts_returns_attempts_count + peek_oldest_with_attempts_empty_returns_none) covering the new query method. All 9 offline_queue tests + all 5 bus_client tests pass
- [x] **Live verification on .107** — `/root/.termlink/outbound.sqlite` had 6 stuck entries (id=1 with 322 attempts on `xhub-real-1777398973` head-blocking ids 2-6). After binary upgrade to 0.9.1678 + flush trigger: id=1 dropped (`flush: dropping poison post after 10 hub-reject attempts attempts=323`), ids 2-6 drained successfully (5 delivered). `queue-status` now shows pending=0 — queue restored from permanent head-block to clean state
- [x] cargo test passes for affected crates (`cargo test -p termlink-session offline_queue` + `bus_client` both green)

## Verification

cargo test -p termlink-session bus_client 2>&1 | tail -20
cargo build --release --bin termlink 2>&1 | tail -5

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

### 2026-05-01T16:38:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1439-outbound-queue-poison-pill-auto-drop-aft.md
- **Context:** Initial task creation

### 2026-05-01T16:46Z — fix shipped + live-drain verified [agent autonomous]
- **Action:** Modified `BusClient::flush()` (bus_client.rs:148) to drop poison entries after `POISON_THRESHOLD = 10` hub-reject attempts instead of head-of-line blocking forever. Added `OfflineQueue::peek_oldest_with_attempts()` so flush reads attempts in the same query as the post payload. Added `dropped_poison` field to `FlushReport` for telemetry
- **Live drain on .107:** queue had 6 stuck entries — id=1 with `xhub-real-1777398973` topic had been head-blocking the queue since 2026-04-28 (3 days, 322 attempts on a topic the local hub doesn't recognize). After install, single flush trigger via `termlink channel post`: id=1 dropped with WARN log `flush: dropping poison post after 10 hub-reject attempts attempts=323`, then ids 2-6 drained cleanly (5 successful posts). `queue-status` confirms `pending: 0`
- **Why this matters for chat-arc:** any future `--hub TCP` post that fails after the binary fell back to queue (rare on current binary post-T-1385 but possible) won't permanently brick the queue for the host. Auto-recovery within 10 flush cycles
- **Tests:** added 2 unit tests on the new query method; all 9 offline_queue + 5 bus_client tests pass
- **Versions:** built fresh binary 0.9.1678, installed system-wide on .107
