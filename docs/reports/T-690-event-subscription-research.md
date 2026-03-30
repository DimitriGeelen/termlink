# T-690: Event Subscription Research

## Problem

TermLink's event system is entirely poll-based. Every consumer (`watch`, `wait`, `collect`, `dispatch`, `request`) runs a `tokio::select!` loop with a fixed sleep interval (250-500ms) and makes RPC calls to pull new events.

**Consequences:**
1. **Latency floor** — events cannot arrive faster than the poll interval (250ms best case)
2. **Wasted work** — most polls return zero events (idle sessions polled at same rate as active ones)
3. **Gap risk** — if a session emits events faster than poll frequency, ring buffer overflow loses events between polls (gap_detected flag signals loss but no recovery)
4. **Fan-in scaling** — `event.collect` polls N sessions sequentially through the hub; latency grows linearly with session count
5. **Hub load** — supervisor polls all sessions for `session.exited` events; dispatch polls all workers

## Current Architecture

### Event Bus (events.rs)
- Ring buffer (VecDeque), default capacity 1024, monotonic sequence numbers
- `emit(topic, payload)` → assigns seq, stores event
- `poll(since_seq)` / `poll_topic(topic, since_seq)` → return events with seq > cursor
- **No pub/sub, no channels, no notification mechanism**

### CLI Polling Patterns
| Command | Interval | Mechanism |
|---------|----------|-----------|
| `watch` | 500ms | Loop: sleep → event.poll RPC per session |
| `wait` | 250ms | Loop: sleep → event.poll RPC, check topic match |
| `collect` | 500ms | Loop: sleep → event.collect RPC (hub fans out) |
| `dispatch` | 500ms | Loop: sleep → event.collect RPC for workers |
| `request` | 250ms | Loop: sleep → event.poll RPC for reply_topic |

All follow identical structure:
```rust
loop {
    tokio::select! {
        _ = ctrl_c() => break,
        _ = sleep(interval) => {
            // RPC poll, process events, update cursor
        }
    }
}
```

### Hub Event Routing (router.rs)
- `event.broadcast` — fan-out `event.emit` to targets (concurrent, 5s timeout each)
- `event.collect` — fan-in `event.poll` from targets (concurrent, sorted by timestamp)
- `event.emit_to` — direct push through hub (already one-shot push)
- All routing is request-response; no persistent subscriptions

### Existing Push Infrastructure
The **data plane** already has production push delivery:
- `tokio::sync::broadcast::Sender<Vec<u8>>` in data_server.rs for PTY output streaming
- Multi-client support via `resubscribe()`
- Graceful overflow handling with `RecvError::Lagged(n)`
- Binary frame codec for efficient streaming

This proves the pattern works. The question is whether to lift it into the event/control plane.

## Design Options

### Option A: EventBus Internal Broadcast
Add `tokio::sync::broadcast::Sender<Event>` to EventBus alongside the ring buffer.

```rust
pub struct EventBus {
    events: VecDeque<Event>,
    capacity: usize,
    seq: u64,
    tx: broadcast::Sender<Event>,  // NEW
}
```

- `emit()` writes to ring buffer AND sends on broadcast channel
- New `subscribe()` returns a `broadcast::Receiver<Event>`
- Existing `poll()` unchanged (backward compatible)
- RPC handler adds `event.subscribe` method that holds connection open and streams events

**Pros:** Minimal change, leverages existing tokio primitive, proven pattern from data plane
**Cons:** Broadcast channel drops messages under pressure (lag), subscription lifetime tied to connection

### Option B: Data Plane Event Stream
Extend the existing data plane to carry typed events alongside binary PTY data.

- Add event frame type to binary codec (already has frame type field)
- Subscriptions use data plane connection (already handles streaming)
- Hub subscribes to session data planes and aggregates

**Pros:** Reuses existing streaming infrastructure, single connection for PTY + events
**Cons:** Mixes concerns (binary PTY data + structured events), data plane is optional (not all sessions start one)

### Option C: Long-Poll RPC (Minimal Change)
Instead of true push, change `event.poll` to hold the connection open until events arrive or timeout.

```rust
// event.poll with wait_timeout parameter
// If no events, sleep up to wait_timeout, checking broadcast channel
```

- No new connection type needed
- Backward compatible (existing poll works with timeout=0)
- Reduces empty polls to near zero

**Pros:** Minimal protocol change, backward compatible, no new infrastructure
**Cons:** Still one-request-per-batch (not true streaming), connection held open wastes resources

## Recommendation

**Option A (EventBus Internal Broadcast)** is the best path:

1. **Proven pattern** — data plane already uses `broadcast::Sender` successfully
2. **Clean separation** — events stay in control plane, PTY stays in data plane
3. **Backward compatible** — `poll()` still works for simple use cases
4. **Incremental** — can ship `subscribe()` on EventBus first, then wire RPC, then CLI
5. **Natural fit** — tokio broadcast channel semantics (lag detection, multi-consumer) match event bus requirements exactly

### Implementation Phases
1. Add `broadcast::Sender<Event>` to EventBus, wire `emit()` to broadcast
2. Add `event.subscribe` RPC method with streaming response
3. Update `watch` CLI to use subscription when available (fallback to poll)
4. Update `wait`/`collect`/`dispatch` to use subscriptions
5. Add hub subscription aggregation (subscribe to sessions, republish)
6. Add MCP `event_subscribe` tool

## Go/No-Go Assessment

**GO signals:**
- Data plane proves broadcast pattern works in production
- Latency floor (250ms) is measurable and impactful for orchestration use cases
- Implementation is incremental and backward compatible
- No new dependencies needed (tokio broadcast is already in use)

**NO-GO signals:**
- If polling latency is acceptable for all current use cases
- If connection lifetime management proves too complex for hub aggregation
- If the protocol needs to stay strictly request-response (no streaming RPC)

## Spike Results

### Spike 1: EventBus Broadcast (PASS)
- Added `broadcast::Sender<Event>` to EventBus alongside ring buffer
- `emit()` writes to both ring buffer and broadcast channel
- `subscribe()` returns `broadcast::Receiver<Event>` for real-time delivery
- 5 new tests: basic receive, no-replay, multi-subscriber, lag detection, no-subscriber safety
- **Result:** Fully backward compatible. All existing poll() tests pass unchanged.

### Spike 2: Long-Poll RPC Handler (PASS)
- Implemented `event.subscribe` as long-poll (not streaming) — waits on broadcast receiver with timeout
- Parameters: `timeout_ms` (default 5000), `topic` (optional filter), `max_events` (default 100)
- Returns normal JSON-RPC response — no protocol extension needed
- Added to auth scope as Observe (matches event.poll)
- 3 handler tests + 1 end-to-end test over Unix socket
- **Important finding:** Subscribe handler holds RwLock<SessionContext> read lock for full timeout. Emitters must access EventBus through read() + Mutex::lock() (not write()). This is correct since EventBus is Arc<Mutex>.
- **Result:** Latency drops from 250-500ms polling interval to ~0 (event arrival time).

### Spike 3: CLI Integration (NOT EXECUTED)
- Deferred to build phase — requires CLI command changes, which is implementation work beyond inception scope.
- Design is clear: `watch` command detects `event.subscribe` availability, falls back to `event.poll`.

## Recommendation Update

The hybrid approach works best: **Option A (broadcast channel) + Option C (long-poll RPC)**. Rather than choosing one design, we implemented both:
- Broadcast channel provides the push mechanism (Spike 1)
- Long-poll RPC provides the API surface without protocol changes (Spike 2)

This avoids the complexity of streaming RPC (no need to split the connection handler, no protocol extension) while achieving the same latency improvement.

## Dialogue Log

### 2026-03-30 — Initial Research
- Explored entire event system: EventBus, CLI commands, hub routing, MCP tools
- Found all 5 event consumers use identical poll-sleep-poll pattern
- Discovered data plane broadcast as proven push infrastructure
- Identified 3 design options with trade-offs
- Recommended Option A (EventBus broadcast) based on proven pattern and incremental path

### 2026-03-30 — Spike Execution
- Spike 1: broadcast::Sender added to EventBus, 5 tests pass, fully backward compatible
- Spike 2: event.subscribe long-poll handler, 4 tests pass, no protocol changes needed
- Key discovery: hybrid Option A+C is better than either alone
- GO recommendation: all 3 go criteria met (backward compat, no protocol redesign, measurable latency improvement)
