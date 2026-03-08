# T-002: Cross-Terminal Session Communication via Keyboard Input Injection

## Problem Statement

Can one terminal session inject keyboard input into another running terminal session, effectively enabling cross-session communication by treating keyboard input as a writable process interface?

## Core Insight

Terminal keyboard input is ultimately a process — bytes written to a file descriptor. If we can write to that fd from another process, we can "type" into a running session from outside it.

## Research Findings

### How Terminal Input Works (Unix/macOS)

1. **PTY (Pseudo-Terminal) Architecture:**
   - Every terminal session runs on a PTY pair: a **master** side and a **slave** side
   - The terminal emulator (iTerm2, Terminal.app) holds the master fd
   - The shell (zsh/bash) reads from the slave fd (`/dev/pts/N` on Linux, `/dev/ttysNNN` on macOS)
   - Keyboard input: User types → terminal emulator writes to master → kernel delivers to slave → shell reads it

2. **The Key Realization:**
   - Writing to the master side of a PTY is indistinguishable from keyboard input to the process reading the slave side
   - The `TIOCSTI` ioctl ("Terminal I/O Control — Simulate Terminal Input") historically allowed injecting characters into a terminal's input queue
   - **macOS/Linux divergence:** Linux kernels >=6.2 disabled `TIOCSTI` by default (security). macOS still supports it but it requires appropriate permissions.

### Mechanisms for Input Injection

#### 1. TIOCSTI (Legacy, Being Deprecated)
```c
// Inject one character at a time into terminal input queue
ioctl(fd, TIOCSTI, &character);
```
- **Pro:** Direct, simple, works at kernel level
- **Con:** Security risk (any process with fd access can inject), being removed from Linux
- **macOS status:** Still works but requires the process to have the terminal's fd

#### 2. Writing to PTY Master
```bash
# If you have access to the master fd, writing to it IS keyboard input
echo "ls -la" > /dev/ptmx  # (conceptual — actual fd management is more involved)
```
- **Pro:** Native mechanism, how terminal emulators already work
- **Con:** Getting the master fd from outside the terminal emulator process is non-trivial

#### 3. tmux / screen (Multiplexer Approach)
```bash
# tmux already solves this elegantly
tmux send-keys -t session_name "echo hello" Enter
```
- **Pro:** Battle-tested, cross-platform, handles escaping
- **Con:** Requires both sessions to be inside tmux

#### 4. AppleScript / osascript (macOS)
```bash
osascript -e 'tell application "Terminal" to do script "echo hello" in window 1'
```
- **Pro:** Works without modifying the target session
- **Con:** macOS only, executes a new command rather than injecting keystrokes

#### 5. Programmatic Keystroke Injection (macOS)
```bash
# Using CGEventCreateKeyboardEvent via Swift/ObjC
# Or: using osascript with System Events
osascript -e 'tell application "System Events" to keystroke "hello"'
```
- **Pro:** Simulates actual keystrokes at the OS level
- **Con:** Requires Accessibility permissions, targets the focused app only

#### 6. Named Pipes / Unix Sockets (Cooperative)
```bash
# Session A creates a pipe and reads from it
mkfifo /tmp/session_bridge
cat /tmp/session_bridge | while read cmd; do eval "$cmd"; done

# Session B writes to it
echo "ls -la" > /tmp/session_bridge
```
- **Pro:** Simple, no special permissions, cooperative
- **Con:** Requires target session to opt-in

#### 7. File Descriptor Passing via Unix Sockets
- One process can pass an open fd to another process via `SCM_RIGHTS` on a Unix domain socket
- This could allow sharing the PTY master fd between processes
- Advanced but very powerful

### Security Considerations

| Mechanism | Permission Required | Security Risk |
|-----------|-------------------|---------------|
| TIOCSTI | Terminal fd access | High (being deprecated for this reason) |
| PTY master write | Master fd access | Medium (contained to PTY) |
| tmux send-keys | tmux socket access | Low (tmux manages auth) |
| osascript | Accessibility perms | Medium (system-wide keystroke injection) |
| Named pipes | Filesystem perms | Low (cooperative, file-perm controlled) |
| fd passing | Socket access | Low (explicit handoff) |

### Prior Art

- **tmux/screen:** The gold standard for inter-terminal communication
- **Expect:** TCL tool that scripts interactive terminal sessions by controlling PTY master
- **reptyr:** Reparents a running process to a new terminal by manipulating PTYs
- **abduco:** Session management via PTY detach/attach
- **dtach:** Minimal detach/attach (like screen but single-window)
- **neercs:** Experimental terminal multiplexer with novel PTY tricks

### The Deep Insight

The concept is sound and well-established at multiple levels:

1. **Kernel level:** PTYs are bidirectional byte streams. Writing to master = keyboard input. This is not a hack — it's the design.
2. **Process level:** Any process that holds the master fd can inject input. Terminal emulators do this every time you press a key.
3. **IPC level:** Unix provides multiple mechanisms (pipes, sockets, shared memory) to bridge between processes.

The real question isn't "is this possible?" — it definitely is. The question is "at what level of abstraction do you want to operate?"

| Level | Approach | Complexity | Power |
|-------|----------|------------|-------|
| Application | tmux send-keys | Low | High (battle-tested) |
| OS | osascript/Accessibility | Medium | Medium (macOS only) |
| PTY | Direct master fd write | High | Very High (universal) |
| Kernel | TIOCSTI | Low (but deprecated) | High |
| Cooperative | Named pipes/sockets | Low | Medium (requires opt-in) |

## Potential Applications

1. **Agent-to-agent communication:** One Claude Code session sends commands to another
2. **Orchestration:** A master process coordinates multiple terminal sessions
3. **Session bridging:** Pass context/commands between separate terminal environments
4. **Remote pairing:** Inject input from a remote source into a local session
5. **Automated testing:** Script complex multi-terminal workflows

## Assumptions to Validate

- A-001: macOS still allows TIOCSTI without kernel restrictions
- A-002: PTY master fd can be obtained for an arbitrary terminal session
- A-003: tmux send-keys latency is acceptable for real-time communication
- A-004: Accessibility permissions are grantable programmatically

## Go/No-Go Criteria

- **Go if:** At least one mechanism works reliably on macOS without excessive permissions
- **No-go if:** All mechanisms require root or system-level permissions that users won't grant

## Constitutional Directive Mapping

The framework's four directives in priority order, scored per mechanism.

### D1 — Antifragility (System strengthens under stress)

The question: does the mechanism **degrade gracefully** and **learn from failures**?

| Mechanism | Antifragility | Reasoning |
|-----------|:---:|-----------|
| tmux send-keys | A | tmux sessions survive terminal crashes. If the target session dies, tmux reports it — you can restart. Session state persists across disconnects. Failure is visible and recoverable. |
| Named pipes/sockets | A | If one end dies, the other gets SIGPIPE/EOF — a clear signal, not silent corruption. Pipes can be recreated. Stateless by nature — nothing to corrupt. |
| fd passing (SCM_RIGHTS) | B | Explicit handoff is robust, but if the receiving process dies while holding the fd, it's gone. Requires reconnection protocol. |
| PTY master write | C | If you lose the master fd reference, recovery requires re-obtaining it (process inspection). No built-in retry or degradation — it works or it doesn't. |
| osascript | C | Relies on macOS automation subsystem. If Accessibility permissions revoke mid-session, fails silently or with opaque errors. No self-healing path. |
| TIOCSTI | D | Being actively removed from kernels. Building on it means your system gets **weaker** over time as OS updates break it. The opposite of antifragile. |

**Verdict:** Cooperative mechanisms (tmux, pipes, sockets) are inherently antifragile — failure modes are explicit, recovery is straightforward, and the system can adapt. Low-level kernel mechanisms are fragile — they depend on OS-level guarantees that are actively eroding.

### D2 — Reliability (Predictable, observable, auditable)

The question: can you **observe** what's happening, **predict** behavior, and **audit** after the fact?

| Mechanism | Reliability | Reasoning |
|-----------|:---:|-----------|
| tmux send-keys | A | tmux logs what was sent, when, to which session. `capture-pane` gives you the output. Full audit trail possible. Behavior is deterministic — send-keys does exactly what it says. |
| Named pipes/sockets | A | Every message is a discrete write. Easy to wrap with logging. Deterministic — what you write is what arrives. No interpretation layer. |
| fd passing | B | The handoff is auditable (socket communication), but once the fd is passed, the recipient's use of it is opaque to the sender. |
| PTY master write | B | What you write arrives, but there's no built-in acknowledgment. You can't easily confirm the target shell processed it. Race conditions with user typing simultaneously. |
| osascript | C | "do script" executes but you don't get reliable completion notification. Keystroke injection via System Events targets the focused app — if focus shifts between send and arrival, input goes to wrong app. Non-deterministic. |
| TIOCSTI | C | Character-at-a-time injection with no atomicity guarantee. If another process reads between characters, you get interleaving. Unreliable under load. |

**Verdict:** Message-based mechanisms (tmux, pipes) are reliable because they're discrete, loggable, and deterministic. Character-stream mechanisms (TIOCSTI, PTY write) have atomicity and observability problems.

### D3 — Usability (Joy to use, sensible defaults, actionable errors)

**Special focus area per your request.**

| Mechanism | Usability | Reasoning |
|-----------|:---:|-----------|
| tmux send-keys | A | One-liner. Intuitive mental model ("send these keys to that session"). Tab-completion on session names. Error messages are clear ("session not found", "window not found"). Zero config for basic use. |
| Named pipes | B | Simple concept (write to file = send message), but requires setup on both ends. The target must opt-in with a reader loop. Error: "broken pipe" is clear but not actionable without context. |
| osascript | B | English-like syntax is readable. But: Accessibility permission dialogs are confusing. Errors are AppleScript stack traces — opaque. Works great until it doesn't, then debugging is painful. |
| fd passing | C | Requires understanding Unix socket programming, ancillary data, cmsg headers. Not something you casually use. Powerful for library authors, hostile for end users. |
| PTY master write | D | Requires: finding the target PID, inspecting /proc or lsof for fd numbers, understanding master vs slave, dealing with permissions. Every step is a potential paper cut. No discoverability. |
| TIOCSTI | D | Requires C code or ctypes. Character-at-a-time API. No framing, no escaping helpers. Plus the "this might stop working after your next OS update" anxiety. |

**Usability deep dive — what "joy to use" looks like here:**

The ideal interface would be:
```bash
# Sender
fw session send "target-name" "echo hello"

# Receiver (auto-enrolled, zero config)
# Just works — input appears as if typed
```

This maps best to **tmux** (already has session naming, send-keys, zero config) or a **custom broker** built on named pipes/Unix sockets (cooperative but can be made transparent with a daemon).

**Usability killer:** Any mechanism that requires the user to know PIDs, fd numbers, or PTY device paths. Users think in session names, not process internals.

### D4 — Portability (No lock-in, prefer standards)

**Special focus area per your request.**

| Mechanism | Portability | Reasoning |
|-----------|:---:|-----------|
| Named pipes/sockets | A | POSIX standard. Works on macOS, Linux, BSDs, WSL. No dependencies. The most portable option possible. |
| tmux send-keys | B+ | tmux runs on macOS, Linux, BSDs, WSL. Available via every package manager. Not POSIX itself, but universally available. Doesn't work on Windows native (but who runs terminals there?). |
| fd passing (SCM_RIGHTS) | B | POSIX standard (`sendmsg`/`recvmsg` with `SCM_RIGHTS`). Works everywhere Unix sockets work. Implementation complexity varies by language. |
| PTY master write | B | PTY is POSIX (`posix_openpt`). But `/proc/PID/fd/` is Linux-only. macOS needs `lsof` or `dtrace` to find fds. Platform-specific discovery. |
| TIOCSTI | D | Deprecated on Linux >=6.2. macOS-specific behavior. Not a portable foundation. |
| osascript | F | macOS only. Period. |

**Portability + Distributed — the real question:**

For distributed operation (across machines, not just local terminals), the mechanisms split into two tiers:

| Tier | Mechanisms | Why |
|------|-----------|-----|
| **Network-ready** | Named pipes → Unix sockets → TCP sockets (trivial upgrade), tmux over SSH (`tmux -S /tmp/shared-socket`), custom broker with WebSocket/gRPC | These can cross machine boundaries with minimal changes |
| **Local-only** | TIOCSTI, PTY master write, osascript, fd passing | Fundamentally tied to local kernel/OS. Cannot distribute without a bridge layer. |

**The distributed architecture would look like:**

```
Machine A                          Machine B
┌─────────────┐                   ┌─────────────┐
│ Session 1   │                   │ Session 3   │
│ (tmux/pipe) │──── broker ──────│ (tmux/pipe) │
│ Session 2   │    (TCP/WS/      │ Session 4   │
│ (tmux/pipe) │     gRPC)        │ (tmux/pipe) │
└─────────────┘                   └─────────────┘
```

- **Local transport:** tmux send-keys or Unix sockets (fast, zero-config)
- **Remote transport:** TCP/WebSocket/gRPC bridge between brokers
- **Discovery:** mDNS/Bonjour for LAN, explicit config for WAN

This is essentially a **message bus with terminal endpoints** — and it maps cleanly to existing patterns (NATS, ZeroMQ, even MCP over stdio).

### Security Deep Dive

**Special focus area per your request.**

The threat model for cross-terminal input injection:

| Threat | Impact | Mitigation |
|--------|--------|------------|
| **Unauthorized injection** — attacker sends commands to your terminal | Critical (arbitrary code execution as your user) | Auth on the transport layer (socket permissions, tmux socket ACLs, TLS for remote) |
| **Eavesdropping** — attacker reads messages in transit | High (credential/secret exposure) | Encryption for remote (TLS/mTLS). Local: Unix socket permissions (0600) |
| **Replay attacks** — attacker replays captured commands | High | Message IDs + timestamps + nonces |
| **Man-in-the-middle** — attacker intercepts and modifies | Critical | mTLS for remote. Local: socket path ownership verification |
| **Privilege escalation** — injecting into a root terminal from unprivileged session | Critical | **Never allow cross-user injection.** Enforce UID matching on all transports. |
| **Session confusion** — input goes to wrong session | Medium (wrong command in wrong context) | Explicit session addressing (names, not indices). Confirmation for destructive commands. |

**Security ranking of mechanisms:**

1. **Named pipes/Unix sockets** — Best. Filesystem permissions control access. `chmod 0600` on the socket = only your user can connect. No ambient authority.
2. **tmux** — Good. Socket permissions + tmux's own session access control. Well-audited codebase.
3. **fd passing** — Good. Requires explicit socket connection. No ambient access.
4. **PTY master write** — Risky. If you can get the fd, you have full control. The "getting" part is the security boundary, and it's OS-dependent.
5. **osascript** — Risky. Accessibility permissions are all-or-nothing. Once granted, any process can inject keystrokes into any app.
6. **TIOCSTI** — Dangerous. Any process with the terminal fd can inject. This is literally why Linux deprecated it.

**For distributed/remote operation, security requirements escalate:**

- **Local:** Unix socket permissions are sufficient (0600, uid match)
- **LAN:** TLS with pre-shared keys or mTLS (certificate pinning)
- **WAN:** mTLS mandatory + message signing + rate limiting + allowlisted commands

### Consolidated Directive Scorecard

| Mechanism | D1 Antifragile | D2 Reliable | D3 Usable | D4 Portable | Security | Distributed | **Overall** |
|-----------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| tmux send-keys | A | A | A | B+ | Good | Via SSH | **Top Tier** |
| Named pipes/Unix sockets | A | A | B | A | Best | Via broker | **Top Tier** |
| fd passing (SCM_RIGHTS) | B | B | C | B | Good | No | Mid Tier |
| PTY master write | C | B | D | B | Risky | No | Low Tier |
| osascript | C | C | B | F | Risky | No | Low Tier |
| TIOCSTI | D | C | D | D | Dangerous | No | **Eliminate** |

### Recommendation

**Two-layer architecture:**

1. **Primary transport: Unix sockets with a lightweight broker** — scores A on portability, A on antifragility, best security. The broker provides session naming, message framing, and logging.
2. **Terminal integration: tmux send-keys where available** — scores A on usability and reliability. Falls back to the broker's own PTY management where tmux isn't present.
3. **Remote extension: broker-to-broker over TLS** — the Unix socket broker grows a TCP listener for distributed operation. Same protocol, different transport.

**TIOCSTI is eliminated.** It fails on every directive and is being actively removed from operating systems.

**osascript is eliminated for core use.** macOS-only violates D4. Acceptable as a platform-specific convenience layer, not as architecture.

## Dialogue Log

### 2026-03-08 — Initial exploration request
- **Human asked:** Can we investigate passing information between terminal sessions by injecting keyboard input, recognizing that terminal input is fundamentally a process/fd operation?
- **Core insight validated:** Yes — PTY architecture means keyboard input is just bytes on a file descriptor. Writing to the master side is indistinguishable from physical keystrokes.
- **Outcome:** Research documented, multiple viable mechanisms identified at different abstraction levels.

### 2026-03-08 — Directive mapping and distributed analysis
- **Human asked:** Map mechanisms against the four constitutional directives, with special attention to usability, security, and portability/distributed operation.
- **Key findings:** tmux and Unix sockets score top tier across all directives. TIOCSTI and osascript eliminated. Two-layer architecture recommended (Unix socket broker + tmux integration). Distributed operation viable via broker-to-broker TLS.
- **Outcome:** Consolidated scorecard produced. Clear recommendation: Unix socket broker as primary, tmux as terminal integration layer, TLS bridge for remote.
