# T-009: Concurrency, Ordering, and Backpressure — Exploration Report

> Generated: 2026-03-12 | Source: TermLink agent mesh (explore-T009-r2)

## 1. Current Concurrency Model

**Control plane (JSON-RPC):** `Arc<RwLock<SessionContext>>` per session. Each client connection gets a tokio task. Requests acquire either a read lock (`dispatch()`) or write lock (`dispatch_mut()`) — determined by `needs_write()` per method. This means:
- Multiple readers can poll events concurrently
- Writers (emit, execute, inject, update) serialize against each other
- No per-connection state — each request independently acquires/releases the lock

**Data plane (binary frames):** `tokio::sync::broadcast` channel (capacity 64) for PTY output. Each client gets its own receiver via `resubscribe()`. Input frames go directly to `Arc<PtySession>` which uses `Arc<Mutex<OwnedFd>>` for write serialization.

**Hub:** One tokio task per connection. `AtomicU32` tracks active connections for graceful drain. `watch::channel` for shutdown signaling. No shared mutable state in the hub — it's stateless, forwarding RPCs to sessions via Unix socket calls.

**EventBus:** `Arc<Mutex<EventBus>>` — a `VecDeque` ring buffer (capacity 1024) behind a tokio Mutex. Emit and poll both acquire the same mutex.

## 2. Ordering Guarantees

**Within a single session:**
- **Events:** Monotonic `next_seq` counter inside `EventBus` under mutex — **strictly ordered**. Cursor-based polling (`seq > since_seq`) guarantees no duplicates.
- **Data frames:** `FrameWriter` has a per-writer `sequence` counter — ordered per-connection. Broadcast channel preserves send order for each receiver.

**Cross-session (hub broadcast):**
- `event.broadcast` iterates sessions **sequentially** (`for reg in &registrations`), calling `rpc_call` per session. No ordering guarantee across sessions — network/scheduling jitter. Matches assumption A3.

**Concurrent pollers on one session:** All see the same `EventBus` state (mutex-serialized). No event loss from concurrent reads since polling is non-destructive (filter, not dequeue).

## 3. Backpressure Mechanisms (and Gaps)

**Data plane — has backpressure:**
- `broadcast::channel(64)` — bounded to 64 messages
- Slow consumers get `RecvError::Lagged(n)` — frames are **dropped** with a log warning
- This is lossy backpressure, not flow-control backpressure

**Control plane event bus — NO backpressure:**
- Ring buffer capacity 1024 — oldest events silently evicted on overflow (`pop_front`)
- No notification to pollers that events were dropped
- A slow poller whose cursor points to an evicted event will simply miss it — **silent event loss**
- No mechanism to detect the gap (cursor < oldest seq in buffer = events were lost)

**Hub broadcast — NO backpressure:**
- Sequential fan-out with no timeout per target (uses default TCP/socket timeouts)
- A slow/dead session blocks the broadcast loop for all subsequent targets
- No concurrency limit on broadcast targets
- `failed` count is returned but not used for flow control

## Key Gaps Summary

| Area | Issue | Severity |
|------|-------|----------|
| EventBus ring overflow | Silent event loss — no gap detection for pollers | High |
| Hub broadcast fan-out | Sequential, blocking — one dead session stalls all | Medium |
| Data plane lagging | Lossy (drops frames) — acceptable for PTY but not for reliable events | Low (by design) |
| Polling latency floor | 2s watcher interval — not a concurrency issue per se | Low |
| No concurrent poller tests | Zero test coverage for multi-client scenarios | Medium |

## Implications for Assumptions

- **A1 (5 concurrent agents ok):** Likely true for polling (non-destructive reads), but untested. The RwLock allows concurrent reads.
- **A2 (single-session ordering):** Confirmed — monotonic seq under mutex.
- **A3 (cross-session unordered):** Confirmed — sequential fan-out, no global clock.
- **A4 (backpressure unnecessary locally):** Partially true — ring buffer overflow is the real risk. At 1024 capacity with low event rates it's fine, but 10 agents each emitting rapidly could overflow before slow pollers catch up.
