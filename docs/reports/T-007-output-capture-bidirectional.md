# T-007: Output Capture & Bidirectional Communication — Research Report

## Question

How does a TermLink session capture terminal output for `query.output` (snapshots) and `data.stream` (live streaming)? What are the architectural options, tradeoffs, and constraints?

## Parent

T-003 (GO: message bus + injection adapter, control/data plane split)

## Problem Statement

TermLink currently has:
- **Input path:** `command.execute` (spawn subprocess, capture stdout/stderr) and `command.inject` (resolve key entries to bytes — PTY injection deferred)
- **No output path:** No way to capture what's happening in the terminal that registered the session

The half-duplex problem: after injecting keystrokes (`command.inject`), there's no way to read what the terminal produced in response. Without output capture, TermLink is a blind remote control.

## Research Areas

1. Output capture mechanisms — how to read terminal output
2. Scrollback buffer management — storing output for `query.output`
3. Live streaming architecture — `data.stream` frame emission
4. PTY integration depth — what level of terminal emulation is needed
5. Interaction with existing `command.execute` — overlap and unification

---

## Findings

### 1. Output Capture Mechanisms

There are four fundamental approaches to capturing terminal output:

#### Option A: PTY Master/Slave (Full PTY ownership)

The session spawns a PTY pair and runs the user's shell as the slave process. TermLink owns the master side, giving it:
- Full read access to all output
- Full write access for input injection
- Resize control (SIGWINCH)
- Signal forwarding

**Pros:**
- Complete bidirectional control
- Output is raw terminal bytes (ANSI sequences preserved)
- Works with interactive programs (vim, less, htop)
- Can implement `data.stream` directly from PTY master reads
- `command.inject` becomes a PTY master write

**Cons:**
- TermLink must BE the terminal emulator (or wrap one)
- User must start their shell through TermLink: `termlink register --shell`
- Existing terminal sessions can't be "attached to" retroactively
- Requires careful signal handling and job control
- Must handle terminal resize (TIOCSWINSZ)

**Complexity:** High, but this is what tools like tmux, screen, and script(1) do.

#### Option B: Script/Tee Approach (Output logging)

Use `script(1)` or equivalent to log terminal output to a file, then read the file.

**Pros:**
- Simple to implement
- Works with existing terminals (no PTY ownership needed)

**Cons:**
- Output is post-processed, not real-time
- No input injection capability
- File-based, requires polling or inotify
- Misses interactive program state
- Not suitable for `data.stream` (latency too high)

**Verdict:** Insufficient for bidirectional communication.

#### Option C: Process Substitution / Pipe Tapping

Redirect stdout/stderr through a pipe that TermLink monitors.

**Pros:**
- Works for simple command output
- Low overhead

**Cons:**
- Only captures stdout/stderr, not PTY output (no ANSI sequences)
- Breaks interactive programs
- Can't inject input
- Not applicable to existing shells

**Verdict:** Too limited. This is what `command.execute` already does.

#### Option D: Hybrid — PTY for registered sessions, pipe for execute

- `command.execute` continues using subprocess with pipe capture (already implemented)
- `termlink register --shell` creates a PTY-backed session for full bidirectional
- `command.inject` only works on PTY-backed sessions
- `query.output` works on PTY sessions (scrollback) and execute results (stdout/stderr)

**Verdict:** This is the right approach. It separates two use cases that have different requirements.

### 2. PTY Architecture Design

```
┌─────────────────────────────────────────────────────────┐
│                    TermLink Session                       │
│                                                          │
│  ┌──────────┐    ┌───────────┐    ┌──────────────────┐  │
│  │ Control   │    │ PTY       │    │ Scrollback       │  │
│  │ Plane     │◄──►│ Manager   │◄──►│ Buffer           │  │
│  │ (JSON-RPC)│    │           │    │ (ring buffer)    │  │
│  └──────────┘    └─────┬─────┘    └──────────────────┘  │
│                        │                                  │
│                  ┌─────┴─────┐                           │
│                  │ PTY Master│                           │
│                  │ (read/    │                           │
│                  │  write)   │                           │
│                  └─────┬─────┘                           │
│                        │                                  │
│                  ┌─────┴─────┐                           │
│                  │ PTY Slave │                           │
│                  │ (shell)   │                           │
│                  └───────────┘                           │
└─────────────────────────────────────────────────────────┘
```

**PTY Manager responsibilities:**
- Spawn shell as PTY slave process
- Read output from PTY master → scrollback buffer + data.stream subscribers
- Write input from command.inject → PTY master
- Forward signals (SIGINT, SIGTERM, SIGWINCH)
- Detect child process exit

### 3. Scrollback Buffer Design

For `query.output` — a ring buffer of terminal output:

```rust
struct ScrollbackBuffer {
    buffer: VecDeque<u8>,
    max_bytes: usize,  // e.g., 1 MiB
}

impl ScrollbackBuffer {
    fn append(&mut self, data: &[u8]);
    fn last_n_lines(&self, n: usize) -> &[u8];
    fn last_n_bytes(&self, n: usize) -> &[u8];
    fn since_marker(&self, marker: u64) -> &[u8]; // offset-based
}
```

**Size considerations:**
- Default: 1 MiB (roughly 10,000–20,000 lines of typical terminal output)
- Configurable per session
- Ring buffer drops oldest bytes when full
- Byte-oriented (preserves ANSI sequences, binary safety)

### 4. Live Streaming Architecture

For `data.stream` — subscribers receive real-time output:

```
PTY Master read loop:
  1. Read bytes from PTY master
  2. Append to scrollback buffer
  3. For each subscriber: encode as data plane frame, send
```

Subscribers connect via the data plane socket (separate from control plane).
Each subscriber gets its own channel_id.

**Backpressure:** If a subscriber can't keep up, options:
- Drop frames (lossy, simple)
- Buffer per-subscriber up to limit, then drop oldest
- Disconnect slow subscribers

This is T-009's territory (concurrency/backpressure), so we note the interface but don't design it here.

### 5. Integration with command.execute

Current `command.execute` spawns a subprocess with piped stdout/stderr. This is orthogonal to PTY output capture:

| Feature | command.execute | PTY capture |
|---------|----------------|-------------|
| Use case | Run a command, get output | Observe terminal activity |
| Input | Command string | Keystrokes (command.inject) |
| Output | stdout + stderr strings | Raw PTY bytes (ANSI included) |
| Lifetime | Request-scoped | Session-scoped |
| Interactive | No | Yes |

Both can coexist. `command.execute` remains the simple "run and capture" path.

### 6. Technical Constraints

**Platform:**
- PTY creation: `posix_openpt()` / `openpty()` — POSIX, works on macOS and Linux
- Rust crate options: `nix` (low-level), `portable-pty` (cross-platform), or raw libc
- macOS specific: no `epoll`, use `kqueue` (tokio handles this)

**Terminal emulation:**
- TermLink does NOT need to be a terminal emulator
- Raw bytes pass through — the user's actual terminal renders them
- Scrollback buffer stores raw bytes, not parsed screen state
- If screen-state queries are needed later (cursor position, screen contents), that's a separate concern (terminal state machine like vte/alacritty_terminal)

**Resize handling:**
- PTY sessions need SIGWINCH forwarding
- Control plane message: `event.resize { cols, rows }`
- PTY master: `ioctl(TIOCSWINSZ)`

## Scope Fence

**IN scope (this inception):**
- PTY ownership model for registered sessions
- Scrollback buffer design
- query.output handler
- data.stream emission interface (not full subscriber management)
- command.inject wiring to PTY master write

**OUT of scope:**
- Terminal state machine / screen parsing (future, if needed)
- Subscriber management / backpressure (T-009)
- Distributed streaming (T-011)
- Interactive program special handling (T-010)
- Security / capability gating for output access (T-008)

## Go/No-Go Analysis

**GO criteria:**
1. PTY creation is well-supported on target platforms (macOS, Linux) — **YES**, POSIX standard
2. Architecture integrates with existing code without major rewrites — **YES**, additive (new module, new handlers)
3. Reasonable complexity for v0.1 — **YES**, core is ~300-500 lines (PTY spawn + read loop + scrollback)
4. Clear path from current command.execute to PTY-backed sessions — **YES**, orthogonal addition

**NO-GO criteria:**
1. Requires terminal emulation — **NO**, raw byte passthrough is sufficient for v0.1
2. Breaks existing functionality — **NO**, purely additive
3. Platform-specific to the point of non-portability — **NO**, POSIX PTY is universal

## Decision

**GO** — Implement PTY-backed sessions with scrollback buffer and output streaming.

### Implementation plan (separate build tasks):
1. **T-0XX: PTY manager** — spawn shell, read/write loop, scrollback buffer
2. **T-0XX: query.output handler** — return scrollback snapshot
3. **T-0XX: Wire command.inject to PTY write** — complete the input path
4. **T-0XX: data.stream emission** — live output over data plane frames

### Key design decisions:
- **PTY ownership model** — TermLink owns the PTY master, shell runs as slave
- **Scrollback is byte-oriented** — no terminal parsing, raw ANSI passthrough
- **Two session modes** — PTY-backed (full bidirectional) vs lightweight (execute-only)
- **Separate from command.execute** — different lifetime, different I/O model
