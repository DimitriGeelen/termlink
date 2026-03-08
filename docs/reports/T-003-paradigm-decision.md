# T-003: Paradigm Decision — Injection vs Message Bus vs Hybrid

## Question

Are we building terminal input injection, a message bus with terminal endpoints, or a hybrid?

## Parent

T-002 (inception, GO decision). This is the first and most foundational investigation topic (IT-001).

## Why This Matters

Every subsequent design decision (protocol, identity, security, distribution) depends on the paradigm. A message bus needs framing, routing, ack/nack. Injection needs PTY access and keystroke encoding. A hybrid needs a clean interface between modes.

## Research Areas

Three parallel investigations:

1. **Use-case mapping** — Which real use cases need injection, which need messaging, which need both?
2. **MCP/existing protocol fit** — Can MCP or another existing protocol serve as the foundation?
3. **Prior art deep dive** — How do existing tools (tmux, Zellij, Wezterm, Expect, reptyr) handle this?

## Findings

### 1. Use-Case Paradigm Mapping (12 use cases analyzed)

| # | Use Case | Paradigm | Bidirectional | Ack? |
|---|----------|----------|:---:|:---:|
| 1 | Agent orchestration | Messaging | Yes | Yes |
| 2 | Parallel test dispatch | Messaging | Yes | Yes |
| 3 | Live pair programming | Injection | No | No |
| 4 | Automated CI scripting | Hybrid | Yes | Yes |
| 5 | Session context sharing | Messaging | Yes | No |
| 6 | Remote assistance | Injection | No | No |
| 7 | Multi-service startup | Messaging | Yes | Yes |
| 8 | REPL interaction | Hybrid | Yes | Yes |
| 9 | Chat between sessions | Messaging | Yes | No |
| 10 | File transfer notification | Messaging | No | No |
| 11 | Shared task queue | Messaging | Yes | Yes |
| 12 | Health monitoring | Messaging | Yes | Yes |

**Distribution:** 58% pure messaging, 17% hybrid, 17% pure injection, 8% notification.

**Signal:** Messaging-first architecture covers 10/12 use cases natively. Injection is needed only for two niche scenarios (live keystroke forwarding, remote assistance) and as a fallback for interactive programs without structured APIs.

### 2. MCP Protocol Fit Analysis

**Fit score: 5/10**

| Terminal Operation | MCP Primitive | Fit |
|---|---|---|
| Inject input | Tool (`inject_input`) | Good |
| Capture output snapshot | Resource (`terminal://session/output`) | Good |
| Discover sessions | Resource listing | Good |
| Send signal (SIGINT) | Tool (`send_signal`) | Good |
| Session lifecycle | Tool (`create/destroy_session`) | Good |
| **Stream live output** | Notification (poll-based) | **Weak** |
| **Low-latency injection** | Tool (JSON-RPC overhead) | **Weak** |
| **Stateful connections** | Streamable HTTP (stateless) | **Weak** |

**Key gaps:** No true output streaming (notify-then-poll), JSON-RPC latency overhead for real-time, stateless HTTP fighting stateful terminal sessions, binary data needs encoding.

**Recommendation:** "Build on MCP transport, extend for streaming" — Use MCP as the **control plane** (session discovery, tool invocation, output snapshots). Add a **streaming data plane** alongside (WebSocket/ZeroMQ sidecar for live output).

**Alternative protocol comparison:**

| Protocol | Fit | Key Gap |
|----------|-----|---------|
| gRPC | High | No AI ecosystem integration |
| NATS | High | Requires broker infrastructure |
| ZeroMQ | High | No discovery, manual serialization |
| D-Bus | Medium | Linux-only |
| LSP | Low | Wrong domain |

### 3. Prior Art Deep Dive

| Tool | Paradigm | Transport | Bidirectional | Structured | Distributed |
|------|----------|-----------|:---:|:---:|:---:|
| tmux | Injection | Unix socket + PTY | No | No | No |
| Zellij | Hybrid | Unix socket + protobuf | Yes | Yes | No |
| Wezterm | Hybrid | Mux, Unix/TLS | Yes | Partial | Yes |
| kitty | Messaging | Unix socket / escape seq | Yes | Yes (JSON) | No |
| Expect/pexpect | Injection | PTY | Yes (send+expect) | No | No |
| reptyr | PTY hijack | ptrace + PTY | N/A | No | No |
| abduco/dtach | PTY relay | Unix socket + PTY | Yes | No | No |
| NATS | Messaging | TCP/WebSocket | Yes | Yes | Yes |
| ZeroMQ | Messaging | TCP/IPC | Yes | Partial | Yes |

**Convergent pattern discovered:** Every tool that matured moved from PTY injection to **control plane / data plane separation over Unix sockets with typed messages:**

- **kitty:** JSON commands over Unix socket, separate from PTY stream. The cleanest example.
- **Zellij:** Protobuf over Unix socket with WASM plugin event system.
- **Wezterm:** Lua scripting + TLS domains for distribution.

**Key insight:** PTY injection is a dead end for inter-agent communication. tmux `send-keys` and Expect both conflate control and data on the same PTY channel — no ack, no error handling, no typing. The modern pattern separates them.

---

## Synthesis

Three independent research streams converge on the same answer:

### The Evidence

| Source | Finding |
|--------|---------|
| Use cases | 75% need messaging; injection is niche |
| MCP analysis | Control plane maps well; needs streaming data plane alongside |
| Prior art | Every maturing tool converges on Unix socket + typed messages, separate from PTY |

### The Architecture

```
┌─────────────────────────────────────────────────────┐
│                   CONTROL PLANE                      │
│         MCP-compatible (JSON-RPC 2.0)                │
│  Tools: inject, signal, create/destroy session       │
│  Resources: session list, output snapshots            │
│  Transport: Unix socket (local), Streamable HTTP     │
│             (remote)                                  │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────┴──────────────────────────────┐
│                    DATA PLANE                         │
│         Live output streaming + raw injection         │
│  Transport: Unix socket (local), WebSocket (remote)   │
│  Protocol: Length-prefixed frames, binary-safe         │
│  For: Real-time output, keystroke injection,          │
│       REPL interaction                                │
└─────────────────────────────────────────────────────┘
```

**Control plane (MCP):** Structured, typed, request/response. Handles orchestration, discovery, lifecycle, command execution. Any MCP client (Claude, Codex, custom agents) gets terminal access for free.

**Data plane (custom, lightweight):** Streaming, low-latency, binary-safe. Handles live output, raw keystroke injection, REPL interaction. Activated only when needed.

### Why This Works

| Directive | How it aligns |
|-----------|---------------|
| D1 Antifragile | Two planes degrade independently. Control plane works without data plane. Injection fallback for non-cooperative targets. |
| D2 Reliable | MCP gives typed contracts, request IDs, error codes. Data plane is observable via control plane queries. |
| D3 Usable | Developers interact via MCP tools (structured). Only drop to data plane for real-time scenarios. |
| D4 Portable | MCP is an open standard with growing ecosystem. Data plane uses POSIX sockets. No vendor lock-in. |

## Decision

**Paradigm: Message bus with injection adapter (control plane / data plane split)**

- **Primary:** Structured message bus using MCP-compatible protocol over Unix sockets
- **Secondary:** Streaming data plane for real-time output and raw injection
- **NOT building:** A general-purpose terminal multiplexer (tmux/Zellij do that), nor a pure keystroke injector
- **Integration point:** Terminal sessions as MCP servers — any MCP client gets terminal orchestration

This is a **message bus that happens to be able to inject keystrokes**, not a keystroke injector that grew messaging capabilities. The data flows through typed channels; injection is a specific tool call, not the foundation.

## Dialogue Log

### 2026-03-08 — Investigation started
- **Human directive:** "Let's go multi-agent where it makes sense"
- **Approach:** Parallel sub-agent dispatch for three independent research areas

### 2026-03-08 — Research synthesis complete
- **Three parallel agents returned:** Use-case mapping, MCP fit analysis, prior art deep dive
- **Convergence:** All three streams independently point to message bus with injection as an adapter
- **Key discovery:** Control plane / data plane separation is the convergent pattern across kitty, Zellij, and Wezterm
- **MCP fit:** 5/10 as-is, but "build on MCP, extend for streaming" is the recommendation
- **Decision:** Message bus (MCP-compatible control plane) + streaming data plane. Terminal sessions as MCP servers.
