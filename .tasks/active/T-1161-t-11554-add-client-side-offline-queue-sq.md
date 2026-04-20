---
id: T-1161
name: "T-1155/4 Add client-side offline queue (SQLite) + flush task to termlink-session"
description: >
  Per T-1155 S-3. Client-side SQLite pending_posts + last_read_cursor tables. Queue on bus-unreachable, idempotent flush on reconnect via (sender_id, client_seq). Cap queue size; fail loudly when full. ~300 LOC Rust.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [T-1155, bus, offline-tolerance]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:08Z
last_update: 2026-04-20T14:12:08Z
date_finished: null
---

# T-1161: T-1155/4 Add client-side offline queue (SQLite) + flush task to termlink-session

## Context

Offline-tolerance for the T-1155 bus (per S-3 verdict): clients queue `channel.post` locally when the hub is unreachable and flush on reconnect. This closes the bootstrap paradox (the bus depends on the hub being reachable, but agents need to keep posting even when it's not).

Depends on: T-1160 (channel API exists). Caps local queue size to prevent unbounded growth (R3 mitigation from T-1155).

## Acceptance Criteria

### Agent
- [ ] Add `rusqlite` (or verify existing) to `crates/termlink-session/Cargo.toml`
- [ ] New module `crates/termlink-session/src/offline_queue.rs` exposes:
  - `OfflineQueue::open(path) -> OfflineQueue` — opens/creates `~/.termlink/outbound.sqlite`
  - `queue.enqueue(post: PendingPost) -> Result<QueueId>` — stores serialized post envelope + topic + timestamp; rejects with `QueueFull` when over cap
  - `queue.flush(client: &BusClient) -> FlushReport {sent: u64, failed: u64}` — pops oldest → POSTs to hub → on success deletes row; on transient failure leaves in place and breaks loop
  - `queue.size() -> u64`, `queue.peek_oldest() -> Option<PendingPost>`
- [ ] Queue cap: default `1000` entries; configurable via env `TERMLINK_OUTBOUND_CAP`; when full, new enqueue returns typed `QueueFull` error (loud failure, not silent drop — R3)
- [ ] Flush task: `tokio::task::spawn` a periodic flusher (5s interval, configurable) started automatically by `BusClient::connect()` — cancel-safe on drop
- [ ] `BusClient::post()` transparently routes through queue when hub unreachable: try direct RPC → on `Transport` error, enqueue + return `Ok(Queued)`; on success, return `Ok(Delivered{offset})`
- [ ] Unit tests: enqueue + flush roundtrip, cap enforcement, flush with hub down leaves queue intact, flush after hub comes back drains queue in order, concurrent enqueue from multiple tasks preserves FIFO within a topic
- [ ] Integration test: spin up a local hub, post 10 messages, kill hub, post 5 more (queued), restart hub, verify all 15 arrive in order
- [ ] `cargo build -p termlink-session && cargo test -p termlink-session offline_queue` passes
- [ ] `cargo clippy -p termlink-session -- -D warnings` passes
- [ ] New CLI verb `termlink channel queue-status` → shows pending count + oldest timestamp (for debugging; optional per scope — punt to follow-up if over-budget)

### Human
- [ ] [REVIEW] Approve the queue-full policy (loud reject vs silent drop-oldest)
  **Steps:**
  1. Confirm "reject new posts when full" matches your failure-mode preference
  2. Alternative: drop-oldest ring behavior — would hide the overflow from callers
  3. The bus spec (R3) recommends loud reject; verify this is still correct
  **Expected:** Approval or switch to drop-oldest ring
  **If not:** Note the required policy and open a refactor task

## Verification

cargo build -p termlink-session
cargo test -p termlink-session offline_queue
cargo clippy -p termlink-session -- -D warnings
grep -q "OfflineQueue" crates/termlink-session/src/offline_queue.rs
grep -q "outbound.sqlite" crates/termlink-session/src/offline_queue.rs

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

### 2026-04-20T14:12:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1161-t-11554-add-client-side-offline-queue-sq.md
- **Context:** Initial task creation
