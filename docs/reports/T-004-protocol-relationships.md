# T-004: Protocol Relationships — MCP, LSP, Existing Standards

## Question

Should TermLink extend MCP, compose with existing protocols, or build from scratch?

## Parent

T-003 (GO: message bus + injection adapter, control/data plane split)

## Context

T-003 decided on **message bus + injection adapter** with **control plane / data plane split**:
- **Control plane:** MCP-compatible (JSON-RPC 2.0) — session discovery, tool invocation, output snapshots. Transport: Unix socket (local), Streamable HTTP (remote).
- **Data plane:** Live output streaming + raw injection. Transport: Unix socket (local), WebSocket (remote). Length-prefixed frames, binary-safe.

This report investigates which existing protocols best serve each plane, and whether to extend, compose, or build from scratch.

---

## 1. MCP Deep Dive

### 1.1 Current Specification (2025-11-25)

The Model Context Protocol (MCP) is an open standard originally introduced by Anthropic in November 2024, donated to the Linux Foundation's Agentic AI Foundation (AAIF) in December 2025. The latest spec revision is 2025-11-25. As of early 2026, MCP has become the de facto standard for connecting AI agents to external tools and data.

**Ecosystem scale:** 97M+ monthly SDK downloads, 5,800+ MCP servers, 300+ MCP clients. Backed by Anthropic, OpenAI, Google, Microsoft, JetBrains, and others.

### 1.2 Core Primitives

| Primitive | Direction | Description |
|-----------|-----------|-------------|
| **Tools** | Server → Client | Functions the AI model can invoke. Request/response over JSON-RPC. |
| **Resources** | Server → Client | Structured data sources (URIs) the AI model or user can read. |
| **Prompts** | Server → Client | Templated message workflows for users. |
| **Sampling** | Client → Server | Server-initiated LLM interaction requests. Enables server-side agent loops. |
| **Roots** | Client → Server | Server-initiated filesystem/URI boundary queries. |
| **Elicitation** | Client → Server | Server-initiated requests for user input. |
| **Tasks** | Server → Client | Async long-running operations (2025-11-25, experimental). Call-now, fetch-later pattern with state machine (working → input_required → completed/failed/cancelled). |

### 1.3 Transport Options

| Transport | Spec Status | Use Case | Session Model |
|-----------|-------------|----------|---------------|
| **stdio** | Official | Local processes. Client spawns server as subprocess. | Process lifecycle |
| **Streamable HTTP** | Official (2025-03-26+) | Remote servers. POST for client→server, SSE for server→client streaming. | Session ID header, stateless-capable |
| **WebSocket** | Proposed (SEP-1288) | Long-lived bidirectional. Under active discussion, not yet in spec. | Persistent connection |
| **Unix socket** | Not in spec | Not addressed. Would need custom transport implementation. | N/A |

**Key transport facts:**
- SSE transport was deprecated in favor of Streamable HTTP (March 2025).
- The spec intentionally limits to two official transports: stdio (local) and Streamable HTTP (remote).
- The June 2026 spec revision may add WebSocket support.
- Unix socket transport would be a custom extension — MCP's transport layer is pluggable, so this is feasible but non-standard.

### 1.4 Extension Mechanisms

MCP is designed for extensibility:
- **Custom tools:** Any server can expose arbitrary tools with typed input/output schemas. TermLink can define `inject_input`, `send_signal`, `create_session`, `destroy_session` as standard MCP tools.
- **Custom resources:** URI-based resource system supports custom schemes. `session://terminal-1/output` is a valid pattern.
- **Custom resource types:** MIME types on resources enable typed content (text, binary via base64).
- **Capability negotiation:** Client and server negotiate supported features at initialization. TermLink-specific capabilities can be declared.
- **Notifications:** JSON-RPC notifications (fire-and-forget) are supported for server→client and client→server.

### 1.5 Mapping to TermLink Control Plane

| TermLink Operation | MCP Primitive | Mapping Quality |
|-------------------|---------------|-----------------|
| Discover sessions | Resource listing (`session://`) | Excellent |
| Get session info | Resource read | Excellent |
| Inject input | Tool (`inject_input`) | Good |
| Send signal (SIGINT, etc.) | Tool (`send_signal`) | Good |
| Create/destroy session | Tool | Good |
| Capture output snapshot | Resource read (`session://X/output`) | Good |
| Subscribe to session events | Notification + polling | Adequate |
| Stream live output | Not natively supported | **Poor** |
| Low-latency keystroke injection | Tool (JSON-RPC overhead ~1-5ms) | **Marginal** |
| Binary data transfer | Base64 encoding in JSON | **Poor** |

### 1.6 MCP Limitations for TermLink

1. **No true streaming:** MCP's Streamable HTTP uses SSE for server→client, but this is notification-based, not a continuous byte stream. Live terminal output at 60fps would overwhelm the JSON-RPC framing.

2. **JSON-RPC overhead:** Every operation is a full JSON-RPC request/response cycle. For keystroke injection at interactive speeds (10-50ms latency budget), the serialization overhead is significant.

3. **No binary-safe transport:** All data must be UTF-8 JSON. Binary terminal output (e.g., curses applications, image protocols like Sixel/Kitty) requires base64 encoding — 33% overhead plus encode/decode latency.

4. **Stateless HTTP design:** Streamable HTTP is designed for stateless request/response. Terminal sessions are inherently stateful, long-lived connections. Session IDs help, but the impedance mismatch adds complexity.

5. **No Unix socket transport:** For local IPC (our primary use case), MCP offers only stdio (requires parent/child process relationship). Unix sockets are not in the spec.

6. **Notification limitations:** Notifications are fire-and-forget (no acknowledgment). For reliable event delivery (session died, output ready), this is insufficient without additional application-level protocol.

### 1.7 Key Finding: Terminal Sessions as MCP Servers

This is viable and valuable. Existing implementations already exist:
- **terminal-mcp-server** (rinardnick): Exposes local and SSH command execution via MCP tools.
- **weidwonder/terminal-mcp-server**: Session persistence with environment variable support.

These prove the model works for **command execution** (control plane). They do NOT address streaming output or real-time interaction (data plane). This validates T-003's split architecture.

### 1.8 MCP Verdict

**Control plane: Strong fit (8/10).** MCP covers session discovery, tool invocation, output snapshots, and lifecycle management naturally. The ecosystem integration means any MCP client (Claude, ChatGPT, Cursor, custom agents) gets terminal access immediately.

**Data plane: Poor fit (2/10).** No streaming, no binary safety, JSON overhead. Not designed for this use case.

**Recommendation:** Use MCP as the control plane protocol. Do not force it into the data plane role.

---

## 2. Alternative Protocol Analysis

### 2.1 gRPC

**What it is:** Google's RPC framework. HTTP/2 transport, Protocol Buffers serialization, four communication patterns (unary, server-streaming, client-streaming, bidirectional streaming). Mature, widely adopted.

| Aspect | Assessment |
|--------|------------|
| **Control plane fit** | Excellent. Unary RPC maps to request/response. Service definitions provide typed contracts. |
| **Data plane fit** | Excellent. Bidirectional streaming is a first-class primitive. Binary-native (protobuf). Low overhead. |
| **Dependencies** | Heavy. Requires protobuf compiler, HTTP/2 stack, code generation. Runtime ~5-15MB depending on language. |
| **Portability** | Good across languages (Go, Rust, Python, TypeScript, C++). HTTP/2 requirement means no raw Unix socket — needs HTTP/2 framing even on local connections. |
| **Ecosystem** | Massive. Standard in microservices. But zero AI ecosystem overlap — no MCP client speaks gRPC natively. |
| **Local IPC** | Supported via Unix domain sockets with HTTP/2 framing, but adds unnecessary protocol overhead for local use. |

**Key strength:** Bidirectional streaming solves both planes in one protocol.
**Key weakness:** No MCP ecosystem integration. Agents would need a gRPC-to-MCP bridge, eliminating the "free integration" benefit. Heavy dependency chain for a CLI tool.

### 2.2 NATS

**What it is:** Lightweight, high-performance messaging system. Pub/sub, request/reply, queue groups. JetStream adds persistence and exactly-once delivery. Single Go binary, ~20MB RAM.

| Aspect | Assessment |
|--------|------------|
| **Control plane fit** | Good. Request/reply maps to RPC. Subject-based routing provides natural topic hierarchy (`termlink.session.X.command`). |
| **Data plane fit** | Excellent. Pub/sub with subjects like `termlink.session.X.output` provides natural streaming. Binary messages supported. |
| **Dependencies** | Requires running a NATS server process. Can be embedded in Go, but not in Rust/Python/TypeScript without running the server binary. |
| **Portability** | Excellent. Clients in every major language. NATS server runs on Linux, macOS, Windows. |
| **Ecosystem** | Strong in cloud-native/microservices. Zero AI ecosystem overlap. |
| **Discovery** | Built-in subject-based discovery. No need for a separate registry. |

**Key strength:** Natural pub/sub model maps perfectly to terminal output streaming. Subject hierarchy provides routing without custom code. JetStream adds replay (reconnect and catch up on missed output).
**Key weakness:** Requires a broker process. Even embedded, this is architectural overhead for what should be a lightweight CLI tool. Not in the MCP ecosystem.

### 2.3 ZeroMQ

**What it is:** Embeddable messaging library (not a broker). Multiple patterns: pub/sub, req/rep, push/pull, dealer/router. Transports: TCP, IPC (Unix socket), inproc (in-process), PGM (multicast).

| Aspect | Assessment |
|--------|------------|
| **Control plane fit** | Good. REQ/REP pattern maps to RPC. DEALER/ROUTER for async. But no built-in serialization — must add JSON-RPC or protobuf on top. |
| **Data plane fit** | Excellent. PUB/SUB over IPC (Unix socket) is exactly our local streaming pattern. Binary-native. Zero-copy capable. |
| **Dependencies** | libzmq C library (~1MB). Bindings exist for all major languages. |
| **Portability** | Excellent. Linux, macOS, Windows. IPC transport uses Unix sockets (Linux/macOS) or named pipes (Windows). |
| **Ecosystem** | Mature but shrinking community. No AI ecosystem overlap. |
| **Discovery** | None built-in. Must implement service discovery separately. |

**Key strength:** No broker needed. IPC transport uses Unix sockets natively. PUB/SUB pattern is ideal for output streaming. Minimal dependency.
**Key weakness:** No serialization format — must layer JSON-RPC (or equivalent) on top. No built-in discovery. Thread-safety issues require careful handling (though newer versions improved this). Community momentum has slowed.

### 2.4 nanomsg/nng

**What it is:** Spiritual successor to ZeroMQ. Cleaner API, thread-safe by design, pluggable transports including WebSocket and TLS. Same patterns (pub/sub, req/rep, push/pull, survey, bus).

| Aspect | Assessment |
|--------|------------|
| **Control plane fit** | Good. Same patterns as ZeroMQ. REQ/REP for RPC, SURVEY for discovery. |
| **Data plane fit** | Excellent. PUB/SUB, binary-native. WebSocket transport built-in (vs. ZeroMQ where it's an add-on). |
| **Dependencies** | C library, smaller than libzmq. Actively maintained. |
| **Portability** | Excellent. Linux, macOS, Windows. IPC, TCP, WebSocket, TLS transports all built-in. |
| **Ecosystem** | Small. Fewer language bindings than ZeroMQ. Rust binding (nng-rs) exists but less battle-tested. |
| **Discovery** | SURVEY pattern provides built-in discovery (ask all peers, collect responses). |

**Key strength:** Thread-safe. Built-in WebSocket + TLS. SURVEY pattern is natural for session discovery. Cleaner API than ZeroMQ.
**Key weakness:** Smaller ecosystem, fewer language bindings. 10% adoption increase over 2 years is slow growth. Less community support if issues arise.

### 2.5 D-Bus

**What it is:** Linux IPC standard. Session bus and system bus. Built-in introspection, naming, signal/method model. Used by every Linux desktop environment.

| Aspect | Assessment |
|--------|------------|
| **Control plane fit** | Good for Linux. Method calls map to RPC. Signal model maps to events. Introspection is powerful for discovery. |
| **Data plane fit** | Poor. 2.5x overhead over direct IPC. Not designed for streaming. Message size limits. |
| **Dependencies** | Requires D-Bus daemon (standard on Linux, available but non-standard on macOS, barely functional on Windows). |
| **Portability** | **Deal-breaker.** Linux-only in practice. macOS/Windows support is technically possible but not reliable. |
| **Ecosystem** | Linux desktop. Zero overlap with AI or cross-platform tooling. |

**Key strength:** Rich introspection and discovery on Linux.
**Key weakness:** Linux-only. Not cross-platform. Performance overhead. Not designed for streaming. **Eliminated by D4 (Portability).**

### 2.6 LSP (Language Server Protocol)

**What it is:** JSON-RPC 2.0 over stdio. Defines a standard protocol for language intelligence features (completion, diagnostics, hover). Created by Microsoft for VS Code.

| Aspect | Assessment |
|--------|------------|
| **Control plane fit** | Architecturally similar to MCP. JSON-RPC 2.0 base. But domain-specific to programming languages — no terminal primitives. |
| **Data plane fit** | Poor. Same limitations as MCP (JSON-RPC, no binary, no streaming). |
| **Dependencies** | None beyond JSON-RPC implementation. |
| **Portability** | Excellent. JSON-RPC over stdio works everywhere. |
| **Ecosystem** | Massive in IDE tooling. Zero overlap with terminal management or AI agents. |

**Key finding:** LSP and MCP share the same transport DNA (JSON-RPC 2.0 over stdio). MCP is explicitly inspired by LSP. For TermLink, MCP is strictly superior because it operates in our domain (AI tools) rather than LSP's domain (language intelligence). There is no reason to use LSP when MCP exists.

### 2.7 Cap'n Proto

**What it is:** Zero-copy serialization + RPC system. Created by former protobuf author (Kenton Varda). No serialization/deserialization step — the in-memory format IS the wire format. 64-bit aligned data.

| Aspect | Assessment |
|--------|------------|
| **Control plane fit** | Good. RPC system with promise pipelining (chain calls without waiting). Typed interfaces. |
| **Data plane fit** | Good. Binary-native, zero-copy means minimal overhead for streaming. But streaming support is less mature than gRPC. |
| **Dependencies** | C++ core library. Language bindings: Rust (excellent), Go, Python, TypeScript (varying maturity). Code generation required. |
| **Portability** | Moderate. C++ dependency can be challenging on some platforms. Rust support is strong. |
| **Ecosystem** | Small but high-quality. Used by Cloudflare Workers (workerd), Thanos. Not widely adopted. |

**Key strength:** Zero-copy eliminates serialization overhead. Promise pipelining reduces round trips. Excellent Rust support.
**Key weakness:** Small ecosystem. Code generation adds build complexity. Streaming less proven than gRPC. No AI ecosystem overlap.

### 2.8 Protocol Comparison Matrix

| Protocol | Control Plane | Data Plane | Dependencies | Portability | AI Ecosystem | Overall |
|----------|:---:|:---:|:---:|:---:|:---:|:---:|
| **MCP** | 8 | 2 | Low | High | **10** | Control only |
| **gRPC** | 9 | 9 | High | Good | 0 | Both, but heavy |
| **NATS** | 7 | 9 | Medium (broker) | High | 0 | Data plane |
| **ZeroMQ** | 6 | 9 | Low | High | 0 | Data plane |
| **nng** | 6 | 9 | Low | High | 0 | Data plane |
| **D-Bus** | 7 | 3 | Medium | **1** | 0 | Eliminated |
| **LSP** | 5 | 2 | Low | High | 0 | Superseded by MCP |
| **Cap'n Proto** | 8 | 7 | Medium | Moderate | 0 | Niche |

---

## 3. Composition Patterns

### 3.1 MCP Control Plane + Custom Length-Prefixed Data Plane

```
Control: MCP (JSON-RPC 2.0) over Unix socket / Streamable HTTP
Data:    Custom length-prefixed binary frames over Unix socket / WebSocket
```

**How it works:** The MCP control plane handles all structured operations (discover, inject, signal, lifecycle). When a client needs live output streaming or real-time injection, the control plane returns a data plane endpoint. The client opens a separate connection to the data plane using a simple length-prefixed binary protocol.

**Frame format example:**
```
[4 bytes: length][1 byte: type][1 byte: flags][N bytes: payload]
Types: 0x01=output, 0x02=input, 0x03=resize, 0x04=signal, 0xFF=keepalive
```

| Directive | Score | Rationale |
|-----------|:-----:|-----------|
| D1 Antifragile | 9 | Two independent planes. Control works without data. No external dependencies to fail. Custom protocol is fully understood — no black-box library behavior. |
| D2 Reliable | 8 | MCP provides typed contracts. Custom data plane is simple enough to reason about completely. No hidden complexity from library internals. |
| D3 Usable | 7 | MCP ecosystem integration is automatic. Data plane requires custom client implementation in each language. But the protocol is trivial to implement (~100 lines). |
| D4 Portable | 9 | Zero external dependencies beyond standard library. Unix sockets + WebSocket are POSIX/browser standards. No language or platform lock-in. |
| **Total** | **33** | |

**Pros:** Minimal dependencies. Full control over wire format. Binary-safe. MCP ecosystem integration. Simple to understand, debug, and maintain.
**Cons:** Must implement data plane client in each language. No existing library ecosystem for the custom protocol. Must handle flow control, backpressure, reconnection ourselves.

### 3.2 MCP Control Plane + ZeroMQ Data Plane

```
Control: MCP (JSON-RPC 2.0) over Unix socket / Streamable HTTP
Data:    ZeroMQ PUB/SUB over IPC (Unix socket) / TCP
```

**How it works:** Same control plane as 3.1. Data plane uses ZeroMQ's PUB/SUB pattern — each terminal session publishes output on a ZeroMQ PUB socket, subscribers connect to receive. Input injection uses PUSH/PULL or REQ/REP.

| Directive | Score | Rationale |
|-----------|:-----:|-----------|
| D1 Antifragile | 8 | Two independent planes. ZeroMQ's reconnection and message queuing add resilience. But libzmq is a complex C library — failure modes are harder to diagnose. |
| D2 Reliable | 7 | PUB/SUB drops messages when subscriber is slow or disconnected (by design). Need XPUB/XSUB or push/pull for guaranteed delivery. ZeroMQ's internal state machine is complex. |
| D3 Usable | 7 | Mature library with good docs. But ZeroMQ has a steep learning curve (socket types, patterns, context management). Debugging ZeroMQ issues is notoriously difficult. |
| D4 Portable | 7 | libzmq available on all platforms, but it's a C dependency that complicates builds. Language bindings vary in quality and maintenance. |
| **Total** | **29** | |

**Pros:** Battle-tested messaging library. Built-in message queuing. Multiple transport options. No broker needed.
**Cons:** C library dependency complicates builds. PUB/SUB drops messages. Debugging is difficult. Community momentum declining. Thread-safety concerns in older versions.

### 3.3 MCP Control Plane + nng Data Plane

```
Control: MCP (JSON-RPC 2.0) over Unix socket / Streamable HTTP
Data:    nng PUB/SUB over IPC / WebSocket
```

**How it works:** Same as ZeroMQ composition but using nng. Built-in WebSocket transport eliminates the need for a separate WebSocket library for remote data plane.

| Directive | Score | Rationale |
|-----------|:-----:|-----------|
| D1 Antifragile | 8 | Thread-safe design reduces failure modes vs ZeroMQ. Built-in WebSocket/TLS avoids additional dependency failures. |
| D2 Reliable | 7 | Same PUB/SUB message-drop semantics as ZeroMQ. SURVEY pattern aids discovery. |
| D3 Usable | 6 | Cleaner API than ZeroMQ, but smaller community means fewer examples, less Stack Overflow coverage, harder debugging. |
| D4 Portable | 7 | C library, similar build concerns as ZeroMQ. Fewer language bindings — Rust binding exists but less mature. |
| **Total** | **28** | |

**Pros:** Thread-safe. Built-in WebSocket. SURVEY pattern for discovery. Cleaner API than ZeroMQ.
**Cons:** Smaller ecosystem. Fewer language bindings. Less battle-tested. Same C dependency concern.

### 3.4 MCP Control Plane + NATS Data Plane

```
Control: MCP (JSON-RPC 2.0) over Unix socket / Streamable HTTP
Data:    NATS PUB/SUB over TCP
```

**How it works:** Terminal sessions publish output on NATS subjects (`termlink.session.{id}.output`). Clients subscribe. JetStream provides replay for reconnecting clients.

| Directive | Score | Rationale |
|-----------|:-----:|-----------|
| D1 Antifragile | 7 | NATS server is a single point of failure. JetStream adds replay resilience. But introducing a mandatory broker process is a significant failure surface. |
| D2 Reliable | 9 | JetStream provides at-least-once and exactly-once delivery. Subject-based routing is robust. Message acknowledgment built-in. |
| D3 Usable | 8 | Excellent documentation. Subject hierarchy is intuitive. JetStream adds complexity but is well-documented. |
| D4 Portable | 6 | NATS server is a Go binary — adds infrastructure dependency. Cannot embed in non-Go projects. Users must install and run a NATS server. |
| **Total** | **30** | |

**Pros:** Excellent delivery guarantees. Natural subject hierarchy. JetStream replay for reconnection. Great documentation.
**Cons:** **Requires running a broker process.** This is the fundamental problem — TermLink should be a lightweight CLI tool, not a service with infrastructure requirements. The NATS server adds operational overhead that contradicts our usability goals.

### 3.5 Pure gRPC (Both Planes)

```
Control: gRPC unary/server-streaming RPCs
Data:    gRPC bidirectional streaming
```

**How it works:** Single gRPC service definition covers everything. Unary RPCs for commands, bidirectional streaming for live I/O. Protobuf for all serialization.

| Directive | Score | Rationale |
|-----------|:-----:|-----------|
| D1 Antifragile | 7 | Single protocol is simpler topology. But gRPC's HTTP/2 stack is complex — failure modes in the transport layer are opaque. |
| D2 Reliable | 9 | Protobuf typed contracts. Streaming with flow control. Deadline propagation. Error codes. |
| D3 Usable | 5 | Protobuf compilation step. Code generation. Heavy toolchain. And critically: **zero MCP ecosystem integration**. Every AI agent would need a gRPC adapter. |
| D4 Portable | 6 | Works across languages, but HTTP/2 dependency adds weight. No MCP compatibility means building a parallel integration ecosystem from scratch. |
| **Total** | **27** | |

**Pros:** One protocol for everything. Excellent streaming. Typed contracts. Flow control.
**Cons:** **No MCP ecosystem integration.** This is the killer. Every MCP client (Claude, ChatGPT, Cursor, etc.) would need a gRPC-to-MCP bridge. Heavy dependency chain (protobuf compiler, HTTP/2 stack). Overkill for local Unix socket IPC.

### 3.6 Full Custom on JSON-RPC 2.0

```
Control: JSON-RPC 2.0 (MCP-compatible subset)
Data:    JSON-RPC 2.0 with streaming notifications
```

**How it works:** Both planes use JSON-RPC 2.0 over the same connection. Data plane uses JSON-RPC notifications for streaming, with binary data base64-encoded.

| Directive | Score | Rationale |
|-----------|:-----:|-----------|
| D1 Antifragile | 6 | Single connection simplifies topology. But forcing streaming through JSON-RPC creates fragility — notification storms, no backpressure, no flow control. |
| D2 Reliable | 5 | JSON-RPC notifications have no acknowledgment. No flow control for streaming. Base64 encoding adds 33% overhead. |
| D3 Usable | 8 | Single protocol. MCP-compatible. Easy to understand and debug (human-readable JSON). |
| D4 Portable | 9 | JSON-RPC libraries exist in every language. No binary dependencies. |
| **Total** | **28** | |

**Pros:** Single protocol. MCP compatible. Human-readable. Zero binary dependencies.
**Cons:** JSON-RPC is not designed for streaming. Base64 overhead for binary. No backpressure. Notification storms under heavy output. Forces MCP into a role it wasn't designed for.

### 3.7 Composition Pattern Comparison

| Pattern | D1 | D2 | D3 | D4 | Total | Key Trade-off |
|---------|:--:|:--:|:--:|:--:|:-----:|---------------|
| **MCP + Custom frames** | 9 | 8 | 7 | 9 | **33** | Must implement data plane ourselves |
| MCP + NATS | 7 | 9 | 8 | 6 | 30 | Requires broker infrastructure |
| MCP + ZeroMQ | 8 | 7 | 7 | 7 | 29 | C library dependency, declining community |
| MCP + nng | 8 | 7 | 6 | 7 | 28 | Small ecosystem, fewer bindings |
| Full custom JSON-RPC | 6 | 5 | 8 | 9 | 28 | Forcing streaming through JSON-RPC |
| Pure gRPC | 7 | 9 | 5 | 6 | 27 | No MCP ecosystem, heavy deps |

---

## 4. Constitutional Directive Scoring

### 4.1 D1 — Antifragility

**Best:** MCP + Custom frames (9/10)
- Two independent planes with zero shared dependencies
- Custom data plane is fully understood — no black-box library behavior
- Control plane degrades gracefully (tools work without streaming)
- Data plane degrades gracefully (streaming works without control commands)
- No external service to crash (broker, daemon)

**Worst:** Full custom JSON-RPC (6/10) — single protocol creates correlated failures; streaming overload kills control plane.

### 4.2 D2 — Reliability

**Best:** Pure gRPC (9/10) and MCP + NATS (9/10)
- gRPC: typed contracts, flow control, deadline propagation
- NATS: JetStream guarantees, acknowledgments, replay

**Runner-up:** MCP + Custom frames (8/10) — must implement reliability ourselves, but the simplicity makes it achievable. MCP provides typed contracts for control plane. Data plane reliability (reconnection, missed-message handling) must be designed.

### 4.3 D3 — Usability

**Best:** MCP + NATS (8/10) and Full custom JSON-RPC (8/10)
- NATS: excellent docs, intuitive subject hierarchy
- JSON-RPC: human-readable, single protocol

**MCP + Custom frames (7/10):** Requires implementing data plane clients in each language. But the protocol is simple enough that a reference implementation is ~100-200 lines. MCP control plane integration is automatic.

### 4.4 D4 — Portability

**Best:** MCP + Custom frames (9/10) and Full custom JSON-RPC (9/10)
- Zero binary dependencies beyond standard library
- Unix sockets + WebSocket are universal standards
- No language, platform, or vendor lock-in

**Worst:** Pure gRPC (6/10) and MCP + NATS (6/10) — gRPC requires HTTP/2 stack + protobuf toolchain; NATS requires installing and running a Go binary.

---

## 5. Recommendation

### Decision: MCP Control Plane + Custom Length-Prefixed Data Plane

**This is Option 3.1, scoring 33/40 across constitutional directives.**

### Rationale

1. **MCP ecosystem integration is non-negotiable.** The AI agent ecosystem has standardized on MCP. Claude, ChatGPT, Cursor, and hundreds of other clients speak MCP natively. Building terminal sessions as MCP servers means every AI agent gets terminal orchestration for free. No other protocol offers this — gRPC, NATS, and ZeroMQ all require building a parallel integration ecosystem.

2. **A custom data plane is simpler than it sounds.** The data plane protocol is: length-prefixed binary frames over Unix socket (local) or WebSocket (remote). This is ~100-200 lines of code per language. The simplicity makes it fully auditable, debuggable, and portable. Every alternative (ZeroMQ, nng, NATS) adds a dependency that is orders of magnitude more complex than the protocol it replaces.

3. **No broker, no daemon, no infrastructure.** TermLink should be `pip install termlink` or `brew install termlink`, not "install NATS, start the server, configure JetStream, then install termlink." Zero infrastructure requirements.

4. **The two-plane architecture maps to two distinct concerns.** Control plane = structured, typed, request/response, moderate latency acceptable. Data plane = streaming, binary, low-latency, high-throughput. These have fundamentally different requirements. Forcing them into one protocol (gRPC, full JSON-RPC) creates impedance mismatch. Separating them with purpose-built transports is the cleanest architecture.

5. **Constitutional directive alignment.** Highest score on D1 (antifragile — independent planes, no black-box dependencies), D4 (portable — zero binary deps, POSIX standards), and competitive on D2 (reliable — must design, but simple enough to get right) and D3 (usable — MCP integration automatic, data plane simple).

### What We're Building

```
┌──────────────────────────────────────────────────────────────┐
│                     CONTROL PLANE                             │
│              MCP-compatible (JSON-RPC 2.0)                    │
│                                                               │
│  Tools: inject_input, send_signal, create/destroy_session     │
│  Resources: session://{id}/output, session://{id}/info        │
│  Transport: Unix socket (local), Streamable HTTP (remote)     │
│  Extension: TermLink-specific capability negotiation          │
│                                                               │
│  Any MCP client gets terminal orchestration for free.         │
└───────────────────────────┬──────────────────────────────────┘
                            │ (data plane endpoint returned
                            │  via control plane tool call)
┌───────────────────────────┴──────────────────────────────────┐
│                      DATA PLANE                               │
│           Custom length-prefixed binary frames                │
│                                                               │
│  Frame: [4B length][1B type][1B flags][payload]               │
│  Types: output, input, resize, signal, keepalive              │
│  Transport: Unix socket (local), WebSocket (remote)           │
│  Properties: Binary-safe, low-latency, flow control           │
│                                                               │
│  Activated only when streaming is needed.                     │
└──────────────────────────────────────────────────────────────┘
```

### What We're NOT Building

- **Not a gRPC service.** Too heavy, no MCP ecosystem.
- **Not a NATS-based system.** Don't want broker infrastructure.
- **Not extending MCP for streaming.** Respect MCP's design boundaries; don't force it into roles it wasn't designed for.
- **Not using ZeroMQ/nng.** The data plane protocol is simpler than the library we'd import.

### Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Custom data plane has bugs we'd avoid with a library | Keep protocol minimal (~6 frame types). Fuzz test extensively. The simplicity IS the safety. |
| MCP Unix socket transport is non-standard | MCP's transport layer is pluggable. Implement a thin adapter. If MCP adds Unix socket support later, swap in. |
| Flow control in custom protocol | Implement simple window-based flow control (credit system). This is well-understood engineering, not novel research. |
| MCP spec changes break us | Pin to spec version. Capability negotiation provides forward compatibility. MCP is governed by Linux Foundation — breaking changes are unlikely. |

### Assumptions Validated

- **A-001 (MCP as control plane): VALIDATED.** MCP maps naturally to session discovery, tool invocation, lifecycle management. Existing terminal MCP servers prove the model.
- **A-002 (MCP limitations require separate data plane): VALIDATED.** No streaming, no binary safety, JSON overhead. Confirmed by spec analysis and ecosystem review.
- **A-003 (No single protocol covers both planes): VALIDATED.** gRPC comes closest but sacrifices MCP ecosystem integration. Every option has significant trade-offs for at least one plane.
- **A-004 (MCP ecosystem maturity): VALIDATED.** 97M+ monthly SDK downloads, 5,800+ servers, Linux Foundation governance. Stable enough to build on.

---

## Dialogue Log

### 2026-03-08 — Investigation started
- **Approach:** Sub-agent research dispatch for protocol analysis

### 2026-03-08 — Research complete
- **MCP deep dive:** Spec analysis (2025-11-25), transport options, extension mechanisms, ecosystem maturity, terminal MCP server precedents
- **Protocol analysis:** gRPC, NATS, ZeroMQ, nng, D-Bus, LSP, Cap'n Proto evaluated on both planes
- **Composition analysis:** Six composition patterns scored against four constitutional directives
- **Outcome:** MCP control plane + custom length-prefixed data plane recommended (33/40 directive score)
- **Key insight:** MCP ecosystem integration is the decisive factor. No other protocol gives free access to the entire AI agent ecosystem. The data plane is simple enough that a library dependency would add more complexity than it removes.
