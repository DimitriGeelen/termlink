# TermLink Integration Spec for Engineering Framework

> Source: TermLink repo (010-termlink), Task T-142 inception + T-148 spec
> Target: Agentic Engineering Framework (`/usr/local/opt/agentic-fw/`)
> Date: 2026-03-16

## Summary

TermLink is a cross-terminal session communication system (Rust, 30+ CLI
commands, JSON output, reliable exit codes). The framework should integrate
it as an optional external tool to unlock: self-testing, parallel dispatch,
remote control, and cross-machine coordination.

## Ownership Boundary

| Repo | Owns |
|------|------|
| **TermLink** | Binary, protocol, session management, transport (Unix + TCP) |
| **Framework** | Agent wrapper, skills, dispatch patterns, CLAUDE.md integration |

Updates flow: TermLink repo -> `cargo install termlink` -> framework detects on PATH.

## What to Build (Phase 0)

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

Rules: WARN not FAIL (TermLink is optional). Include install hint.

### 2. `agents/termlink/` directory

```
agents/termlink/
  AGENT.md        # Intelligence: when/how to use TermLink
  termlink.sh     # Mechanical: wrapper for common patterns
```

### 3. `termlink.sh` subcommands

```bash
termlink.sh check          # Is TermLink on PATH? Print version. Exit 0/1.
termlink.sh spawn --task T-XXX [--name NAME]
                            # Register shell session tagged with task ID.
                            # Opens new Terminal.app window via osascript.
                            # Waits for session registration.
termlink.sh exec <session> <command>
                            # Run command via `termlink interact --json`.
                            # Return structured {output, exit_code, elapsed_ms}.
termlink.sh status          # List active TermLink sessions with task tags.
termlink.sh cleanup         # Deregister stale sessions. 3-phase Terminal cleanup:
                            #   Phase 1: kill child processes via TTY
                            #   Phase 2: `do script "exit"` to each window
                            #   Phase 3: close by reference using tracked window IDs
termlink.sh dispatch --task T-XXX --name <worker> --prompt "..."
                            # Spawn a claude -p worker in a real terminal.
                            # Fire-and-forget via `pty inject`.
                            # Completion signaled via `termlink event emit <worker> worker.done`.
termlink.sh wait --name <worker> [--all] [--timeout 300]
                            # Wait for worker.done event(s).
termlink.sh result --name <worker>
                            # Read worker result file from /tmp/tl-dispatch-*/
```

### 4. `fw termlink` route

Add to `fw` CLI routing:

```bash
termlink)
    exec "$AGENTS_DIR/termlink/termlink.sh" "$@"
    ;;
```

### 5. CLAUDE.md additions

Add a TermLink section covering:
- When to use (self-test, parallel dispatch, observation)
- Available primitives (interact, discover, events, spawn)
- Budget rule: don't spawn new sessions when context > 60%
- Cleanup rule: always cleanup spawned sessions before session end

## Key TermLink Primitives (for AGENT.md)

| Command | Purpose | Framework Use |
|---------|---------|---------------|
| `termlink interact <session> <cmd> --json` | Run command, get structured output | Star primitive. `termlink.sh exec` wraps this. |
| `termlink discover --json` | Find sessions by tag/role/name | Worker discovery |
| `termlink event emit/wait/poll` | Inter-session signaling | Coordination backbone |
| `termlink event broadcast <topic> <data>` | Fan-out to all listeners | Multi-worker notification |
| `termlink list --json` | List all sessions | Status overview |
| `termlink status <session> --json` | Session details | Health check |
| `termlink output <session> --strip-ansi` | Read terminal output | Log observation |
| `termlink pty inject <session> --enter` | Send input (fire-and-forget) | Long-running command start |
| `termlink register --shell --name X --tag Y` | Create named session | Tagged session lifecycle |
| `termlink hub start [--tcp ADDR]` | Start hub (optional TCP) | Cross-machine coordination |

## Existing Prototypes (in TermLink repo)

These can be referenced or adapted:

| File | What | Notes |
|------|------|-------|
| `scripts/tl-dispatch.sh` | Full dispatch script (spawn/status/wait/result/cleanup) | T-143, tested with 3 parallel workers |
| Skills: `/self-test` | Self-testing loop via TermLink | T-136/T-138, validated |
| `crates/termlink-hub/` | TCP hub with remote store, heartbeat, cross-machine discovery | T-145/T-146/T-147, 264 tests passing |

## Phased Rollout

| Phase | Scope | Depends On |
|-------|-------|------------|
| **0** | fw doctor check + agents/termlink/ + fw termlink route | Nothing (start here) |
| **1** | Self-test integration (move /self-test to fw subcommand) | Phase 0 |
| **2** | Parallel dispatch via TermLink (replace Agent tool mesh) | Phase 0 + tl-dispatch.sh reference |
| **3** | Remote control + attach patterns | Phase 0 + TermLink attach/inject |
| **4** | TCP transport + cross-machine | Phase 0 + TermLink TCP hub (already built) |

## Terminal Cleanup (Critical - T-074 Lesson)

**MUST use 3-phase cleanup. Never use `close` directly on Terminal windows.**

1. **Phase 1:** Get TTY via osascript, kill child processes (spare login/shell PID)
2. **Phase 2:** `do script "exit"` to each window (graceful shell exit)
3. **Phase 3:** Close remaining by reference using tracked window IDs

Track window IDs at spawn time. This was learned the hard way (T-074, T-143).

## TCP Hub Capabilities (Available Now)

- `termlink hub start --tcp 0.0.0.0:9100` — dual-listen on Unix + TCP
- `session.register_remote` RPC — remote session registers with host:port
- `session.heartbeat` RPC — keep-alive (default TTL: 5 min, reaper every 30s)
- `session.deregister_remote` RPC — clean removal
- `session.discover` — returns both local and remote sessions (hybrid)
- `hub.forward` — transparently routes to remote TCP sessions
- Auth: LAN-only first (no auth on TCP). Prod auth is Phase 4+.
