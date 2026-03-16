# Framework Pickup Prompt — TermLink Integration

> Paste everything below the line into a Claude Code session in the framework project.

---

I want to integrate TermLink into the framework. TermLink is a cross-terminal session communication tool we built — it's our project, hosted at `https://onedev.docker.ring20.geelenandcompany.com/termlink`. It's already installed at `/Users/dimidev32/.cargo/bin/termlink` (v0.1.0, 26 commands), built from source via `cargo install --path .`. Battle-tested with 264 passing tests across 4 crates.

**Repo (GitHub):** `https://github.com/DimitriGeelen/termlink`
**Repo (OneDev):** `https://onedev.docker.ring20.geelenandcompany.com/termlink`
**Install:** `git clone https://github.com/DimitriGeelen/termlink.git && cd termlink && cargo install --path crates/termlink-cli`
**Binary:** `/Users/dimidev32/.cargo/bin/termlink`

## What TermLink already provides (DO NOT rebuild these)

TermLink is a Rust binary with 26 commands. Run `termlink --help` to see them all. Key ones:

```
termlink register     # Register a session (--shell for interactive, --name, --tag)
termlink list         # List sessions (--json)
termlink interact     # Run command in PTY session, wait for completion, return output (--json)
termlink discover     # Find sessions by tag/role/name (--json)
termlink event emit   # Emit event to session
termlink event wait   # Wait for event on session (--topic, --timeout)
termlink event broadcast  # Fan-out to all listeners
termlink pty inject   # Send input to PTY (fire-and-forget, --enter)
termlink pty output   # Read terminal output (--strip-ansi)
termlink status       # Session details (--json)
termlink spawn        # Spawn command in new terminal with session registration (--backend auto|terminal|tmux|background)
termlink hub start    # Start hub server (--tcp for cross-machine)
termlink run          # Ephemeral session: register, execute, deregister
termlink clean        # Remove stale session registrations
termlink kv           # Per-session key-value store
termlink token        # Capability-based auth tokens
```

Every query command supports `--json`. Exit codes are semantic (0=success, 1=timeout/not-found).

TermLink also has a working **dispatch script** that spawns `claude -p` workers in real Terminal.app windows. This script is tested and working — adapt it, don't rewrite it. The full source is included below.

## What the framework needs to build (Phase 0)

### 1. `fw doctor` check

Add to the optional tools section of `doctor.sh`:

```bash
if command -v termlink >/dev/null 2>&1; then
    version=$(termlink --version 2>/dev/null | head -1)
    echo -e "  ${GREEN}OK${NC}  TermLink ($version)"
else
    echo -e "  ${YELLOW}WARN${NC}  TermLink not installed (cargo install termlink)"
    warnings=$((warnings + 1))
fi
```

WARN not FAIL — TermLink is optional. Include install hint.

### 2. Create `agents/termlink/AGENT.md`

This should document when and how to use TermLink from framework agents. Include the primitives table:

| Command | Purpose | Framework Use |
|---------|---------|---------------|
| `termlink interact <session> <cmd> --json` | Run command, get structured output | **Star primitive.** `fw termlink exec` wraps this. |
| `termlink discover --json` | Find sessions by tag/role/name | Worker discovery |
| `termlink event emit/wait/poll` | Inter-session signaling | Coordination backbone |
| `termlink event broadcast <topic> <data>` | Fan-out to all listeners | Multi-worker notification |
| `termlink list --json` | List all sessions | Status overview |
| `termlink status <session> --json` | Session details | Health check |
| `termlink pty output <session> --strip-ansi` | Read terminal output | Log observation |
| `termlink pty inject <session> --enter` | Send input (fire-and-forget) | Long-running command start |
| `termlink register --shell --name X --tag Y` | Create named session | Tagged session lifecycle |
| `termlink hub start [--tcp ADDR]` | Start hub (optional TCP) | Cross-machine coordination |

### 3. Create `agents/termlink/termlink.sh`

This is a **thin wrapper** around the `termlink` binary. It adds framework-specific concerns (task-tagging, budget checks, cleanup tracking) but delegates all real work to the binary. Subcommands:

```
fw termlink check                        # Is termlink on PATH? Print version. Exit 0/1.
fw termlink spawn --task T-XXX [--name N] [--backend auto|terminal|tmux|background]
                                          # Spawn tagged session — delegates to `termlink spawn --backend`
fw termlink exec <session> <command>      # Wraps `termlink interact --json`
fw termlink status                        # Wraps `termlink list --json` + annotates with task tags
fw termlink cleanup                       # Per-backend cleanup (tmux kill-session / kill PID / 3-phase Terminal.app)
fw termlink dispatch --task T-XXX --name <worker> --prompt "..."
                                          # Spawn claude -p worker in real terminal
fw termlink wait --name <worker> [--all]  # Wait for worker.done event
fw termlink result --name <worker>        # Read worker result file
```

**Backend selection:** `termlink spawn` auto-detects the best backend: macOS GUI → Terminal.app, tmux available → tmux, fallback → background PTY. Override with `--backend tmux` or env var `TL_DISPATCH_BACKEND=tmux`. This means the framework works on headless Linux servers, not just macOS desktops.

### 4. `fw termlink` route in fw CLI

```bash
termlink)
    exec "$AGENTS_DIR/termlink/termlink.sh" "$@"
    ;;
```

### 5. CLAUDE.md section

Add a TermLink section:
- When to use: self-test, parallel dispatch, observation, remote control
- Available via: `fw termlink <subcommand>` or raw `termlink` CLI
- Budget rule: don't spawn new sessions when context > 60%
- Cleanup rule: always `fw termlink cleanup` before session end
- The `termlink` binary does the heavy lifting — the framework wrapper adds task context

## CRITICAL: Per-Backend Cleanup Protocol

Cleanup is backend-aware. The dispatch script records which backend was used in `meta.json` and stores backend-specific tracking files:

| Backend | Tracking file | Cleanup method |
|---------|--------------|----------------|
| **tmux** | `tmux_session` (e.g., `tl-worker-1`) | `tmux kill-session -t <name>` |
| **background** | `pid` | `kill <pid>` |
| **terminal** (macOS) | `window_id` | 3-phase osascript (see below) |

### Terminal.app 3-Phase Cleanup (macOS only)

**Never close Terminal.app windows directly.** Direct close kills interactive sessions and leaves orphaned processes.

1. **Phase 1 — Kill child processes via TTY** (spare login/shell)
2. **Phase 2 — Exit shells gracefully** (`do script "exit"`)
3. **Phase 3 — Close remaining by tracked window ID** (fallback)

The full 3-phase implementation is in `tl-dispatch.sh cmd_cleanup()` below.

## Reference Implementation: tl-dispatch.sh (ADAPT THIS, DON'T REWRITE)

This is the working, tested dispatch script from the TermLink project. The `dispatch`, `wait`, `result`, and `cleanup` subcommands in `termlink.sh` should adapt this code directly. Key patterns to preserve:

- **`termlink spawn --backend auto`** for cross-platform session creation (Terminal.app, tmux, or background PTY)
- **`TL_DISPATCH_BACKEND` env var / `--backend` flag** for backend override
- **Per-backend cleanup** — tmux kill-session, kill PID, or 3-phase osascript depending on backend
- **`termlink pty inject --enter`** for fire-and-forget command injection (NOT `interact` — claude takes minutes)
- **Background process + kill watchdog** for timeout (macOS has no `timeout` command)
- **`termlink event emit worker.done`** for completion signaling
- **File-based result collection** (`/tmp/tl-dispatch/<worker>/result.md`)

See `scripts/tl-dispatch.sh` in the TermLink repo for the full implementation (423 lines). Key highlights of the current version:

- **Multi-backend spawn**: `cmd_spawn()` delegates to `termlink spawn --backend "$backend"` instead of calling osascript directly
- **Backend override**: `--backend` flag per-worker or `TL_DISPATCH_BACKEND` env var (default: `auto`)
- **Per-backend tracking**: Records `tmux_session`, `pid`, or `window_id` files for cleanup
- **Per-backend cleanup**: `cmd_cleanup()` iterates workers, kills by tmux session name, PID, or 3-phase osascript depending on what tracking files exist
- **meta.json includes `"backend"` field** for post-mortem analysis

Clone the repo and read `scripts/tl-dispatch.sh` directly — it's the source of truth.

## What's already built in TermLink (don't rebuild — just use)

**Repo (GitHub):** `https://github.com/DimitriGeelen/termlink`
**Repo (OneDev):** `https://onedev.docker.ring20.geelenandcompany.com/termlink`

- **26 CLI commands** — all with `--json`, semantic exit codes
- **`interact --json`** — run command, wait, return `{output, exit_code, elapsed_ms, marker_found}`
- **Event system** — emit/wait/poll/broadcast/topics/collect
- **TCP hub** — `termlink hub start --tcp 0.0.0.0:9100` for cross-machine
- **Remote session store** — register_remote/heartbeat/deregister_remote RPCs with TTL
- **Hybrid discovery** — `termlink discover` returns both local + remote TCP sessions
- **Hub forwarding** — transparently routes requests to remote sessions
- **Auth tokens** — `termlink token` for capability-based session auth
- **Platform-aware spawn** — `termlink spawn --backend auto|terminal|tmux|background` works on macOS, headless Linux, and anywhere tmux is available
- **Working dispatch script** — `scripts/tl-dispatch.sh` in the repo, multi-backend, tested with 3 parallel workers
- **264 tests passing** across 4 crates

To update TermLink: `cd /Users/dimidev32/001-projects/010-termlink && git pull && cargo install --path crates/termlink-cli`
Or from scratch: `git clone https://github.com/DimitriGeelen/termlink.git && cd termlink && cargo install --path crates/termlink-cli`

## Phased Rollout (framework owns all phases)

| Phase | Scope | TermLink provides |
|-------|-------|-------------------|
| **0** | fw doctor + agents/termlink/ + fw termlink route | Binary on PATH |
| **1** | Self-test (fw self-test via termlink interact) | interact, output, register |
| **2** | Parallel dispatch (fw termlink dispatch) | pty inject, events, discover |
| **3** | Remote control + observation | attach, inject, output, stream |
| **4** | Cross-machine coordination | TCP hub, remote store, hybrid discover |

Build Phase 0 now. The `termlink` binary is at `/Users/dimidev32/.cargo/bin/termlink`. Verify it works: `termlink --version` should print `termlink 0.1.0`.
