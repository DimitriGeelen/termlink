# TermLink Status Report — 2026-03-14

## What We've Built

TermLink is a cross-terminal session communication system in Rust. It lets
terminal sessions (shells, agents, REPLs) discover each other, exchange
messages, execute commands, stream output, and coordinate work — all over
structured JSON-RPC and binary streaming protocols.

### Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Session A   │     │     Hub     │     │  Session B   │
│  (agent)     │◄───►│  (router)   │◄───►│  (shell)     │
│              │     │             │     │              │
│ Control ─────┤     │ Broadcast   │     ├───── Control │
│ (JSON-RPC)   │     │ Collect     │     │ (JSON-RPC)   │
│              │     │ Forward     │     │              │
│ Data ────────┤     │ Discover    │     ├──────── Data │
│ (binary)     │     │             │     │ (binary)     │
└─────────────┘     └─────────────┘     └─────────────┘
       │                                       │
       └── Unix socket / TCP ──────────────────┘
```

**5 crates, ~8,000 LOC, 253 tests passing.**

### Crate Breakdown

| Crate | Purpose | LOC | Tests |
|-------|---------|-----|-------|
| `termlink-protocol` | Wire types: JSON-RPC, binary frames, transport addresses, events | ~1,200 | 20+ |
| `termlink-session` | Session lifecycle, RPC handlers, PTY, streaming, auth | ~4,500 | 150+ |
| `termlink-hub` | Stateless message router, supervisor, discovery | ~800 | 50+ |
| `termlink-cli` | 26 CLI subcommands | ~1,200 | 30+ |
| `termlink-test-utils` | Shared test helpers (TestDir, ProcessGuard) | ~300 | — |

## What's Working

### Session Management
- **Register** sessions with name, capabilities, roles, tags
- **Discover** sessions by tag, role, capability, or name pattern
- **Lifecycle** state machine: Initializing → Ready → Busy → Draining → Terminated
- **Heartbeat** + automatic stale session cleanup
- **PID-based liveness** detection (Unix) + TCP connect probe

### Communication (18 RPC Methods)
- `termlink.ping` / `termlink.info` — health check, session metadata
- `query.status` / `query.capabilities` / `query.output` — inspect session state
- `command.execute` — run commands with timeout, env, cwd; returns stdout+stderr+exit code
- `command.inject` — send keystrokes to PTY
- `event.emit` / `event.poll` / `event.topics` — pub/sub event bus with cursors
- `session.signal` / `session.resize` — PTY control
- `session.update` — modify tags, roles, capabilities at runtime
- `kv.get` / `kv.set` / `kv.delete` / `kv.list` — per-session key-value store
- `auth.token` — HMAC capability token authentication

### Hub Coordination
- **Broadcast** — fan-out messages to multiple sessions (by tag/role filter)
- **Collect** — fan-in event aggregation with cursor tracking
- **Forward** — route to specific session by name
- **Discover** — query all registered sessions with filters
- **Supervision** — 30s loop, PID checks, stale cleanup, graceful shutdown

### Security (4 Layers)
1. **UID authentication** — socket peer credentials (SO_PEERCRED)
2. **Scoped permissions** — Observe/Interact/Control/Execute per method
3. **HMAC tokens** — fine-grained capability tokens for multi-agent auth
4. **Command allowlist** — defense-in-depth for `command.execute`

### Transport Layer
- **Unix sockets** — default, zero-config local communication
- **TCP transport** — `TcpTransport` + `TcpLivenessProbe` for cross-machine
- **SSH tunneling** — works transparently, zero code changes needed
- **Transport abstraction** — `TransportAddr` enum, `Transport` trait, pluggable

### Agent Orchestration
- **Spawn** — launch specialist agents in new terminals
- **Request** — emit-and-wait delegation pattern
- **Watch** — real-time event polling
- **Task delegation events** — TaskDelegate/Accepted/Progress/Completed/Failed schemas
- **Agent mesh** — dispatch.sh, worktree isolation, auto-commit, merge orchestration

### CLI (26 Commands)
```
termlink register    termlink list        termlink ping
termlink status      termlink info        termlink exec
termlink run         termlink output      termlink inject
termlink attach      termlink stream      termlink signal
termlink resize      termlink emit        termlink broadcast
termlink collect     termlink topics      termlink watch
termlink send        termlink tag         termlink discover
termlink clean       termlink wait        termlink hub {start|stop|status}
termlink token {create|inspect}
termlink spawn       termlink request
```

### Data Plane (Binary Streaming)
- Frame codec: magic + version + type + length + payload
- Stream types: stdout, stderr, stdin, resize, close
- Async bidirectional streaming over separate socket
- Scrollback buffer (1 MiB default) for output queries

## What's Tested

- **253 tests** passing (`cargo test --workspace`)
- Unit tests for all protocol types, serialization roundtrips
- Async tests for session lifecycle, RPC handlers, event bus
- Integration tests: multi-session ping, cross-session exec, discovery
- CLI integration tests: register, list, ping, status, exec, events
- E2E tests: full session lifecycle with hub coordination
- Transport tests: Unix + TCP connect/bind/accept, liveness probes
- Security tests: UID auth, scope enforcement, token validation, allowlist

## What's Available for Framework Integration

### Today (Zero Changes Needed)
1. `termlink exec <session> -- fw doctor` — run framework commands in another session
2. `termlink output <session>` — read the scrollback (stdout) from that session
3. `termlink emit` / `termlink watch` — coordinate between framework agents
4. SSH tunneling for cross-machine framework testing

### With Minimal Work
1. TCP transport for container/CI environments (T-133 done, needs human validation)
2. HTTP REST gateway for non-Rust/non-CLI clients
3. Task delegation CLI for formalized agent-to-agent workflows
