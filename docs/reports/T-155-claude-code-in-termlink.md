# T-155: Claude Code inside TermLink Session — Research

## Problem Statement

Claude Code runs in a single terminal. The human can only interact with it from
that terminal. If you're on another machine, another terminal, or want to observe
what the master session is doing — you can't. TermLink already provides session
management, PTY I/O, events, and TCP hub. Can we wrap Claude Code itself in a
TermLink session to make it remotely accessible?

## Research Questions & Findings

### Q1: Launch Method

**Recommended: Option A — `termlink spawn --name master -- claude`**

How it works internally (main.rs:3012):
1. `termlink register --shell` runs in background (allocates PTY)
2. Waits for registration (1s)
3. Runs `claude` in the allocated PTY
4. On claude exit, kills registration process

| Option | Viable? | TUI support | User interaction | Hub registered |
|--------|---------|-------------|------------------|----------------|
| `spawn --backend auto` | YES (recommended) | Full PTY | Transparent | Yes |
| `register --shell` + inject | YES (awkward) | Full PTY | Via attach | Yes |
| `termlink run -- claude` | NO | No PTY | None | N/A |
| Custom PTY mode | Unnecessary | Already exists via spawn | N/A | N/A |

`spawn` with `--backend auto` picks the right backend per environment:
- macOS GUI → Terminal.app window (user interacts directly)
- tmux available → tmux session (attach with `tmux attach -t tl-master`)
- Headless → background PTY (attach with `termlink attach master`)

### Q2: Observation / Interaction Model

Three observation mechanisms, ordered by capability:

**1. `termlink stream master`** — BEST for live TUI observation
- Event-driven binary frames via data plane (Unix socket)
- Latency: <5ms. PTY reads 4096-byte chunks, broadcasts immediately
- Handles all escape codes, alternate screen, colors transparently
- Bidirectional: can send Input frames and Resize frames back
- If client can't keep up, frames are dropped (no backpressure on PTY)

**2. `termlink attach master`** — Good fallback
- Polls scrollback every 50ms, writes raw bytes to stdout
- Full TUI mirroring (puts local terminal in raw mode)
- Read+Write: stdin forwarded via `command.inject` RPC
- Detach with Ctrl+]
- Multiple observers can attach simultaneously (each polls independently)
- 20-50ms latency average — fine for human observation

**3. `termlink pty output master --strip-ansi`** — Text extraction only
- Reads from 1 MiB ring buffer (configurable via `scrollback_bytes`)
- `--strip-ansi` removes escape sequences and `\r`
- Returns recent history (last N lines/bytes), not full screen state
- Good for: extracting Claude's last response, log observation
- Bad for: reconstructing current TUI screen

**Key insight:** `attach` and `stream` both give full TUI mirror. A remote
user can see exactly what the local user sees. `stream` is lower latency
but requires data plane socket; `attach` works over any RPC connection.

### Q3: Input Handling

`termlink pty inject` writes directly to PTY master fd via `libc::write()`:

| Input type | Method | Works for Claude Code? |
|------------|--------|----------------------|
| Text + Enter | `pty inject master "hello" --enter` | YES — types into prompt |
| Special keys | `pty inject master --key Ctrl+C` | YES — named key lookup |
| Raw bytes | `pty inject master --raw <base64>` | YES — arbitrary escape sequences |
| Slash commands | `pty inject master "/help" --enter` | YES — text injection |

- Latency: <1ms for small inputs
- No artificial delay, synchronous write
- `--enter` appends newline automatically
- Multiple partial writes retried with yield (no data loss)

**For `attach` or `stream`:** stdin is forwarded bidirectionally in real-time.
A remote user attached to the session can type normally — it's indistinguishable
from sitting at the original terminal.

### Q4: claude-fw Integration — Steelman vs Strawman

**Current `claude-fw` wrapper** (`/usr/local/opt/agentic-fw/libexec/bin/claude-fw`):
- Runs `command claude "${CLAUDE_ARGS[@]}"` as subprocess (line 66)
- Checks `.context/working/.restart-requested` after exit
- Max 5 auto-restarts, 3s pause between, Ctrl+C to cancel
- No TermLink integration exists today

#### STEELMAN: Auto-register in claude-fw

**Argument for:**
- Zero friction — every `claude-fw` session is automatically discoverable
- Restart-aware: TermLink session persists across claude restarts (same PTY)
- Remote monitoring comes free: `termlink attach master` from any terminal
- Workers can discover the master: `termlink discover --tag master`
- Cross-machine: with TCP hub, monitor from another machine
- Signal extension: restart JSON includes `termlink_session` for continuity
- Opt-in via `--termlink` flag, no behavior change for users who don't want it

**Implementation:**
```bash
# claude-fw modification (conceptual)
if [ "$TERMLINK_ENABLED" = "1" ] && command -v termlink >/dev/null; then
    termlink spawn --name "claude-master" --tags "master,framework" \
        --backend auto -- claude "${CLAUDE_ARGS[@]}"
else
    command claude "${CLAUDE_ARGS[@]}"
fi
```

**Cost:** ~15 lines in claude-fw. TermLink dependency remains optional (graceful fallback).

#### STRAWMAN: Auto-register in claude-fw

**Argument against:**
- Adds a dependency to the critical path — if TermLink has a bug, claude-fw breaks
- PTY-in-a-PTY: user's terminal already has a PTY; TermLink allocates another one.
  Could cause terminal size mismatches, escape code double-processing, or signal issues
- Session name collisions: multiple `claude-fw` instances on same machine
- Restart complexity: TermLink session must survive claude exit + restart cycle
  without orphaning the PTY or losing scrollback
- Security: TermLink session makes Claude Code's I/O accessible to anyone who can
  reach the hub socket. No auth by default (capability tokens exist but aren't wired)
- Overhead: additional process (TermLink session manager) per Claude Code instance

**Risk assessment:**
- PTY nesting is the biggest technical risk — needs a spike to validate
- Security is solvable (capability tokens from T-079)
- Session naming is solvable (PID or session ID suffix)

#### VERDICT: Steelman wins, with conditions

The benefits (remote access, monitoring, worker discovery) clearly outweigh the costs.
But two things need validation before building:

1. **PTY nesting spike** — Does `termlink spawn -- claude` actually render Claude's TUI
   correctly? Or does the double-PTY cause issues? (5-minute test)
2. **Session persistence across restart** — Can the TermLink session survive `claude`
   exiting and restarting? Or does the PTY close? (needs spawn mode that keeps PTY alive)

## Assumptions to Validate

- A1: Claude Code TUI renders correctly inside a TermLink-managed PTY (PTY nesting)
- A2: `termlink attach` gives a usable remote mirror of Claude Code's TUI
- A3: Input injection via attach/stream works for Claude Code's prompt
- A4: Session can survive claude restart (for claude-fw integration)
- A5: TCP hub enables cross-machine observation (already validated in T-144/T-145)

## Dialogue Log
