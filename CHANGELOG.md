# Changelog

All notable changes to TermLink are documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2026-03-25

### Added
- **MCP server** (`termlink-mcp` crate) with 26 tools for AI agent integration
  - Core: `ping`, `list_sessions`, `status`, `discover`, `exec`, `run`
  - PTY: `output`, `inject`, `interact`, `resize`
  - Events: `emit`, `emit_to`, `event_poll`, `broadcast`, `wait`, `request`
  - Metadata: `tag`, `kv_set`, `kv_get`, `kv_list`, `kv_del`
  - Orchestration: `spawn`, `signal`
  - Self-healing: `doctor`, `clean`
- MCP resources: session list + session detail (read-only)
- MCP prompts: `debug_session`, `session_overview`, `orchestrate`
- `termlink doctor` CLI command with 6 health checks (runtime dir, sessions, hub, sockets, version)
- `termlink doctor --fix` for auto-remediation of stale sessions, orphaned sockets, stale hub
- 41 MCP integration tests, 472 total tests

## [0.7.0] - 2026-03-23

### Added
- Hub `orchestrator.route` RPC — discover, delegate, relay to specialist sessions
- Bypass registry — Tier 3 operationalized for local execution of known-safe commands
  - Atomic file writes, file locking, denylist, mutation awareness
  - Transport failure tracking, infra vs command failure distinction
  - Pattern invalidation signals, full cache busting
- Circuit breaker for dead session failover optimization
- Live orchestration test harness — 13 E2E scenarios with real sessions
- Interactive session picker — prompt when no target given in CLI commands
- Fix attach output freezing (delta exceeding buffer size)

## [0.6.0] - 2026-03-14

### Added
- **Remote TCP hub** — `termlink hub --tcp <addr>` for cross-machine communication
- TLS with auto-generated self-signed certificates
- Token-based authentication for TCP connections (`hub.auth` RPC)
- Remote commands: `remote ping`, `remote list`, `remote status`, `remote inject`, `remote exec`, `remote events`, `remote send-file`
- Hub profile management (`remote profile add/list/remove/show`)
- `termlink mirror` — read-only terminal mirroring via data plane
- `register --self` — event-only endpoint for existing processes
- Push messaging — `event.emit_to` RPC for direct session-to-session events
- Route cache with confidence decay and TTL
- Negotiation protocol types and state machine
- Template cache (local, shared, schema hash invalidation)
- Trust assessment (3-axis qualitative supervision scoring)

## [0.5.0] - 2026-03-09

### Added
- `termlink spawn` — open new terminal with session auto-registration
- `termlink request` — emit + wait request-reply pattern
- CLI integration test harness — 18 end-to-end tests
- Interactive TTY tests via rexpect (attach and stream)
- 156 total tests

### Fixed
- Events `--since` off-by-one (all events visible by default)

## [0.4.0] - 2026-03-09

### Added
- Session event system — structured pub/sub with `EventBus`, RPC, and CLI
- `termlink watch` — real-time event polling across sessions
- Hub event routing — `broadcast` and `collect` across sessions
- Session tags — tag-based organization with runtime updates
- Session metadata persistence — `session.update` writes to disk
- `termlink discover` — filtered session queries by tag, role, capability, name
- `termlink clean` — reap stale sessions from runtime directory
- `termlink wait` — block until session emits matching event
- `termlink run` — ephemeral session with command execution
- `termlink collect` — fan-in events from multiple sessions via hub
- `termlink topics` — list event topics from sessions
- `termlink info` — runtime diagnostics and system overview
- Session KV store — per-session key-value metadata via RPC

## [0.3.0] - 2026-03-08

### Added
- **PTY manager** with scrollback buffer — bidirectional terminal I/O
- `query.output` and `command.inject` wired to PTY sessions
- `--shell` mode for register (spawns shell with PTY)
- Hub server — Unix socket listener with discover + forward routing
- `termlink output` — read terminal output from PTY sessions
- `termlink inject` — send keystrokes to PTY sessions
- `termlink attach` — interactive PTY session with live I/O
- `termlink signal` — send signals to session processes
- **Data plane** — async frame codec and binary streaming server
- `termlink stream` — real-time data plane attach
- Stream enhancements — SIGWINCH resize forwarding, scrollback catch-up
- Data plane availability reported in session status and discovery
- `command.resize` RPC method and CLI command

## [0.2.0] - 2026-03-08

### Added
- Session registration with Unix socket listener
- Liveness detection (process existence checks)
- Session listing with state tracking
- Control plane JSON-RPC 2.0 handler over Unix sockets
- `termlink register` — register a new session
- `termlink list` — list active sessions
- `termlink ping` — check if a session is alive
- `termlink status` — get detailed session status
- Hub message routing and `termlink send`
- Command execution (`command.execute`), key injection, signal handlers
- End-to-end integration tests — 7 tests, multi-session communication

## [0.1.0] - 2026-03-08

### Added
- Rust workspace scaffold — 4 crates (`termlink-protocol`, `termlink-session`, `termlink-hub`, `termlink-cli`)
- Binary frame codec with type-safe protocol
- Session identity system (ULID + display name)
- 19 initial tests
