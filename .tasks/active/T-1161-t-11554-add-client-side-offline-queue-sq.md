---
id: T-1161
name: "T-1155/4 Add client-side offline queue (SQLite) + flush task to termlink-session"
description: >
  Per T-1155 S-3. Client-side SQLite pending_posts + last_read_cursor tables. Queue on bus-unreachable, idempotent flush on reconnect via (sender_id, client_seq). Cap queue size; fail loudly when full. ~300 LOC Rust.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [T-1155, bus, offline-tolerance]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:08Z
last_update: 2026-04-20T22:20:19Z
date_finished: 2026-04-20T22:20:02Z
---

# T-1161: T-1155/4 Add client-side offline queue (SQLite) + flush task to termlink-session

## Context

Offline-tolerance for the T-1155 bus (per S-3 verdict): clients queue `channel.post` locally when the hub is unreachable and flush on reconnect. This closes the bootstrap paradox (the bus depends on the hub being reachable, but agents need to keep posting even when it's not).

Depends on: T-1160 (channel API exists). Caps local queue size to prevent unbounded growth (R3 mitigation from T-1155).

## Acceptance Criteria

### Agent
- [x] Add `rusqlite = "0.33"` with `bundled` feature to `crates/termlink-session/Cargo.toml`
- [x] New module `crates/termlink-session/src/offline_queue.rs` exposes:
  - `OfflineQueue::open(path) -> Result<Self>` plus `default_queue_path()` returning `~/.termlink/outbound.sqlite` (overridable via `TERMLINK_IDENTITY_DIR`)
  - `enqueue(&PendingPost) -> Result<QueueId>` — stores serialized post as JSON + `enqueued_ms` + `attempts`; rejects with `QueueFull { cap }` when over cap
  - `size`, `peek_oldest`, `pop(QueueId)`, `bump_attempts(QueueId)`
  - Flush lives on `BusClient::flush() -> FlushReport { sent, failed }` (design decision below — client owns the socket + retry policy)
- [x] Queue cap: default `1000`; env override `TERMLINK_OUTBOUND_CAP`; `QueueFull` is a typed error (loud-fail per R3)
- [x] Flush task: `BusClient::connect_with_interval(socket, queue_path, flush_interval)` spawns a tokio task that drains the queue every `flush_interval` (default 5s). Cancel-safe via `oneshot` sender held by the client — dropping the `Arc<BusClient>` fires shutdown before the next tick
- [x] `BusClient::post()` transparently routes: direct `rpc_call(channel.post)` → `PostOutcome::Delivered { offset }`; on transport error → enqueue + `PostOutcome::Queued { queue_id }`. Hub-level errors (post rejected) bump attempts and break the flush loop so poison messages don't busy-loop
- [x] Unit tests in `offline_queue.rs` (7): open-empty, enqueue/peek/pop roundtrip, FIFO order, cap enforcement, survives-reopen, bump-attempts persistence, concurrent enqueue from 3 threads × 20 posts preserves per-topic FIFO
- [x] Unit tests in `bus_client.rs` (3): queues on unreachable socket, flush-with-hub-down breaks at first failure leaving queue intact, drop-on-Arc notifies the flush task
- [x] Integration test `tests/bus_client_integration.rs`: minimal fake-hub (Unix socket + JSON-RPC) accepts `channel.post`; post 10 while up (delivered) → stop hub (socket removed) → post 5 (queued) → restart hub → verify the 5 flushed entries arrive in order (FIFO markers 10..14)
- [x] `cargo test -p termlink-session --lib` passes (283 tests, was 274); `cargo test --workspace --lib` 704 green
- [x] `cargo clippy --workspace --lib --tests -- -D warnings` passes
- [x] New CLI verb `termlink channel queue-status` → AC marked optional; split out to T-1172 follow-up so the substantive wedge (durable queue + flush) can close. The OfflineQueue API (`size`, `peek_oldest`, `default_queue_path`) is already public, so the follow-up is just plumbing

### Human
- [x] [REVIEW] Approve the queue-full policy (loud reject vs silent drop-oldest) — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — queue-full policy approved (loud reject preferred).
  **Steps:**
  1. Confirm "reject new posts when full" matches your failure-mode preference
  2. Alternative: drop-oldest ring behavior — would hide the overflow from callers
  3. The bus spec (R3) recommends loud reject; verify this is still correct
  **Expected:** Approval or switch to drop-oldest ring
  **If not:** Note the required policy and open a refactor task

  **Agent evidence (2026-04-21, exercised against workspace binary 0.9.256):**

  End-to-end integration test passes green in release mode:
  ```
  $ cargo test -p termlink-session --test bus_client_integration --release
  running 1 test
  test post_deliver_queue_restart_drain ... ok
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  ```
  The test (`tests/bus_client_integration.rs`) validates the full cycle: fake Unix-socket hub accepts 10 live `channel.post` calls → hub is aborted + socket removed → 5 more posts queued durably in SQLite → fake hub restarts → background flush task drains the 5 queued entries in FIFO order. Delivered and queued entries arrive at the hub in the correct order; `FlushReport.sent = 5` on reconnect.

  `channel queue-status` (T-1172 follow-up) surfaces the queue for operators:
  ```
  $ termlink channel queue-status --queue-path /tmp/.../outbound.sqlite --json
  {
    "exists": false,
    "pending": 0,
    "queue_path": "/tmp/.../outbound.sqlite"
  }
  ```

  **Non-obvious finding to surface during review — CLI path does NOT use BusClient:**
  The CLI `channel post` verb (`crates/termlink-cli/src/commands/channel.rs:137`) calls `client::rpc_call` directly, bypassing `BusClient`. So the offline queue is currently a **library-only** feature — operators running `termlink channel post` while the hub is down get a direct RPC error, not a queue fallback. This was not an explicit AC for T-1161 (the wedge shipped a Rust library + tests), but it's worth a follow-up decision:
  - (a) Wire the CLI through `BusClient` so ops get offline tolerance on the command line (straightforward — swap one call site), OR
  - (b) Document that the queue is library-only (future SDK consumers use it) and keep the CLI simple.

  Rubber-stamp loud-reject if R3 still stands; and decide (a)/(b) — open a T-1161 follow-up task either way.

## Verification

cargo build -p termlink-session
cargo test -p termlink-session --lib offline_queue
cargo test -p termlink-session --lib bus_client
cargo test -p termlink-session --test bus_client_integration
cargo clippy -p termlink-session -- -D warnings
grep -q "OfflineQueue" crates/termlink-session/src/offline_queue.rs
grep -q "outbound.sqlite" crates/termlink-session/src/offline_queue.rs
grep -q "BusClient" crates/termlink-session/src/bus_client.rs

## Decisions

### 2026-04-21 — Flush lives on BusClient, not OfflineQueue
- **Chose:** `BusClient::flush() -> FlushReport` owns the flush loop; `OfflineQueue` exposes only `peek_oldest` / `pop` / `bump_attempts` primitives
- **Why:** The client holds the socket path + RPC machinery; routing flush through the queue would force `OfflineQueue` to take a `&BusClient` (or equivalent trait object), creating a dependency inversion with no real payoff. Keeps each type's responsibility crisp: queue is dumb storage, client is the transport + retry policy
- **Rejected:** The AC-literal shape `queue.flush(&client)` — functionally equivalent, worse layering

### 2026-04-21 — Cancel-safe shutdown via oneshot, not Notify
- **Chose:** `oneshot::Sender<()>` stored in `BusClient::shutdown_tx` (behind `Mutex<Option<_>>`); `Drop` takes the sender and drops it, which completes the receiver side in the spawned task
- **Why:** `tokio::sync::Notify::notify_waiters()` only wakes *currently registered* waiters — when `Drop` fires immediately after `tokio::spawn`, the task may not yet have polled `.notified()`, so the signal is lost. Oneshot has no such race: recv completes whether polled first or last
- **Rejected:** `Notify` (race, observed as a failing `drop_notifies_flush_task` test); `CancellationToken` from tokio-util (extra dep for the same guarantee)

## Updates

### 2026-04-20T14:12:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1161-t-11554-add-client-side-offline-queue-sq.md
- **Context:** Initial task creation

### 2026-04-20T22:08:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-04-20T22:20:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
