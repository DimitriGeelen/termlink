# TermLink Architecture

> Cross-terminal session communication system in Rust

## System Overview

TermLink enables **multiple terminal sessions to communicate** with each other via structured messaging over Unix sockets. It provides a dual-plane architecture: a **JSON-RPC control plane** for commands and queries, and a **binary data plane** for raw terminal I/O streaming.

```
┌─────────────────────────────────────────────────────────┐
│                        CLI (termlink)                    │
│  28 commands: register, list, ping, exec, attach, ...   │
└────────┬────────────────────┬────────────────────────────┘
         │ direct             │ via hub
         ▼                    ▼
┌─────────────────┐  ┌──────────────────────────────────┐
│   Session A      │  │             Hub                   │
│  ┌────────────┐  │  │  ┌─────────┐  ┌──────────────┐  │
│  │Control Sock│◄─┼──┼──│ Router  │  │ Supervisor   │  │
│  │ (JSON-RPC) │  │  │  └────┬────┘  │ (30s sweep)  │  │
│  ├────────────┤  │  │       │       └──────────────┘  │
│  │ Data Sock  │  │  │  ┌────┴────┐  ┌──────────────┐  │
│  │ (Binary)   │  │  │  │Discover │  │  Pidfile      │  │
│  └────────────┘  │  │  │Broadcast│  │  Lifecycle    │  │
│  ┌────────────┐  │  │  │Collect  │  └──────────────┘  │
│  │ Event Bus  │  │  │  │Forward  │                     │
│  │ (Ring Buf) │  │  │  └─────────┘                     │
│  ├────────────┤  │  └──────────────────────────────────┘
│  │ PTY/Exec   │  │
│  │ KV Store   │  │         Session B, C, ...
│  │ Scrollback │  │        (same structure)
│  └────────────┘  │
└─────────────────┘
```

---

## Crate Hierarchy

```
termlink-protocol   (foundation — zero dependencies on other crates)
    ▲
    │
termlink-session    (core — depends on protocol)
    ▲
    │
termlink-hub        (coordination — depends on protocol + session)
    ▲
    │
termlink (CLI)      (user interface — depends on all crates)
```

---

## 1. Protocol Layer (`termlink-protocol`)

**Purpose:** Wire format definitions shared by all crates. No business logic.

### Components

| Component | File | Purpose |
|-----------|------|---------|
| **jsonrpc** | `src/jsonrpc.rs` | JSON-RPC 2.0 types: `Request`, `Response`, `ErrorResponse`, `RpcResponse` |
| **control** | `src/control.rs` | RPC method name constants (`query.status`, `command.execute`, etc.), error codes, `KeyEntry`, `Capabilities`, `CommonParams` |
| **data** | `src/data.rs` | Binary frame format: `FrameHeader`, `FrameType` (Stdout/Stderr/Stdin/Control/Resize), `FrameFlags`, encode/decode |
| **error** | `src/error.rs` | `ProtocolError` enum for frame parsing and validation failures |
| **events** | `src/events.rs` | Typed delegation protocol schemas: `TaskDelegate`, `TaskAccepted`, `TaskProgress`, `TaskCompleted`, `TaskFailed` |

### Dual Plane Design

- **Control Plane:** JSON-RPC 2.0 over Unix sockets (newline-delimited). Used for all RPC methods (query, execute, events, KV, session management).
- **Data Plane:** Binary frames over a separate Unix socket (`.sock.data`). Used for raw terminal I/O streaming (attach, stream commands).

---

## 2. Session Layer (`termlink-session`)

**Purpose:** Core business logic. A "session" is a named process that can receive RPC commands and communicate with other sessions.

### Identity & Lifecycle

| Component | File | Purpose |
|-----------|------|---------|
| **identity** | `src/identity.rs` | `SessionId` — ULID-based unique identifier, filesystem-safe |
| **lifecycle** | `src/lifecycle.rs` | `SessionState` enum: `Initializing → Ready → Busy → Draining → Gone` |
| **registration** | `src/registration.rs` | `Registration` JSON sidecar: config, metadata, atomic write/read, heartbeat |
| **discovery** | `src/discovery.rs` | Runtime directory resolution (`$TERMLINK_RUNTIME_DIR`, `$XDG_RUNTIME_DIR`, `$TMPDIR`) |

### Session Management

| Component | File | Purpose |
|-----------|------|---------|
| **manager** | `src/manager.rs` | `Session::register()` / deregister, `list_sessions()`, `find_session()` by ID/name/tag/role/capability |
| **liveness** | `src/liveness.rs` | `is_alive()` via `kill(pid, 0)` + socket existence, `cleanup_stale()` |
| **client** | `src/client.rs` | JSON-RPC client: connect to session socket, `rpc_call()` convenience |

### Security

| Component | File | Purpose |
|-----------|------|---------|
| **auth** | `src/auth.rs` | `PeerCredentials` extraction (SO_PEERCRED / LOCAL_PEERCRED), UID check, 4-tier `PermissionScope` (Observe/Interact/Control/Execute), `method_scope()` mapping, HMAC-SHA256 capability tokens (`create_token`, `validate_token`, `generate_secret`) |

### RPC Handlers

| Component | File | Purpose |
|-----------|------|---------|
| **handler** | `src/handler.rs` | `SessionContext`, `dispatch()` / `dispatch_mut()` with read/write lock branching, all RPC implementations |
| **server** | `src/server.rs` | Control plane server: accept loop, peer credential check, per-method permission enforcement, connection handling |

### RPC Method Inventory (18 methods)

| Method | Scope | Handler |
|--------|-------|---------|
| `termlink.ping` | Observe | Returns session identity + state |
| `query.status` | Observe | PID, uptime, state, capabilities |
| `query.capabilities` | Observe | List session capabilities |
| `query.output` | Observe | Read PTY scrollback (lines/bytes) |
| `event.poll` | Observe | Poll events since cursor |
| `event.topics` | Observe | List known event topics |
| `kv.get` | Observe | Read key-value entry |
| `kv.list` | Observe | List all KV keys |
| `event.emit` | Interact | Emit event to session's bus |
| `event.broadcast` | Interact | (Hub) fan-out to multiple sessions |
| `event.collect` | Interact | (Hub) fan-in from multiple sessions |
| `command.resize` | Interact | Resize PTY terminal |
| `session.update` | Interact | Update tags/name/roles at runtime |
| `session.heartbeat` | Interact | Touch heartbeat timestamp |
| `kv.set` / `kv.delete` | Interact | Write/delete KV entries |
| `command.inject` | Control | Inject keystrokes into PTY |
| `command.signal` | Control | Send POSIX signal to child |
| `command.execute` | Execute | Run shell command via `sh -c` |
| `auth.token` | (special) | Authenticate connection, upgrade scope from token |

### Execution & PTY

| Component | File | Purpose |
|-----------|------|---------|
| **executor** | `src/executor.rs` | Shell command execution with timeout/env/cwd, input validation, signal sending |
| **pty** | `src/pty.rs` | Pseudo-terminal session: spawn shell, async read/write, resize, scrollback |
| **scrollback** | `src/scrollback.rs` | Ring buffer for PTY output (default 1 MiB), last N lines/bytes queries |

### Events & Streaming

| Component | File | Purpose |
|-----------|------|---------|
| **events** | `src/events.rs` | `EventBus` ring buffer (1024 events): emit, poll with cursor, topic filtering, monotonic sequence IDs |
| **codec** | `src/codec.rs` | Binary frame codec: `FrameReader` / `FrameWriter` for async data plane I/O |
| **data_server** | `src/data_server.rs` | Data plane server: binary frame streaming, output broadcast, input forwarding |

---

## 3. Hub Layer (`termlink-hub`)

**Purpose:** Multi-session coordination. Routes messages between sessions that don't know each other's socket paths.

### Components

| Component | File | Purpose |
|-----------|------|---------|
| **router** | `src/router.rs` | Request routing: `session.discover` (with tag/role/capability filters), `event.broadcast` (fan-out), `event.collect` (fan-in with per-session cursors), forward-to-target |
| **server** | `src/server.rs` | Hub server: Unix socket accept loop, UID check, `ShutdownHandle` for graceful shutdown, connection drain (5s timeout) |
| **pidfile** | `src/pidfile.rs` | Daemon lifecycle: write/read/validate/remove PID file, prevents double-start, cleans stale pidfiles |
| **supervisor** | `src/supervisor.rs` | Session supervision: polls liveness every 30s, auto-cleans dead sessions |

### Hub Architecture

The hub is a **stateless routing service** — it holds no persistent state:
- Session registry is file-based (reads `sessions/*.json` on every discover call)
- Event stores live in sessions, not the hub
- Crash recovery is simply "restart"

```
Client → Hub Socket → Router
                        ├── session.discover → Read sessions/*.json
                        ├── event.broadcast  → Fan-out to N session sockets
                        ├── event.collect    → Fan-in from N session sockets
                        └── other methods    → Forward to target session socket
```

---

## 4. CLI Layer (`termlink`)

**Purpose:** User-facing binary with 26 subcommands.

### Command Groups

| Group | Commands | Description |
|-------|----------|-------------|
| **Session** | `register`, `list`, `clean`, `info`, `wait` | Create, discover, and manage sessions |
| **Query** | `ping`, `status`, `output` | Read session state and terminal output |
| **Execution** | `exec`, `run`, `inject`, `signal` | Run commands, inject keystrokes, send signals |
| **Events** | `events`, `emit`, `broadcast`, `collect`, `topics` | Structured event messaging |
| **Streaming** | `attach`, `stream`, `watch`, `resize` | Real-time terminal I/O |
| **Metadata** | `tag`, `send` | Session tagging and generic messaging |
| **Hub** | `hub start`, `hub stop`, `hub status`, `discover` | Hub daemon management |
| **Token** | `token create`, `token inspect` | Capability token management |
| **Infrastructure** | `completions` | Shell completion generation |

---

## Component Dependency Graph

```
                    ┌──────────────┐
                    │   CLI (26    │
                    │  commands)   │
                    └──┬───────┬───┘
                       │       │
              ┌────────┘       └────────┐
              ▼                         ▼
    ┌──────────────────┐     ┌───────────────────┐
    │       Hub         │     │     Session        │
    │  router           │     │  manager           │
    │  server           │────▶│  handler           │
    │  pidfile          │     │  server             │
    │  supervisor       │     │  auth               │
    └────────┬─────────┘     │  events             │
             │               │  executor           │
             │               │  pty                │
             │               │  scrollback         │
             │               │  codec              │
             │               │  data_server        │
             │               │  client             │
             │               │  liveness           │
             │               │  discovery          │
             │               │  identity           │
             │               │  lifecycle          │
             │               │  registration       │
             │               └──────────┬──────────┘
             │                          │
             └──────────┬───────────────┘
                        ▼
              ┌──────────────────┐
              │    Protocol       │
              │  jsonrpc          │
              │  control          │
              │  data             │
              │  error            │
              │  events           │
              └──────────────────┘
```

---

## Security Model

### Three Phases

1. **Phase 1 (implemented, T-077):** UID-based authentication — extract peer UID via socket credentials, reject cross-user connections
2. **Phase 2 (implemented, T-078/T-084):** 4-tier permission scoping — `method_scope()` maps each RPC method to a required scope, checked before dispatch
3. **Phase 3 (implemented, T-086/T-087/T-088):** Capability tokens — HMAC-SHA256 signed tokens for fine-grained multi-agent authorization. Sessions with `token_secret` in registration default to Observe scope; clients authenticate via `auth.token` to upgrade. Legacy sessions (no secret) retain Execute scope for backward compatibility.
4. **Phase 4 (implemented, T-090):** Command allowlist — optional prefix-based allowlist for `command.execute`. Sessions with `allowed_commands` in registration restrict which commands can be executed, even for clients with Execute scope. Defense-in-depth against command injection (G-001).

### Permission Hierarchy

```
Execute (3)  ─── can do everything
   ▲
Control (2)  ─── + command.inject, command.signal
   ▲
Interact (1) ─── + event.emit, session.update, kv.set, ...
   ▲
Observe (0)  ─── ping, query.*, event.poll, kv.get (read-only)
```

---

## Runtime Layout

```
$TERMLINK_RUNTIME_DIR/          # /tmp/termlink-$UID or $XDG_RUNTIME_DIR/termlink
├── hub.sock                     # Hub control plane socket
├── hub.pid                      # Hub daemon pidfile
└── sessions/
    ├── {session-id}.sock        # Session control plane socket
    ├── {session-id}.sock.data   # Session data plane socket
    └── {session-id}.json        # Registration metadata (name, PID, state, tags, ...)
```

---

## Test Coverage

| Crate | Tests | Coverage Focus |
|-------|-------|----------------|
| termlink-protocol | 21 | JSON-RPC parsing, frame encode/decode, error types |
| termlink-session | 131 | Handlers (all 18 RPC methods), events, PTY, liveness, auth (tokens), server |
| termlink-hub | 28 | Router (discover, broadcast, collect, forward), server, pidfile, supervisor |
| termlink (CLI) | 15+4 | Integration tests (register, ping, exec, events, KV), interactive TTY tests |
| **Total** | **211** | |

---

## Key Design Patterns

- **Dual plane:** Control (JSON-RPC) + Data (binary frames) — separates command/query from streaming I/O
- **Write-lock dispatch:** `dispatch_mut()` + `needs_write()` — only 3 methods need write lock, rest use read lock
- **File-based registry:** Session metadata stored as JSON sidecar files, not in-memory — survives hub crash
- **Ring buffer events:** `EventBus` with monotonic sequence IDs, topic filtering, cursor-based polling
- **Stateless hub:** Hub reads registry from disk on every call — no cache, no stale state, crash = restart
- **Graceful shutdown:** `ShutdownHandle` pattern with `tokio::sync::watch`, 5-second connection drain
- **Cross-platform auth:** `SO_PEERCRED` (Linux) / `LOCAL_PEERCRED` + `LOCAL_PEERPID` (macOS)
