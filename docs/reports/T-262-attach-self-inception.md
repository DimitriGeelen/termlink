# T-262: Attach-Self Inception Research

## Core Question

Can TermLink register an **already-running** interactive shell as an endpoint, making it injectable/observable without spawning a new process?

## Current State

### What `register --shell` does today
1. Spawns a **new PTY** via `PtySession::spawn()` (session.rs:62-76)
2. Creates Unix socket + JSON sidecar for discovery
3. Runs an RPC server accepting inject/query/execute/emit
4. Bidirectional I/O loop between PTY master FD and data plane

### What `attach` does today
- Connects to an **already-registered** PTY session
- Polls `query.output` for scrollback, sends `command.inject` for input
- Requires session to already exist with a PTY — cannot register an existing shell

### The Gap
`register` spawns a new process. There's no way to make the **current** shell (e.g., the one you SSH'd into) a TermLink endpoint.

## Design Options

### Option A: `termlink attach-self` — PTY interposition

Wrap the current shell's stdin/stdout through a TermLink-managed proxy:

1. Save current terminal settings
2. Create a Unix socket + register with hub
3. Interpose on stdin: multiplex between real terminal input and injected RPC input
4. Interpose on stdout: tee output to both terminal and TermLink data plane
5. Run RPC event loop in background thread

**Pros:** Full parity with `register --shell` — inject, output, events all work
**Cons:** Complex — needs to intercept the existing PTY's I/O without breaking it. Race conditions between real keystrokes and injected ones. Terminal state management (raw mode conflicts).

### Option B: `termlink attach-self` — sidecar with named pipe

Create a sidecar process that registers as a TermLink endpoint and communicates with the shell via a named pipe or shared PTY:

1. Fork a background sidecar
2. Sidecar creates Unix socket + registers
3. Shell writes to named pipe → sidecar injects via PTY
4. Sidecar captures output via script(1) or similar

**Pros:** Simpler than PTY interposition
**Cons:** Fragile, platform-dependent, lossy output capture

### Option C: Shell function + `register` (no new command)

Add shell functions to `.bashrc`/`.zshrc` that auto-register on shell startup:

```bash
termlink-attach() {
  termlink register --name "$(hostname)-ssh" --shell --background
  # But this spawns a NEW shell inside TermLink, not the current one
}
```

**Cons:** This doesn't solve the problem — it spawns a nested shell, not registering the current one.

### Option D: `termlink register --attach-stdin` — register with current TTY

Modify `register` to optionally use the current process's TTY instead of spawning a new PTY:

1. Detect current TTY via `/dev/tty` or `ttyname(STDIN_FILENO)`
2. Open the TTY's master FD (if accessible) for injection
3. Register as normal — Unix socket, RPC server, discovery
4. For output: read from TTY slave or use `TIOCPKT`
5. For inject: write to TTY master

**Problem:** The current shell's PTY master is owned by the terminal emulator or SSH daemon, not by our process. We can't get the master FD without being the parent process.

### Option E: `termlink register --self` — event-only endpoint

Register the current shell as an **event-only** endpoint (no inject/output):

1. Create Unix socket + register with hub
2. Run RPC server in background thread
3. Support: events (emit/poll), kv store, status queries
4. Do NOT support: inject, output, stream (requires PTY ownership)
5. The shell can emit events via `termlink emit` and receive via `termlink event wait`

**Pros:** Simple, clean, solves the core use case (cross-machine agent communication)
**Cons:** No inject/output — but that's the honest capability boundary

### Option F: `termlink register --self` — event + inject via shell integration

Like Option E, but add inject support via a shell integration hook:

1. Register as event-only endpoint
2. Install a shell `PROMPT_COMMAND` or `precmd` hook that checks for pending injections
3. Pending injections are read from a queue file or socket
4. Hook feeds them to the shell's input buffer via `zle` (zsh) or `bind` (bash)

**Pros:** Works with existing shell, no PTY interception needed
**Cons:** Injection only happens at prompt boundaries (not real-time). Limited to simple commands.

## Recommendation

**Option E is the pragmatic GO.** The real use case from the pickup message is: "SSH into remote, run `termlink attach`, local agent connects via hub." The agent needs to **send events and receive events** — it doesn't need to inject keystrokes into someone's shell. That's what `register --shell` is for.

Option E gives us:
- Event bus (emit, poll, collect, emit-to)
- KV store (metadata, state sharing)
- Hub discovery (tags, roles, capabilities)
- Status queries

This covers 90% of the cross-machine agent communication use case. The remaining 10% (inject/output) requires PTY ownership which is architecturally incompatible with attaching to an existing shell.

**Option F** could be a follow-up if shell-level injection proves necessary.

## Go/No-Go Assessment

**GO for Option E** — event-only self-registration:
- Bounded scope: reuse existing RPC server, just skip PTY spawn
- ~50 lines of new code in `session.rs` (new `--self` flag path)
- Solves the stated use case
- No PTY complexity

**NO-GO for Options A/B/D** — PTY interposition:
- High complexity, fragile, platform-dependent
- SSH PTY master FD is not accessible to child processes
- Solving a problem that `register --shell` already handles (just spawn a new session)
