# T-256 Q1: TermLink Messaging Primitives Inventory

**Task:** T-256 — Interactive Multi-Agent Communication
**Date:** 2026-03-23
**Scope:** CLI commands, protocol RPCs, and semantics for inter-session messaging

---

## 1. CLI Commands for Bidirectional Communication

### One-Way Emission

| Command | Semantics | Timeout | File |
|---------|-----------|---------|------|
| `emit` | Fire-and-forget event to local bus | None | `events.rs:52-80` |
| `broadcast` | Hub fan-out to N sessions | 5s per target | `events.rs:82-125` |
| `inject` | PTY keystroke injection | None | `pty.rs:198-230` |

### One-Way Listeners

| Command | Semantics | Timeout | File |
|---------|-----------|---------|------|
| `watch` | Poll events from multiple sessions (500ms interval) | Ctrl+C | `events.rs:127-240` |
| `wait` | Block until one matching event | Configurable (default: forever) | `events.rs:242-300` |
| `collect` | Hub fan-in from N sessions with per-session cursors | Ctrl+C | `events.rs:359-453` |
| `agent listen` | Poll `agent.request` events | Configurable (default: forever) | `agent.rs:148-218` |
| `poll` | Single-shot event snapshot | None | `events.rs:6-50` |
| `topics` | List distinct topics in event bus | None | `events.rs:302-357` |

### Bidirectional Request-Reply

| Command | Semantics | Timeout | File |
|---------|-----------|---------|------|
| `request` | Emit on topic → poll reply topic, correlated by `request_id` | 300s (cfg) | `execution.rs:109-215` |
| `agent ask` | Typed agent protocol: emit `agent.request` → poll `agent.response`/`agent.status` | 30s (cfg) | `agent.rs:12-146` |
| `attach` | Interactive PTY over RPC polling (100ms) + stdin forwarding | Ctrl+] | `pty.rs:259-304` |
| `stream` | Real-time binary data plane for PTY I/O | Ctrl+] | `pty.rs:407-473` |

### Generic RPC

| Command | Semantics | File |
|---------|-----------|------|
| `send` | Raw JSON-RPC call to any session method | `session.rs:401-426` |

All paths relative to `crates/termlink-cli/src/commands/`.

---

## 2. Protocol-Level RPCs

Source: `crates/termlink-protocol/src/control.rs`

### Session-Local (handled by session process)

| RPC Method | Type | Purpose |
|------------|------|---------|
| `event.emit` | Request-Response | Append event to local ring buffer (1024 slots) |
| `event.poll` | Request-Response | Delta-poll with seq cursor + gap detection |
| `event.topics` | Request-Response | List distinct topics in buffer |
| `event.state_change` | Notification | Reserved, not implemented |
| `event.error` | Notification | Reserved, not implemented |

### Hub-Level (handled by hub router)

| RPC Method | Type | Purpose |
|------------|------|---------|
| `event.broadcast` | Request-Response | Fan-out: emit to N sessions concurrently (5s timeout each) |
| `event.collect` | Request-Response | Fan-in: poll from N sessions, enrich with source metadata |
| `orchestrator.route` | Request-Response | Discover sessions by selector → forward RPC → first success wins |
| `session.discover` | Request-Response | Filter sessions by tags/roles/capabilities/name |

### Event-Based Topics (not RPCs — structured payloads on the event bus)

**Agent protocol** (`crates/termlink-protocol/src/events.rs:115-197`):
- `agent.request` — `{request_id, from, to, action, params, timeout_secs}`
- `agent.response` — `{request_id, from, status: Ok|Error, result, error_message}`
- `agent.status` — `{request_id, from, phase, message, percent}`

**Task delegation** (`events.rs:42-113`):
- `task.delegate` — `{task_id, command, args, timeout_secs}`
- `task.accepted` — `{task_id, message}`
- `task.progress` — `{task_id, percent, message}`
- `task.completed` — `{task_id, result}`
- `task.failed` — `{task_id, error_code, message, retryable}`
- `task.cancelled` — `{task_id, reason, cancelled_by}`

**File transfer** (`events.rs:199-260`):
- `file.init` → `file.chunk` (base64, chunked) → `file.complete` (SHA-256) | `file.error`

---

## 3. Semantics Deep Dive

### Payload Format
All control-plane communication is **JSON-RPC 2.0** over newline-delimited Unix sockets (local) or TCP+TLS (remote). The data plane uses a binary frame protocol with typed frames (Output, Input, Resize, Signal, Ping/Pong, Close) — see `crates/termlink-protocol/src/data.rs:8-16`.

### Blocking vs Async
Nothing truly blocks. All commands use **async polling loops** with configurable intervals:
- `agent ask`: 250ms poll, 30s timeout
- `request`: 250ms poll, 300s timeout
- `watch`/`collect`: 500ms poll, no timeout (Ctrl+C)
- `wait`: 250ms poll, configurable timeout
- `attach`: 100ms poll for output + stdin forwarding
- `stream`: Zero-poll — real-time binary frames via `tokio::select!`

### Event Bus Mechanics
Each session has a **local ring buffer** (1024 events). Events don't cross session boundaries automatically. Inter-session messaging requires:
1. **Direct**: Client connects to target session's socket, calls `event.emit`
2. **Hub broadcast**: Client calls `event.broadcast` on hub, hub calls `event.emit` on each target
3. **Hub collect**: Client calls `event.collect` on hub, hub calls `event.poll` on each target

Gap detection: `event.poll` returns `gap_detected: true` + `events_lost` count when the cursor falls behind the ring buffer head (`events.rs:103-120` in termlink-session).

### Request-Reply Correlation
Both `request` and `agent ask` build request-reply on top of events:
1. Generate `request_id` (ULID or `req-<pid>-<ts>`)
2. Snapshot event cursor (`event.poll` to get current seq)
3. Emit request event
4. Poll for response events with matching `request_id`
5. Timeout if no match within deadline

The hub is **not involved** in correlation — it's purely client-side logic.

### Hub Message Flow

```
Sender ──event.broadcast──▶ Hub ──event.emit──▶ Target₁ (local bus)
                                 ──event.emit──▶ Target₂ (local bus)
                                 ──event.emit──▶ Target₃ (local bus)

Collector ──event.collect──▶ Hub ──event.poll──▶ Target₁
                                  ──event.poll──▶ Target₂
                                  returns merged + enriched events
```

Hub router: `crates/termlink-hub/src/router.rs:141-373`

### Orchestrator Route
`orchestrator.route` combines discovery + forwarding + failover in one call. Selector filters by `{tags, roles, capabilities, name}`. Tries candidates sequentially; first success wins. Includes bypass registry for Tier 3 pre-approved commands. (`router.rs:480-607`)

---

## 4. Gap Analysis: Worker-Pushes-Results-to-Orchestrator

**Pattern required:** An orchestrator dispatches work to N workers; each worker pushes results back asynchronously as they complete. Orchestrator collects results with progress tracking.

### What Already Works

**Strong foundation:**
- `event.broadcast` — orchestrator can fan-out a `task.delegate` event to workers
- `event.collect` — orchestrator can fan-in `task.completed`/`task.progress` from workers
- `agent ask` / `agent listen` — typed request-response with progress (`agent.status`)
- `task.*` topic schema — full lifecycle (delegate → accepted → progress → completed/failed)
- `orchestrator.route` — selector-based dispatch with failover
- Per-session cursor tracking in `collect` prevents duplicate processing

### Gaps

| # | Gap | Impact | Severity |
|---|-----|--------|----------|
| G1 | **No push notification** — workers can only emit to their *own* bus; orchestrator must poll to discover results. There's no "emit to another session's bus" RPC. Broadcast goes hub→targets, not worker→orchestrator. | Orchestrator must continuously poll via `event.collect`. Latency = poll interval (500ms default). For real-time results, this is wasteful. | Medium |
| G2 | **No structured dispatch CLI** — `task.delegate` schema exists in the protocol but no CLI command wraps it. Workers would need raw `emit` + `wait` with manual JSON construction. | Friction for multi-agent orchestration scripts. | Medium |
| G3 | **Ring buffer overflow risk** — 1024-event buffer per session. A burst of results from many workers could overflow the orchestrator's buffer before collect runs. Gap detection exists but data is lost. | Under high fan-in load, results silently drop (gap_detected=true). | High |
| G4 | **No delivery guarantee** — `event.broadcast` reports succeeded/failed counts but doesn't retry failures. No at-least-once semantics. | Worker may never receive the task delegation. | Medium |
| G5 | **No subscription/push channel** — all event consumption is poll-based. No way for a session to subscribe to another session's events and receive pushes. The data plane (binary frames) only handles PTY I/O. | Forces polling patterns everywhere. A push channel (even simple Unix socket notification) would eliminate G1. | High |
| G6 | **Collect doesn't filter by sender** — `event.collect` filters by topic and target sessions but can't filter by who emitted the event (the `from` field in agent payloads). Orchestrator must client-side filter. | Minor inefficiency; manageable. | Low |

### Viable Workaround (No Code Changes)

The existing primitives support the pattern with polling:

```bash
# Orchestrator: dispatch to workers
termlink broadcast --topic task.delegate --payload '{"task_id":"T-1","command":"build"}'

# Each worker: listen + respond
termlink wait --topic task.delegate | process_task
termlink emit --topic task.completed --payload '{"task_id":"T-1","result":"ok"}'

# Orchestrator: collect results
termlink collect --topic task.completed --interval 250
```

The `agent ask` / `agent listen` pair provides a higher-level alternative with built-in correlation and progress updates, but requires the orchestrator to poll each worker individually (no fan-in `agent ask`).

### Recommended Enhancements (for T-256)

1. **`emit-to` RPC** — Allow `event.emit` with a `target` param so workers push directly to orchestrator's bus (eliminates G1, G5 for this pattern)
2. **`task dispatch` CLI** — Wrap `task.delegate` + `collect task.completed` into a single orchestration command (eliminates G2)
3. **Configurable ring buffer** — Allow larger buffers for orchestrator sessions expecting high fan-in (mitigates G3)

---

*File references use paths relative to project root unless absolute. CLI command files are in `crates/termlink-cli/src/commands/`.*
