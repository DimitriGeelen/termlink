![TermLink](header.svg)

A coordination substrate for parallel AI agents — a hub-mediated, durable
append-log message bus with terminal endpoints.

TermLink lets a fleet of agents (and humans) **discover each other, exchange durable
messages, claim work, and control terminal sessions** across one or many machines.
Sessions register with a hub; every other session can then message it on append-log
channel topics, ping it, execute commands, stream its terminal output, or inject
keystrokes — all from the CLI or via 270+ MCP tools.

It began as a cross-terminal session-control tool and grew into the coordination
layer for multi-agent parallel execution (see
[docs/architecture/parallel-execution-substrate.md](docs/architecture/parallel-execution-substrate.md)
— the authoritative statement of the substrate design and its invariants).

## Use Cases

- **AI agent coordination** — presence heartbeats, durable DM threads, doorbell
  push-wake, claim/lease work-stealing across a fleet of Claude Code workers
- **Remote observation** — attach to a running session from another terminal and mirror its TUI
- **Process orchestration** — dispatch commands across sessions, wait for completion signals
- **Cross-machine access** — TCP hub bridges sessions across SSH tunnels or LAN

## Guarantees

What the substrate promises (and, honestly, what it does not):

- **Ordering** — every channel topic is an append-only log with per-topic
  monotonically increasing offsets. Readers replay from any offset.
- **Durability** — hub-side, reader-oriented: a posted message is durable in the
  topic log under the hub's retention policy (`days` / `messages` / `latest` /
  `latest-per-cv-key`; nothing is pruned until an explicit `channel sweep`).
  Durability is *reader-side* — the hub does not guarantee a recipient consumed
  a message; use receipts/acks for that.
- **Exactly-once post** — client retries are absorbed by hub-side
  `(sender, client_msg_id)` dedupe; the CLI's offline queue persists unsent posts
  across hub blips and replays them safely.
- **Delivery confirmation is explicit, not implicit** — `post --await-ack` writes a
  durable obligation; `channel awaiting-ack` surfaces sends nobody confirmed.
  Nothing is silently assumed delivered.
- **Topology** — strict star: spokes talk to a hub, never to each other. Channel
  topics are **per-hub state; there is no inter-hub federation primitive.**
  Cross-hub visibility is always explicit, client-driven cross-posting
  (`channel post --hub <addr>`). A shared topic name on two hubs is two topics.

## Trust Model

Two transports, two non-equivalent trust anchors — know which one you're on:

- **Same host (Unix sockets):** kernel UID trust — only same-user peers connect.
- **Cross host (TCP):** persistent 32-byte HMAC hub secret + TOFU-pinned TLS
  certificate. Rotation is a first-class operational event with detection
  (`fleet verify`, `fleet doctor`) and declarative heal (`bootstrap_from` per
  profile, `fleet reauth`).
- **Authorization is coarse:** authenticated callers get permission-scoped access
  (Observe/Interact/Control/Execute + capability tokens), but there is no
  per-user/per-agent authorization model beyond that — treat every authenticated
  peer as trusted. Suitable for a single-operator fleet; not yet for adversarial
  multi-tenancy.

## Quick Start

```bash
# One-liner install (any Linux or macOS, no toolchain required)
curl -fsSL https://raw.githubusercontent.com/DimitriGeelen/termlink/main/install.sh | sh

# Or via Homebrew (macOS preferred)
brew tap DimitriGeelen/termlink
brew install termlink

# Or build from source (requires Rust toolchain)
cargo install --git https://github.com/DimitriGeelen/termlink.git termlink --force

# Start a named session (opens a shell)
termlink spawn --name demo --tags "test" --shell

# From another terminal:
termlink list                          # See all sessions
termlink ping demo                     # Check it's alive
termlink exec demo "echo hello"        # Run a command
termlink pty output demo --lines 5     # Read recent output
termlink pty inject demo "ls" --enter  # Send keystrokes
termlink pty attach demo               # Full TUI mirror (bidirectional)
```

## Architecture

TermLink uses a **dual-plane design**:

- **Control plane** — JSON-RPC 2.0 over Unix sockets for commands, queries, and events
- **Data plane** — binary frames over a separate socket for raw terminal I/O streaming

```
┌──────────────────────────────────────────────────┐
│                   CLI (termlink)                   │
│  30 commands: register, list, ping, exec, ...     │
└───────┬───────────────────┬───────────────────────┘
        │ direct            │ via hub
        v                   v
┌────────────────┐  ┌─────────────────────────────┐
│   Session A     │  │            Hub               │
│  Control Sock   │  │  Router / Discover / Forward │
│  Data Sock      │  │  Supervisor (30s sweep)      │
│  Event Bus      │  │  TCP listener (optional)     │
│  PTY / Exec     │  └─────────────────────────────┘
│  KV Store       │
└────────────────┘         Session B, C, ...
```

Four crates, layered bottom-up:

| Crate | Purpose |
|-------|---------|
| `termlink-protocol` | Wire format — JSON-RPC types, binary frames, event schemas |
| `termlink-session` | Core — session lifecycle, RPC handlers, PTY, auth, discovery |
| `termlink-hub` | Coordination — routing, broadcast, collect, TCP transport |
| `termlink-mcp` | MCP server — expose TermLink as structured tools for AI agents |
| `termlink` (CLI) | User interface — 30 commands wrapping the above |

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full technical reference.

## CLI Commands

| Group | Commands | Purpose |
|-------|----------|---------|
| **Session** | `register`, `spawn`, `list`, `status`, `info`, `ping`, `clean` | Lifecycle and discovery |
| **Execution** | `exec`, `interact`, `run`, `dispatch` | Run commands on sessions, parallel dispatch |
| **PTY** | `pty output`, `pty inject`, `pty attach`, `pty stream`, `pty resize`, `mirror` | Terminal I/O and mirroring |
| **Events** | `event emit`, `event poll`, `event watch`, `event broadcast`, `event collect`, `event wait`, `event topics` | Inter-session signaling |
| **Discovery** | `discover`, `tag`, `kv` | Find sessions by tag/role/capability, metadata |
| **Agent** | `agent ask`, `agent listen`, `agent negotiate` | Typed agent-to-agent communication |
| **Files** | `file send`, `file receive` | Chunked file transfer between sessions |
| **Hub** | `hub start`, `hub stop`, `hub status` | Multi-session coordination |
| **Remote** | `remote ping`, `remote list`, `remote exec`, `remote profile` | Cross-machine via TCP hub |
| **Security** | `token create`, `token inspect` | Capability tokens (HMAC-SHA256) |
| **Tools** | `doctor`, `vendor`, `mcp serve` | Health checks, binary vendoring, MCP server |
| **Other** | `send`, `signal`, `request`, `completions`, `version` | Low-level RPC, shell completions |

Every command supports `--json` for machine-readable output. Run `termlink <command> --help` for details.

## MCP Server (AI Agent Integration)

TermLink ships a built-in [Model Context Protocol](https://modelcontextprotocol.io/) server with **270+ tools** (276 at last count — trimming in progress, see arc `mcp-slimming`), enabling AI agents (Claude Code, etc.) to coordinate through the substrate and orchestrate terminal sessions programmatically.

### Setup

```bash
# Vendor into any project — auto-configures .claude/settings.local.json
termlink vendor
```

### Core tools (sample of 276)

| Category | Tools | Purpose |
|----------|-------|---------|
| **Core** | `ping`, `list_sessions`, `status`, `discover`, `exec`, `run`, `spawn` | Session lifecycle, execution |
| **PTY** | `output`, `inject`, `interact`, `resize`, `pty_mode` | Terminal I/O and mode detection |
| **Events** | `emit`, `emit_to`, `event_poll`, `event_subscribe`, `broadcast`, `wait`, `request`, `collect`, `topics` | Inter-session signaling and fan-in |
| **Metadata** | `tag`, `kv_set`, `kv_get`, `kv_list`, `kv_del` | Session tags and key-value store |
| **Files** | `file_send` | Chunked file transfer between sessions |
| **Agent** | `agent_ask` | Typed agent-to-agent request/response |
| **Orchestration** | `signal` | Process signals |
| **Self-healing** | `doctor`, `clean` | Health checks, stale session cleanup |
| **Hub** | `hub_start`, `hub_stop`, `hub_status` | Hub lifecycle management |
| **Diagnostics** | `info`, `dispatch_status` | Runtime info, dispatch manifest |

All tools are prefixed with `termlink_` (e.g., `termlink_ping`). The server also exposes 2 resources and 3 prompts.

## Common Workflows

### Dispatch parallel workers

```bash
# Atomic dispatch: spawn 3 workers, collect results, cleanup — one command
termlink dispatch --count 3 --timeout 300 -- bash -c 'echo "Worker done"; termlink event emit $TERMLINK_WORKER_NAME task.completed'

# With git worktree isolation (each worker gets its own branch)
termlink dispatch --count 3 --isolate --auto-merge --timeout 300 -- make build

# Check dispatch status
termlink dispatch-status --json
```

<details><summary>Manual alternative (without dispatch command)</summary>

```bash
for i in 1 2 3; do
  termlink spawn --name "worker-$i" --tags "worker" --backend auto \
    -- bash -c "echo 'Worker $i done'; termlink event emit worker-$i worker.done"
done
for i in 1 2 3; do
  termlink event wait "worker-$i" worker.done --timeout 60
done
```

</details>

### Remote session observation

```bash
# Start Claude Code in a TermLink session
scripts/tl-claude.sh --name claude-master

# From another terminal (or machine via SSH tunnel):
termlink pty attach claude-master          # Full TUI mirror
termlink pty stream claude-master          # Low-latency binary stream
termlink pty output claude-master --strip-ansi --lines 20  # Text snapshot
termlink pty inject claude-master "/help" --enter          # Send input
```

### Hub for multi-session coordination

```bash
# Start the hub
termlink hub start

# Sessions auto-discover each other
termlink discover --tag worker --json    # Find all workers
termlink event broadcast hub task.ready '{"payload": "go"}'  # Fan-out
termlink event collect hub task.done --count 3 --timeout 120  # Fan-in
```

## Spawn Backends

`termlink spawn` auto-detects the best backend for your platform:

| Backend | Platform | How |
|---------|----------|-----|
| `terminal` | macOS | Opens a new Terminal.app window via osascript |
| `tmux` | macOS, Linux | Creates a tmux session (headless, attach with `tmux attach`) |
| `background` | macOS, Linux | Daemonizes with `setsid` (no visible terminal) |
| `auto` | Any | Picks `terminal` on macOS GUI, `tmux` if available, else `background` |

Override with `--backend tmux` or set `TL_DISPATCH_BACKEND=tmux`.

## Platform Support

| | macOS | Linux | Windows |
|-|-------|-------|---------|
| Core binary | Yes | Yes | No |
| PTY operations | Yes | Yes | No |
| Terminal.app spawn | Yes | — | — |
| tmux spawn | Yes | Yes | — |
| TCP hub | Yes | Yes | — |

TermLink requires POSIX PTY support (`openpty`, `fork`, `tcgetattr`). Windows is not supported.

## Installation

### Homebrew (recommended for macOS/Linux)

```bash
brew tap DimitriGeelen/termlink
brew install termlink
termlink --version
```

### From source (for development/debugging)

**Requirements:**
- Rust toolchain (edition 2024) — install via [rustup.rs](https://rustup.rs)
- macOS or Linux
- Optional: tmux (for tmux spawn backend)

```bash
# From GitHub
cargo install --git https://github.com/DimitriGeelen/termlink.git termlink --force

# From local clone
cargo install --path crates/termlink-cli --force

# Verify
termlink --version
termlink info
```

The binary installs to `~/.cargo/bin/termlink`. The runtime directory is auto-created at `$TERMLINK_RUNTIME_DIR`, `$XDG_RUNTIME_DIR/termlink`, or `$TMPDIR/termlink-$UID`.

## Security Model

Four layers, progressively stricter:

1. **UID-based auth** — Unix socket peer credentials; only same-user sessions can connect
2. **Permission scopes** — methods classified as Observe/Interact/Control/Execute
3. **Capability tokens** — HMAC-SHA256 tokens granting specific scopes to specific sessions
4. **Command allowlist** — sessions can restrict which commands are executable

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## Testing

```bash
cargo test --workspace           # All unit + integration tests
./tests/e2e/run-all.sh           # End-to-end tests (levels 1-7)
```

## Scripts

| Script | Purpose |
|--------|---------|
| `scripts/tl-claude.sh` | Launch Claude Code inside a TermLink session for remote access |
| `scripts/tl-dispatch.sh` | Dispatch parallel Claude Code workers in real terminals |

Both support `--help`.

## License

MIT
