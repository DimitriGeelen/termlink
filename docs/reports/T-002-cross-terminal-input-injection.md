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

## Deep Reflection

### What We Actually Have vs What We Think We Have

We've identified mechanisms and scored them — but we haven't questioned our own framing. The original question was about "passing information between terminal sessions via keyboard input injection." But there's a deeper tension here that the scorecard papers over:

**Are we building input injection or a message bus?**

These are fundamentally different things:

| Aspect | Input Injection | Message Bus |
|--------|----------------|-------------|
| Mental model | "Type into another terminal" | "Send a message to another session" |
| Granularity | Characters/keystrokes | Structured messages |
| Bidirectional? | One-way (inject only) | Two-way (request/response) |
| Acknowledgment | None (fire and forget) | Built-in (ack/nack) |
| Target awareness | Target doesn't know it's receiving | Target is an active participant |
| State | Stateless | Can be stateful |

The tmux approach blurs this line — `send-keys` is injection, but tmux's session model gives you bus-like properties. The Unix socket approach is inherently a bus. **We need to decide which we're building**, because the architecture diverges significantly.

**The honest answer:** The *interesting* use cases (agent-to-agent coordination, orchestration, distributed operation) all need a message bus. Pure input injection is a parlor trick — cool but limited. The real value is in structured, bidirectional, acknowledged communication that *happens to be able to* inject terminal input as one of its capabilities.

### The Abstraction Inversion Problem

There's a subtle trap in our two-layer architecture. If we build a Unix socket broker and then use tmux as the terminal integration layer, we've created an abstraction inversion:

```
Our broker (high-level, custom) → tmux (high-level, mature) → PTY (low-level, kernel)
```

We're wrapping a mature, feature-rich tool (tmux) inside our own less-mature broker. The question is: **should tmux be the broker?** tmux already has:
- Session naming and discovery
- Socket-based IPC
- Multi-client support
- Scriptable via command mode
- SSH tunneling

What tmux *doesn't* have:
- Structured message protocol (it's keystroke-oriented)
- Cross-machine native federation
- Message acknowledgment
- Message routing/pub-sub

This leads to a design fork we must resolve.

### The "Cooperative vs Invasive" Spectrum

Our security analysis treats "cooperative" as a downside (requires opt-in). But from the framework's D1 (antifragility) perspective, cooperative IS the feature:

- **Invasive injection** (TIOCSTI, PTY write): The target doesn't consent. This is powerful but inherently adversarial. It's useful for `expect`-style automation but hostile for agent-to-agent communication.
- **Cooperative messaging** (pipes, sockets, tmux): Both sides agree to communicate. This enables contracts, protocols, versioning, and graceful degradation.

**For agent-to-agent communication, cooperative is not a limitation — it's a requirement.** You want sessions that explicitly register as participants, declare their capabilities, and handle messages with intention.

### What's Missing: The Hard Problems

The research so far covers *transport* well but hasn't touched:

1. **Session identity** — How does Session A find Session B? By name? By role? By capability?
2. **Message semantics** — What's in a message? Raw text? JSON? A command with arguments?
3. **Conversation state** — Is this fire-and-forget or request/response? Can messages be correlated?
4. **Failure handling** — What happens when a message can't be delivered? Retry? Dead letter? Notify sender?
5. **Ordering guarantees** — If A sends 3 messages to B, do they arrive in order?
6. **Concurrency** — Two senders target the same session simultaneously. What happens?
7. **Backpressure** — Target is busy. Does the sender block? Buffer? Drop?
8. **The output problem** — We can inject INPUT. How do we capture OUTPUT? The PTY master can read the slave's output, but in the broker model, how does the sender get the result of a command it sent?
9. **Interactive programs** — What happens when the target is running `vim`, `less`, `python REPL`, or `ssh`? Input injection semantics change completely per program.
10. **Signals** — Can we send Ctrl+C (SIGINT), Ctrl+Z (SIGTSTP)? These aren't just characters — they trigger kernel signal delivery via the terminal driver's line discipline.

---

## Investigation Topics

Each topic below is a self-contained investigation area. Ordered by dependency — earlier topics inform later ones.

### IT-001: Fundamental Paradigm — Injection vs Message Bus vs Hybrid

**Question:** Are we building terminal input injection, a message bus with terminal endpoints, or a hybrid?

**Why it matters:** Every subsequent design decision depends on this. A message bus needs framing, routing, ack/nack. Injection needs PTY access and keystroke encoding. A hybrid needs a clean interface between the two modes.

**What to investigate:**
- Survey use cases: which need injection, which need messaging, which need both?
- Can a message bus degrade gracefully to injection for non-cooperative targets?
- What does the MCP (Model Context Protocol) do here? It's already a structured message bus for AI agents — could we extend it rather than building a new one?
- How does this relate to LSP (Language Server Protocol)? Both MCP and LSP are JSON-RPC over stdio — is there a convergence point?

**Expected output:** A paradigm decision document with use-case mapping.

### IT-002: Session Identity, Discovery, and Lifecycle

**Question:** How do sessions find each other, and what happens when they appear/disappear?

**Why it matters:** Without discovery, you need hardcoded addresses. Without lifecycle management, you get stale references and silent delivery failures.

**What to investigate:**
- **Naming:** Human-readable names? UUIDs? Role-based ("the-coder", "the-tester")?
- **Registration:** Automatic (session starts → registers) vs explicit (`fw session register --name builder`)?
- **Discovery:** Local (filesystem: `/tmp/fw-sessions/`), LAN (mDNS/Bonjour), WAN (registry service)?
- **Liveness:** Heartbeat? Socket liveness check? PID monitoring?
- **Deregistration:** Automatic on process exit (socket close, PID gone) vs explicit?
- **How tmux handles this** — tmux's `list-sessions` is already a discovery mechanism. Can we piggyback?
- **How containers/VMs/SSH change this** — sessions inside Docker, across SSH tunnels, in devcontainers

**Expected output:** Session lifecycle state machine + discovery protocol sketch.

### IT-003: Message Protocol Design

**Question:** What's the wire format? What message types exist? How are messages framed?

**Why it matters:** This is the contract between all participants. Get it wrong and everything downstream suffers.

**What to investigate:**
- **Framing:** Length-prefixed? Newline-delimited JSON? Protobuf? MessagePack?
- **Message types:** At minimum: `inject` (raw keystrokes), `command` (structured), `query` (request/response), `event` (notification), `control` (lifecycle)
- **Envelope fields:** sender, target, message_id, correlation_id, timestamp, type, payload
- **Encoding:** How to represent special keys (Enter, Ctrl+C, arrow keys) in the injection case?
- **Versioning:** How does the protocol evolve without breaking existing sessions?
- **Size limits:** Max message size? Chunking for large payloads?
- **Relation to MCP:** MCP uses JSON-RPC 2.0. Could our protocol be a MCP transport or tool?

**Expected output:** Protocol specification draft (v0.1).

### IT-004: The Output Capture Problem

**Question:** We can inject input. How do we capture the result?

**Why it matters:** A command without its output is half a conversation. Agent orchestration requires knowing what happened.

**What to investigate:**
- **PTY master read:** The master side of a PTY receives all output. If we own the PTY (broker-managed), we can read it. But if we're injecting into an existing tmux session, tmux owns the master.
- **tmux capture-pane:** Captures the visible pane content. Works but lossy — only what's on screen.
- **tmux pipe-pane:** Streams pane output to a file/pipe. This is the real answer for tmux-based output capture.
- **Script/typescript:** The `script` command records terminal sessions. Could sessions auto-record?
- **Command wrapping:** Instead of injecting `ls -la`, inject `ls -la > /tmp/fw-result-$(uuid) 2>&1; echo __DONE__`. Parse the marker.
- **Shell integration:** A shell function that wraps command execution and reports results to the broker. Like how iTerm2 shell integration tracks command boundaries.
- **The streaming problem:** Some commands produce continuous output (logs, watches). How to handle?

**Expected output:** Output capture strategy with trade-off matrix.

### IT-005: Concurrency, Ordering, and Backpressure

**Question:** What happens when multiple senders target one session, or one sender blasts many messages?

**Why it matters:** Without concurrency control, you get interleaved commands, garbled output, and race conditions.

**What to investigate:**
- **Serialization:** Queue messages per target? Mutual exclusion (one sender at a time)?
- **Ordering:** FIFO per sender? Global order? Causal ordering?
- **Backpressure:** Block sender? Buffer in broker? Drop with notification?
- **Priority:** Can urgent messages (Ctrl+C) jump the queue?
- **The typing race:** User is typing in a terminal. An injection arrives. What happens to the user's partial input?
- **tmux behavior:** What does tmux do when two clients send-keys simultaneously?
- **Distributed ordering:** Across machines, global ordering requires consensus (Lamport clocks, vector clocks). How far do we go?

**Expected output:** Concurrency model document.

### IT-006: Security Model — Capability-Based Access

**Question:** Beyond transport-level security, how do we authorize *what* a sender can do to a target?

**Why it matters:** "Can connect to the socket" is not the same as "can run arbitrary commands in my terminal." We need fine-grained authorization.

**What to investigate:**
- **Capability tokens:** Target issues tokens that grant specific permissions (inject, command, query, control)
- **Command allowlists:** Target declares what commands it accepts (`["git status", "fw *", "echo *"]`)
- **Role-based access:** "orchestrator" can send commands, "observer" can only query
- **Consent prompts:** Target session displays "[Session X wants to run 'rm -rf /'] Allow? [y/n]" — like mobile permission dialogs
- **Audit logging:** Every cross-session action logged with sender, target, message, timestamp
- **Revocation:** Can a target revoke a sender's access mid-session?
- **The Tier 0 connection:** Our framework already has Tier 0 (destructive action approval). Cross-session injection of destructive commands should integrate with this.

**Expected output:** Capability model specification.

### IT-007: Interactive Program Handling

**Question:** What happens when the target is running something other than a shell prompt?

**Why it matters:** A terminal session isn't always at a `$` prompt. It might be in vim, a Python REPL, an SSH session, a `less` pager, or a password prompt. Each has completely different input semantics.

**What to investigate:**
- **Mode detection:** How to know what the target is running? (`$TERM_PROGRAM`, process tree inspection, shell integration hooks)
- **Vim/Neovim:** Input means different things in normal mode vs insert mode vs command mode
- **REPLs:** Python, Node, Ruby REPLs — they read lines, but with their own editing (readline/libedit)
- **Password prompts:** `sudo`, `ssh` — injecting here is a security minefield
- **Pagers:** `less`, `more` — single-character commands, not line-oriented
- **Nested sessions:** `ssh` into remote → tmux on remote → vim inside tmux. How deep does injection go?
- **Should we even try?** Maybe the answer is: only inject when target is at a known shell prompt. Everything else is out of scope for v1.

**Expected output:** Program compatibility matrix + scope decision.

### IT-008: Distributed Topology and Network Architecture

**Question:** How does this work across machines, containers, and cloud instances?

**Why it matters:** Local Unix sockets don't cross machine boundaries. The distributed story needs its own design.

**What to investigate:**
- **Broker federation:** Broker A and Broker B connect over TCP/TLS. Messages route across.
- **NAT traversal:** Machines behind NAT can't accept connections. Need relay or hole-punching.
- **Container networking:** Docker containers, Kubernetes pods — how do sockets/connections work?
- **SSH tunneling:** `ssh -L` can forward Unix sockets. tmux over SSH is proven. Is SSH the transport for v1?
- **Cloud instances:** EC2, GCP VMs — public IPs, security groups, IAM. How does auth work?
- **Latency tolerance:** Local is <1ms. LAN is 1-10ms. WAN is 50-200ms. At what latency does "input injection" stop making sense?
- **Partition tolerance:** Machine B goes offline. What happens to queued messages?
- **Existing solutions:** How do Tailscale, WireGuard, Cloudflare Tunnel change the picture?
- **MCP over network:** MCP currently runs over stdio. There are proposals for HTTP/SSE transport. Could our distributed layer just be MCP-over-network?

**Expected output:** Network architecture document with deployment topology diagrams.

### IT-009: Relationship to MCP, LSP, and Existing Protocols

**Question:** Are we reinventing the wheel? Can we extend or compose existing protocols?

**Why it matters:** D4 (portability) says prefer standards. If MCP or another protocol already solves 80% of this, we should build on it, not beside it.

**What to investigate:**
- **MCP (Model Context Protocol):** JSON-RPC over stdio. Designed for AI agent tool use. Already has resources, tools, prompts. Could terminal sessions be MCP resources? Could `inject` be an MCP tool?
- **LSP (Language Server Protocol):** JSON-RPC over stdio. Similar transport. Different domain but same architectural pattern.
- **D-Bus:** Linux IPC standard. Session bus + system bus. Has discovery, naming, introspection. Not available on macOS natively.
- **gRPC:** Structured, typed, bidirectional streaming, built-in auth. Heavy but powerful.
- **NATS/ZeroMQ:** Lightweight message brokers. NATS has built-in clustering. ZeroMQ is embeddable.
- **The convergence thesis:** MCP + terminal sessions = an AI agent that can not only call tools but also operate inside real terminal environments. This might be the killer use case.

**Expected output:** Protocol comparison matrix + integration recommendation.

### IT-010: Agent-to-Agent Communication Patterns

**Question:** If two Claude Code sessions can talk to each other, what patterns emerge?

**Why it matters:** This is the highest-value application. The framework already has agent roles (coder, tester, reviewer). Cross-session communication could enable true multi-agent workflows.

**What to investigate:**
- **Delegation:** Orchestrator session assigns work to specialist sessions
- **Reporting:** Worker sessions report progress/results back to orchestrator
- **Peer review:** One session's output is piped to another for review
- **Shared context:** Multiple sessions working on the same codebase — how to coordinate git operations?
- **Conflict resolution:** Two agent sessions want to edit the same file. Who wins?
- **The fw framework connection:** Our task system already has `owner` fields. Could tasks be "owned" by a specific session and transferred between sessions?
- **Scaling limits:** At what point do you need a proper workflow engine (Temporal, Airflow) instead of terminal-to-terminal messaging?

**Expected output:** Pattern catalog with sequence diagrams.

### Investigation Priority Matrix

| ID | Topic | Dependency | Risk if Skipped | Suggested Order |
|----|-------|-----------|-----------------|:-:|
| IT-001 | Paradigm decision | None | Architecture built on wrong foundation | **1** |
| IT-009 | Protocol relationships | IT-001 | Reinvent the wheel | **2** |
| IT-003 | Message protocol | IT-001, IT-009 | Incompatible implementations | **3** |
| IT-002 | Session identity | IT-001 | No way to address messages | **4** |
| IT-004 | Output capture | IT-001 | Half-duplex only | **5** |
| IT-006 | Security model | IT-003 | Insecure by default | **6** |
| IT-005 | Concurrency | IT-003 | Race conditions under load | **7** |
| IT-007 | Interactive programs | IT-004 | Broken UX with non-shell targets | **8** |
| IT-008 | Distributed topology | IT-003, IT-006 | Local-only forever | **9** |
| IT-010 | Agent patterns | All above | Build without knowing the use cases (but we have intuition) | **10** |

## Dialogue Log

### 2026-03-08 — Initial exploration request
- **Human asked:** Can we investigate passing information between terminal sessions by injecting keyboard input, recognizing that terminal input is fundamentally a process/fd operation?
- **Core insight validated:** Yes — PTY architecture means keyboard input is just bytes on a file descriptor. Writing to the master side is indistinguishable from physical keystrokes.
- **Outcome:** Research documented, multiple viable mechanisms identified at different abstraction levels.

### 2026-03-08 — Directive mapping and distributed analysis
- **Human asked:** Map mechanisms against the four constitutional directives, with special attention to usability, security, and portability/distributed operation.
- **Key findings:** tmux and Unix sockets score top tier across all directives. TIOCSTI and osascript eliminated. Two-layer architecture recommended (Unix socket broker + tmux integration). Distributed operation viable via broker-to-broker TLS.
- **Outcome:** Consolidated scorecard produced. Clear recommendation: Unix socket broker as primary, tmux as terminal integration layer, TLS bridge for remote.

### 2026-03-08 — Deep reflection and investigation topic identification
- **Human asked:** Deep reflect on the document, identify gaps, and detail topics for further investigation.
- **Key insight:** The framing is wrong — we're not really building "input injection," we're building a message bus with terminal endpoints. Pure injection is a parlor trick; the value is in structured, bidirectional, acknowledged communication.
- **Critical gap found:** The "output capture problem" — we can inject input but have no story for capturing results. Half-duplex is useless for agent orchestration.
- **Abstraction inversion warning:** Wrapping tmux inside our broker inverts the abstraction. Need to decide: is tmux the broker, or is it one adapter among many?
- **10 investigation topics identified:** Paradigm decision (IT-001) through agent patterns (IT-010), dependency-ordered.
- **Outcome:** Investigation roadmap produced. IT-001 (paradigm) and IT-009 (protocol relationships) are highest priority — they determine whether we build something new or extend MCP/existing standards.
