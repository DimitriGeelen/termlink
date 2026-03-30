# Changelog

All notable changes to TermLink are documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] - 2026-03-26

### Added
- **`termlink vendor`** — vendor TermLink binary into a project directory for path isolation
  - Auto-configures MCP server in `.claude/settings.local.json`
  - Auto-creates/updates `.gitignore` for vendored binary
  - `--status`, `--check`, `--dry-run`, `--json` flags
- **`termlink push`** — one-command cross-project file delivery with PTY notification
- **File transfer** — `file send` / `file receive` for chunked file transfer between sessions
- **Agent protocol** — `agent ask`, `agent listen`, `agent negotiate` for typed agent-to-agent communication
- **Git-derived versioning** — `build.rs` reads version from git tags (`v0.9.0` → exact, N commits after → `0.9.N`)
- `termlink version` subcommand with commit hash and build target
- `--json` output added to all 30 CLI commands for scripting
- `--timeout` flag on `ping`, `status`, and `send` commands
- `--count` flag on `list` for quick session counting
- `--tag`, `--name`, `--role` filters on `list`
- `--check` flag on `info` for health check scripting
- `--quiet` flag on `register` to suppress startup output
- `--no-header` flag on `remote profile list`
- `tag` command shows current tags when called without modification flags
- `pty.mode` RPC — query terminal mode (canonical, echo, raw, alternate screen)
- Linux aarch64 added to release workflow (4 platform builds: macOS arm64/x86_64, Linux x86_64/aarch64)
- Homebrew formula updated with 4 platform variants
- E2E test runner (`tests/e2e/run-all.sh`) — discovers and runs level scripts with summary
- 585 total tests (from 474) — protocol frame edge cases, EventBus boundary tests, registration serde, CLI error paths, token roundtrip

### Changed
- **Release profile optimization** — LTO, strip, single codegen-unit reduces binary from 18MB to 12MB (33%)
- All JSON responses now include `ok: true/false` field for consistent error handling
- JSON error exit uses `json_error_exit()` helper — fixes stdout buffering issues
- Updated 17 Cargo dependencies to latest compatible versions
- ARCHITECTURE.md updated — MCP crate in hierarchy, 12 command groups, 30 commands, all module component tables current
- README updated — 30 commands, MCP crate in architecture table

### Fixed
- `vendor --json` no longer leaks status messages into JSON output
- Shell completions no longer panic on missing target argument
- `ping --json` timeout and error responses now use `ok: false` consistently
- `remote exec --json` propagates non-zero exit codes correctly
- `pty interact --json` exit code propagation and `ok` field accuracy
- `push` command heredoc injection vulnerability — now uses base64 encoding
- 3 unsafe `.unwrap()` calls in CLI/MCP replaced with proper error handling
- All clippy warnings resolved (zero warnings workspace-wide)

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
- `termlink dispatch` — atomic spawn+tag+collect for multi-worker orchestration
- `session.exited` lifecycle events — hub supervisor emits before cleanup, enabling crash detection
- 41 MCP integration tests, 474 total tests

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
