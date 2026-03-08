# T-005: Message Protocol Design — Research Report

## Question

What is the wire format, message types, envelope fields, framing, and versioning for TermLink?

## Parent

T-003 (GO: message bus + injection adapter, control/data plane split)

## Research Areas

1. Message type taxonomy — mapping use cases to concrete types
2. Envelope design — required and optional fields
3. Framing analysis — newline JSON vs length-prefixed vs MessagePack vs Protobuf vs CBOR vs Cap'n Proto
4. Special key encoding — Ctrl+C, arrow keys, escape sequences
5. Versioning strategy
6. Protocol specification draft (v0.1)

---

## Findings

### 1. Message Type Taxonomy

Starting from the 12 use cases identified in T-002/T-003, the goal is to find the minimum set of message types that covers all scenarios.

#### Use Case to Message Type Mapping

| # | Use Case | Paradigm | Required Message Types |
|---|----------|----------|----------------------|
| 1 | Agent orchestration | Messaging | `command.execute`, `query.status`, `event.state_change` |
| 2 | Parallel test dispatch | Messaging | `command.execute`, `query.status`, `event.output` |
| 3 | Live pair programming | Injection | `command.inject`, `data.stream` |
| 4 | Automated CI scripting | Hybrid | `command.execute`, `command.inject`, `query.output`, `event.output` |
| 5 | Session context sharing | Messaging | `query.output`, `query.capabilities` |
| 6 | Remote assistance | Injection | `command.inject`, `data.stream`, `command.signal` |
| 7 | Multi-service startup | Messaging | `command.execute`, `query.status`, `event.state_change` |
| 8 | REPL interaction | Hybrid | `command.inject`, `data.stream`, `query.output` |
| 9 | Chat between sessions | Messaging | `data.transfer` (text blob) |
| 10 | File transfer notification | Messaging | `event.state_change` |
| 11 | Shared task queue | Messaging | `command.execute`, `query.status` |
| 12 | Health monitoring | Messaging | `session.heartbeat`, `query.status` |

#### Minimum Viable Message Set

After mapping, the minimum set that covers all 12 use cases is **14 message types** organized into 5 categories:

**Control Messages (4)** — Session lifecycle management (control plane)

| Type | Direction | Ack | Purpose |
|------|-----------|-----|---------|
| `session.register` | Client -> Hub | Yes | Register a terminal session with the hub |
| `session.deregister` | Client -> Hub | Yes | Graceful session departure |
| `session.discover` | Client -> Hub | Yes | List available sessions and their capabilities |
| `session.heartbeat` | Bidirectional | Yes | Liveness probe with optional status payload |

**Command Messages (3)** — Actions to perform in a target session (control plane)

| Type | Direction | Ack | Purpose |
|------|-----------|-----|---------|
| `command.execute` | Client -> Target | Yes | Execute a structured command (shell command string, env vars, working dir) |
| `command.inject` | Client -> Target | No | Inject raw bytes into target PTY (keystrokes, escape sequences) |
| `command.signal` | Client -> Target | Yes | Send a POSIX signal (SIGINT, SIGTERM, SIGTSTP, etc.) |

**Query Messages (3)** — Request information from a session (control plane)

| Type | Direction | Ack | Purpose |
|------|-----------|-----|---------|
| `query.status` | Client -> Target | Yes | Query process status (running, exit code, PID) |
| `query.output` | Client -> Target | Yes | Request output snapshot (last N lines of scrollback) |
| `query.capabilities` | Client -> Target | Yes | Query session capabilities (shell type, features, dimensions) |

**Event Messages (2)** — Asynchronous notifications (control plane, but may be high-volume)

| Type | Direction | Ack | Purpose |
|------|-----------|-----|---------|
| `event.state_change` | Target -> Subscribers | No | Session state changed (started, exited, resized) |
| `event.error` | Any -> Any | No | Error notification (non-fatal, informational) |

**Data Messages (2)** — Binary/streaming data (data plane)

| Type | Direction | Ack | Purpose |
|------|-----------|-----|---------|
| `data.stream` | Target -> Subscriber | No | Live output stream (binary-safe terminal output) |
| `data.transfer` | Any -> Any | Yes | Bulk data transfer (file contents, large text blobs) |

#### Coverage Analysis

Every use case is covered by 2-4 message types from this set. No use case requires a message type outside this set. The 14-type count is within the go/no-go threshold of <15.

#### Types Considered and Rejected

- **`command.batch`** — Execute multiple commands atomically. Rejected: adds complexity; achievable by sequencing `command.execute` with `correlation_id`. Revisit in v0.2 if demand emerges.
- **`event.output`** — Originally proposed as a control-plane output event. Merged into `data.stream` on the data plane, since live output is inherently a streaming concern. Control-plane output retrieval is handled by `query.output` (snapshot).
- **`session.subscribe` / `session.unsubscribe`** — Explicit pub/sub for events. Deferred: session registration can include subscription interests. Explicit subscription management adds protocol complexity without v0.1 value.

---

### 2. Envelope Design

#### Control Plane Envelope (JSON-RPC 2.0 Compatible)

The control plane uses JSON-RPC 2.0 as its wire format. This gives us free compatibility with every MCP client and JSON-RPC library.

##### Request Envelope

```json
{
  "jsonrpc": "2.0",
  "method": "command.execute",
  "id": "01J7K9MXYZ...",
  "params": {
    "target": "session-build-01",
    "sender": "session-orchestrator",
    "timestamp": "2026-03-08T15:30:00.123Z",
    "correlation_id": "corr-abc123",
    "ttl": 30,
    "payload": {
      "command": "npm test",
      "cwd": "/app",
      "env": { "CI": "true" }
    }
  }
}
```

##### Field Justification

| Field | Required | Type | Justification |
|-------|----------|------|---------------|
| `jsonrpc` | Yes | `"2.0"` | JSON-RPC 2.0 compliance. Enables any JSON-RPC client to speak the protocol. |
| `method` | Yes | string | Message type routing. Format: `category.action`. |
| `id` | Conditional | string | Required for request/response. Omitted for notifications (JSON-RPC 2.0 rule). Use ULIDs (sortable, unique, timestamp-embedded). |
| `params.target` | Yes | string | Routing destination. Session name or `"*"` for broadcast. Without this, the hub cannot route. |
| `params.sender` | Yes | string | Origin session. Enables reply routing and audit trails. |
| `params.timestamp` | Yes | ISO-8601 | Ordering, staleness detection, debugging. Millisecond precision. |
| `params.correlation_id` | No | string | Links related messages (e.g., command and its result). Essential for multi-step workflows. |
| `params.ttl` | No | integer | Time-to-live in seconds. Messages older than TTL are dropped. Prevents stale command execution. Default: 30s for commands, 0 (no expiry) for queries. |
| `params.payload` | Yes | object | Method-specific data. Schema varies by `method`. |

##### Response Envelope

Standard JSON-RPC 2.0 response:

```json
{
  "jsonrpc": "2.0",
  "id": "01J7K9MXYZ...",
  "result": {
    "status": "accepted",
    "execution_id": "exec-def456",
    "timestamp": "2026-03-08T15:30:00.456Z"
  }
}
```

Error response:

```json
{
  "jsonrpc": "2.0",
  "id": "01J7K9MXYZ...",
  "error": {
    "code": -32001,
    "message": "Session not found",
    "data": {
      "target": "session-build-01",
      "timestamp": "2026-03-08T15:30:00.456Z"
    }
  }
}
```

##### Design Decisions

**Why ULID for `id` instead of UUIDv4?** ULIDs are lexicographically sortable by creation time, making log analysis trivial. They're the same 128 bits as UUID, compatible with UUID columns in databases, and encode to 26 characters (vs UUID's 36).

**Why `target` and `sender` inside `params` rather than top-level?** JSON-RPC 2.0 reserves the top level for `jsonrpc`, `method`, `id`, `params`, `result`, `error`. Adding custom top-level fields breaks spec compliance. Routing metadata lives in `params` alongside the payload.

**Why `ttl` instead of `expires_at`?** TTL is simpler to set (relative) and doesn't require clock synchronization between sessions. The hub computes expiry from `timestamp + ttl`.

**Why no `priority` field?** Priority implies a queue, and queues imply buffering. For v0.1, all messages are processed in arrival order. Priority is a v0.2 concern when task queues are implemented.

#### Data Plane Envelope (Binary Framed)

The data plane uses a compact binary envelope for streaming data. Every frame is self-contained and parseable without context.

##### Frame Layout

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Payload Length                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Version(4) | Type(4) |     Flags     |       Reserved        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Sequence Number                        |
|                        (64-bit)                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Channel ID                             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Payload ...                            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

##### Field Details

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 4 bytes | `payload_length` | Length of payload in bytes (big-endian uint32). Max 16 MiB (0x01000000). Does NOT include header. |
| 4 | 4 bits | `version` | Protocol version (0-15). Current: 1. |
| 4.5 | 4 bits | `type` | Frame type (0-15). See type table below. |
| 5 | 1 byte | `flags` | Bitfield. See flags table below. |
| 6 | 2 bytes | `reserved` | Must be zero. Future use (e.g., compression algorithm, priority). |
| 8 | 8 bytes | `sequence` | Monotonically increasing per channel (big-endian uint64). Enables ordering and gap detection. |
| 16 | 4 bytes | `channel_id` | Identifies the stream/session (big-endian uint32). Enables multiplexing multiple sessions on one connection. |
| 20 | N bytes | `payload` | Raw bytes. Interpretation depends on `type`. |

**Total header size: 20 bytes.** This is a fixed-size header — no variable-length header fields. Parsing requires exactly one 20-byte read followed by one `payload_length`-byte read.

##### Frame Types (4 bits, 0-15)

| Value | Name | Payload Content |
|-------|------|----------------|
| 0x0 | `OUTPUT` | Terminal output bytes (stdout/stderr interleaved, as PTY delivers) |
| 0x1 | `INPUT` | Raw input bytes to inject into target PTY |
| 0x2 | `RESIZE` | Terminal dimensions: 4 bytes (uint16 cols + uint16 rows) |
| 0x3 | `SIGNAL` | 1 byte: signal number (SIGINT=2, SIGTERM=15, etc.) |
| 0x4 | `TRANSFER` | Bulk data (file contents). Chunked via CONTINUATION flag. |
| 0x5 | `PING` | Keepalive. Payload: 8-byte timestamp (sender's monotonic clock). |
| 0x6 | `PONG` | Keepalive response. Payload: echo of PING timestamp. |
| 0x7 | `CLOSE` | Graceful channel close. Payload: 1 byte reason code. |
| 0x8-0xF | Reserved | Future use. |

##### Flags (8 bits)

| Bit | Name | Meaning |
|-----|------|---------|
| 0 | `FIN` | Final frame for this logical message (for chunked transfers). |
| 1 | `COMPRESSED` | Payload is zstd-compressed (negotiate via control plane). |
| 2 | `BINARY` | Payload is binary (not UTF-8 text). Informational hint for logging. |
| 3 | `URGENT` | Out-of-band urgent data (e.g., Ctrl+C injection). Process before queued frames. |
| 4-7 | Reserved | Must be zero. |

##### Design Decisions for Data Plane

**Why 20-byte header instead of the originally proposed 14 bytes?**

The original proposal (`[4:length][1:type][1:flags][8:sequence]` = 14 bytes) omits three things that prove necessary in practice:

1. **Version nibble (4 bits):** Without a version field in binary frames, protocol upgrades require out-of-band negotiation for every connection. A 4-bit version field in the header costs zero extra bytes (shares byte with type) and allows the parser to reject incompatible frames immediately.

2. **Channel ID (4 bytes):** Without multiplexing, each session pair needs a dedicated socket. With `channel_id`, a single data-plane connection can carry multiple streams. This is essential for use case #2 (parallel test dispatch) and #7 (multi-service startup). 4 bytes supports 4 billion concurrent channels.

3. **Reserved bytes (2 bytes):** Future-proofing for compression algorithm ID, priority bits, or other metadata. Without reserved space, any header extension requires a version bump.

**Why big-endian?** Network byte order is the universal convention for wire protocols (TCP, HTTP/2, WebSocket, QUIC). Using big-endian avoids platform-specific byte swapping bugs and matches what every network programmer expects.

**Why 16 MiB max payload?** Large enough for any terminal output buffer or reasonable file chunk. Small enough to prevent memory exhaustion attacks. Aligns with HTTP/2's default max frame size philosophy. Chunking via CONTINUATION handles larger transfers.

**Why no CRC/checksum?** Unix domain sockets provide kernel-guaranteed delivery without corruption. TCP provides checksums. Adding a CRC to every frame wastes CPU for zero benefit on these transports. If TermLink ever runs over UDP (unlikely), add CRC via a version bump.

---

### 3. Framing Analysis

Comparing framing approaches for the **data plane** specifically. The control plane uses JSON-RPC 2.0 (decided in T-003); this analysis is about the binary streaming layer.

| Criterion | Newline JSON | Length-Prefixed Binary | MessagePack | Protobuf | CBOR | Cap'n Proto |
|-----------|:-----------:|:--------------------:|:-----------:|:--------:|:----:|:-----------:|
| **Binary-safe** | No (must base64 encode) | Yes | Yes | Yes | Yes | Yes |
| **Human debuggable** | Excellent | Poor (hex dump) | Poor (binary) | Poor (binary) | Poor (binary) | Poor (binary) |
| **Per-message overhead** | ~50-200 bytes (field names) | 20 bytes (fixed header) | ~10-30 bytes | ~5-15 bytes | ~10-25 bytes | ~8 bytes (header) + padding |
| **Parse complexity** | Low (JSON.parse) | Very low (read header, read N bytes) | Low (library) | Medium (codegen + library) | Low (library) | Low (zero-copy) |
| **Corruption recovery** | Good (scan for newline) | Poor (length corruption = lost sync) | Poor | Poor | Poor | Poor |
| **Streaming support** | Natural (line-by-line) | Natural (frame-by-frame) | No native framing | No native framing | No native framing | Yes (segments) |
| **Zero-copy possible** | No | Yes | No | No (decode required) | No | Yes |
| **Schema evolution** | Flexible (add fields) | Manual (version field) | Flexible | Excellent (field numbers) | Flexible (CDDL) | Excellent |
| **Ecosystem / Go libs** | stdlib | stdlib (3 lines) | msgpack/go | google/protobuf | fxamacker/cbor | capnproto/go |
| **External dependency** | None | None | Runtime lib | Codegen + runtime | Runtime lib | Codegen + runtime |

#### Analysis by Constitutional Directive

**D1 (Antifragility) — Corruption Recovery:**
- Newline JSON wins: a corrupted message loses one line; the next newline resyncs.
- Length-prefixed binary loses: if the 4-byte length is corrupted, the parser reads garbage as the next frame. Mitigation: add a **magic byte** (sync marker) at frame start. After detecting corruption, scan for the next magic byte.
- All binary serialization formats share this weakness.

**D2 (Reliability) — Deterministic Parsing:**
- Length-prefixed binary and Protobuf win: fixed header means exactly two reads (header + payload). No ambiguity.
- Newline JSON has edge cases: embedded newlines in strings, incomplete writes.
- MessagePack/CBOR: deterministic but require library-specific parsers.

**D3 (Usability) — Debuggability:**
- Newline JSON is unmatched for debugging (pipe through `jq`).
- Everything else requires specialized tooling.
- Mitigation for binary: provide a `termlink decode` CLI tool that renders frames as human-readable text.

**D4 (Portability) — Dependencies:**
- Length-prefixed binary: zero external dependencies. Implementable in any language with socket access.
- Protobuf/Cap'n Proto: require codegen toolchains. Lock-in to their ecosystems.
- MessagePack/CBOR: runtime library only, available in all major languages.

#### Recommendation: Length-Prefixed Binary with Magic Sync Marker

**Primary rationale:** Zero dependencies, zero-copy capable, minimal overhead (20 bytes), trivially implementable in any language. Aligns with D4 (portability) and D2 (reliability).

**Corruption recovery mitigation:** Add a 2-byte magic marker (`0xTL` = `0x544C`) before each frame header. On corruption, scan forward for the magic bytes, then validate the header. This costs 2 extra bytes per frame but provides newline-JSON-level recovery.

Updated frame with sync marker:

```
[2 bytes: magic 0x544C][20 bytes: header][N bytes: payload]
```

Total overhead per frame: **22 bytes**. For a typical terminal output chunk of 1-4 KB, this is 0.5-2% overhead.

**Why not MessagePack or CBOR as the frame payload encoding?** They solve a different problem (structured data serialization). Data plane payloads are raw terminal bytes — already in their final form. Wrapping raw bytes in MessagePack adds overhead without value. For the control plane's structured data, JSON-RPC 2.0 handles serialization.

**Why not Protobuf or Cap'n Proto?** Both require codegen toolchains and schema compilation. This violates D4 (portability) — a new language binding requires installing protoc/capnpc. For a 20-byte fixed header, the "schema" is just a struct definition in a comment.

---

### 4. Special Key Encoding

Terminal special keys in injection messages (`command.inject` on control plane, `INPUT` frames on data plane) need consistent encoding.

#### The Problem

Terminal input encoding is messy. The same logical key can have different byte representations depending on terminal emulator, TERM type, and mode (normal vs application). Examples:

| Key | Representation | Bytes |
|-----|---------------|-------|
| Ctrl+C | ASCII control char | `0x03` |
| Ctrl+D | ASCII control char | `0x04` |
| Ctrl+Z | ASCII control char | `0x1A` |
| Up arrow | ANSI escape sequence | `0x1B 0x5B 0x41` (`\e[A`) |
| Down arrow | ANSI escape sequence | `0x1B 0x5B 0x42` (`\e[B`) |
| F1 | Varies by terminal | `\eOP` (vt100) or `\e[11~` (xterm) |
| Alt+x | Varies | `\ex` (meta-sends-escape) or `0xF8` (8-bit) |

#### Design Decision: Raw Bytes as Primary, Symbolic Names as Convenience

For the **data plane** (`INPUT` frames): always raw bytes. The sender is responsible for encoding keys into the correct byte sequence for the target terminal. This is the only approach that is:
- Binary-safe (handles any terminal encoding)
- Zero-overhead (no parsing/translation layer)
- Complete (handles sequences we haven't thought of)

For the **control plane** (`command.inject`): support both raw bytes and symbolic names. The hub translates symbolic names to raw bytes using the target session's TERM capabilities.

##### Control Plane Symbolic Key Format

```json
{
  "method": "command.inject",
  "params": {
    "target": "session-01",
    "sender": "orchestrator",
    "payload": {
      "keys": [
        { "type": "text", "value": "ls -la" },
        { "type": "key", "value": "Enter" },
        { "type": "raw", "value": "AQID" },
        { "type": "key", "value": "Ctrl+C" }
      ]
    }
  }
}
```

##### Symbolic Key Names

| Category | Names | Raw Bytes |
|----------|-------|-----------|
| **Control chars** | `Ctrl+A` through `Ctrl+Z` | `0x01` through `0x1A` |
| **Special** | `Enter`, `Tab`, `Backspace`, `Escape`, `Delete` | `0x0D`, `0x09`, `0x7F`, `0x1B`, `\e[3~` |
| **Arrow keys** | `Up`, `Down`, `Left`, `Right` | `\e[A`, `\e[B`, `\e[D`, `\e[C` |
| **Function keys** | `F1` through `F12` | Per terminfo (e.g., `\eOP` for F1 on vt100) |
| **Modifiers** | `Shift+Tab`, `Alt+x`, `Ctrl+Left` | Per terminfo |
| **Signals** | `Ctrl+C`, `Ctrl+D`, `Ctrl+Z`, `Ctrl+\` | `0x03`, `0x04`, `0x1A`, `0x1C` |

##### Key Encoding Rules

1. **`type: "text"`** — UTF-8 string, sent as-is. For typing commands.
2. **`type: "key"`** — Symbolic name. Hub resolves to bytes via target's terminfo database. Fallback to xterm-256color if TERM unknown.
3. **`type: "raw"`** — Base64-encoded raw bytes. Escape hatch for anything symbolic names don't cover.
4. **Sequence ordering:** Keys array is processed in order. Timing is not guaranteed between elements (the hub sends as fast as the PTY accepts).
5. **Delay insertion:** If timing matters (e.g., waiting for a prompt), use `command.execute` with expected output patterns instead of blind injection with delays.

##### Why Not a tmux-Style Key String?

tmux uses a compact string format: `send-keys C-c Enter "ls" Enter`. This is terse but:
- Ambiguous: Is `C-c` Ctrl+C or the literal characters C, -, c?
- Not self-describing: requires parser knowledge of tmux syntax
- Not extensible: adding new key types means extending the parser grammar

The array-of-objects format is more verbose but unambiguous, self-describing, and trivially extensible.

---

### 5. Versioning Strategy

#### Protocol Version Field

Both planes carry a version indicator:

- **Control plane:** The `method` namespace serves as the version. Initial methods are unversioned (e.g., `command.execute`). Breaking changes introduce namespaced methods (e.g., `v2/command.execute`). This follows the JSON-RPC convention of using method names for versioning.
- **Data plane:** The 4-bit `version` field in the binary header. Current: `1`. Max: `15` (enough for the lifetime of any protocol).

#### Capability Negotiation

On initial connection, sessions exchange capabilities via `session.register`:

```json
{
  "method": "session.register",
  "params": {
    "sender": "session-01",
    "target": "*",
    "payload": {
      "name": "session-01",
      "capabilities": {
        "protocol_version": 1,
        "data_plane": true,
        "compression": ["zstd"],
        "max_frame_size": 16777216,
        "features": ["streaming", "injection", "signals"]
      },
      "terminal": {
        "term": "xterm-256color",
        "cols": 120,
        "rows": 40,
        "shell": "/bin/zsh"
      }
    }
  }
}
```

The hub responds with the negotiated capabilities:

```json
{
  "result": {
    "session_id": "session-01",
    "negotiated": {
      "protocol_version": 1,
      "compression": "zstd",
      "max_frame_size": 16777216
    },
    "hub_version": "0.1.0"
  }
}
```

#### Backward Compatibility Rules

1. **Additive changes are always safe.** New optional fields in `params`, new event types, new flag bits — existing parsers ignore what they don't understand.
2. **Removing or renaming a field is a breaking change.** Requires control-plane method version bump or data-plane version increment.
3. **New message types are not breaking.** Unknown `method` strings receive a JSON-RPC `-32601 Method not found` error. Senders can gracefully degrade.
4. **New frame types are not breaking.** Unknown data-plane `type` values cause the frame to be logged and skipped. The receiver does not crash.
5. **The `reserved` bytes in the data-plane header may be assigned meaning in future versions.** Existing implementations MUST set them to zero and MUST ignore non-zero values (unless they understand the version).

#### Feature Flags

Feature flags are carried in the `capabilities.features` array during registration. They enable gradual rollout of new functionality without version bumps:

| Flag | Meaning |
|------|---------|
| `streaming` | Session can send/receive `data.stream` frames |
| `injection` | Session accepts `command.inject` / `INPUT` frames |
| `signals` | Session accepts `command.signal` / `SIGNAL` frames |
| `compression` | Session supports compressed frames |
| `transfer` | Session supports `data.transfer` / `TRANSFER` frames |
| `broadcast` | Session accepts broadcast messages (`target: "*"`) |

If a sender invokes a capability the target doesn't advertise, the hub returns error `-32003 Capability not supported`.

#### Deprecation Protocol

1. Log a warning for 2 minor versions when a feature is deprecated.
2. Remove in the next major version.
3. Deprecated methods return results with a `"deprecated": true` field in the response `result` object.

---

### 6. Protocol Specification Draft (v0.1)

This section is the implementable spec. Everything above is rationale; everything below is normative.

---

## TermLink Protocol v0.1 Specification

### 1. Overview

The TermLink protocol enables communication between terminal sessions. It consists of two planes:

- **Control Plane:** JSON-RPC 2.0 over Unix domain socket (local) or WebSocket (remote). Handles session lifecycle, command execution, queries, and events.
- **Data Plane:** Length-prefixed binary frames over Unix domain socket (local) or WebSocket (remote). Handles live output streaming, raw input injection, and bulk data transfer.

Both planes may share a transport connection (multiplexed) or use separate connections.

### 2. Control Plane Protocol

#### 2.1 Transport

- **Local:** Unix domain socket at `$XDG_RUNTIME_DIR/termlink/control.sock` (fallback: `/tmp/termlink-$UID/control.sock`)
- **Remote:** WebSocket at `wss://host:port/control`

#### 2.2 Message Format

All control plane messages are JSON-RPC 2.0. Requests have `method`, `id`, and `params`. Responses have `id` and either `result` or `error`. Notifications have `method` and `params` but no `id`.

#### 2.3 Common Parameters

All requests MUST include these fields in `params`:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `target` | string | Yes | Destination session name, or `"*"` for broadcast |
| `sender` | string | Yes | Source session name |
| `timestamp` | string | Yes | ISO-8601 with milliseconds (e.g., `"2026-03-08T15:30:00.123Z"`) |
| `correlation_id` | string | No | Links related request/response pairs across methods |
| `ttl` | integer | No | Seconds until message expires. Default: 30 for commands, 0 for queries |
| `payload` | object | Yes | Method-specific parameters |

#### 2.4 Message Type Catalog

##### 2.4.1 session.register

Register a terminal session with the hub.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "session.register",
  "id": "01JF7K9MXY000001",
  "params": {
    "target": "*",
    "sender": "build-runner-01",
    "timestamp": "2026-03-08T15:30:00.123Z",
    "payload": {
      "name": "build-runner-01",
      "capabilities": {
        "protocol_version": 1,
        "data_plane": true,
        "compression": ["zstd"],
        "max_frame_size": 16777216,
        "features": ["streaming", "injection", "signals"]
      },
      "terminal": {
        "term": "xterm-256color",
        "cols": 120,
        "rows": 40,
        "shell": "/bin/zsh",
        "pid": 12345
      }
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "01JF7K9MXY000001",
  "result": {
    "session_id": "build-runner-01",
    "negotiated": {
      "protocol_version": 1,
      "compression": "zstd",
      "max_frame_size": 16777216
    },
    "hub_version": "0.1.0",
    "timestamp": "2026-03-08T15:30:00.456Z"
  }
}
```

##### 2.4.2 session.deregister

Graceful session departure.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "session.deregister",
  "id": "01JF7K9MXY000002",
  "params": {
    "target": "*",
    "sender": "build-runner-01",
    "timestamp": "2026-03-08T16:00:00.000Z",
    "payload": {
      "reason": "shutdown"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "01JF7K9MXY000002",
  "result": {
    "status": "deregistered",
    "timestamp": "2026-03-08T16:00:00.123Z"
  }
}
```

##### 2.4.3 session.discover

List available sessions.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "session.discover",
  "id": "01JF7K9MXY000003",
  "params": {
    "target": "*",
    "sender": "orchestrator",
    "timestamp": "2026-03-08T15:30:01.000Z",
    "payload": {
      "filter": {
        "features": ["injection"],
        "name_pattern": "build-*"
      }
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "01JF7K9MXY000003",
  "result": {
    "sessions": [
      {
        "name": "build-runner-01",
        "capabilities": {
          "features": ["streaming", "injection", "signals"]
        },
        "terminal": {
          "term": "xterm-256color",
          "cols": 120,
          "rows": 40,
          "shell": "/bin/zsh"
        },
        "connected_since": "2026-03-08T15:30:00.123Z",
        "status": "idle"
      }
    ],
    "timestamp": "2026-03-08T15:30:01.456Z"
  }
}
```

##### 2.4.4 session.heartbeat

Bidirectional liveness probe.

**Request (notification — no `id`):**
```json
{
  "jsonrpc": "2.0",
  "method": "session.heartbeat",
  "params": {
    "target": "*",
    "sender": "build-runner-01",
    "timestamp": "2026-03-08T15:31:00.000Z",
    "payload": {
      "status": "idle",
      "uptime_seconds": 60
    }
  }
}
```

When sent as a request (with `id`), the hub responds with its own status. When sent as a notification, no response is expected.

##### 2.4.5 command.execute

Execute a structured command in the target session.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "command.execute",
  "id": "01JF7K9MXY000005",
  "params": {
    "target": "build-runner-01",
    "sender": "orchestrator",
    "timestamp": "2026-03-08T15:32:00.000Z",
    "correlation_id": "pipeline-run-42",
    "ttl": 30,
    "payload": {
      "command": "npm test -- --reporter=json",
      "cwd": "/app",
      "env": {
        "CI": "true",
        "NODE_ENV": "test"
      },
      "timeout": 120,
      "capture_output": true
    }
  }
}
```

**Response (accepted):**
```json
{
  "jsonrpc": "2.0",
  "id": "01JF7K9MXY000005",
  "result": {
    "status": "accepted",
    "execution_id": "exec-78901",
    "timestamp": "2026-03-08T15:32:00.234Z"
  }
}
```

**Completion notification (sent later):**
```json
{
  "jsonrpc": "2.0",
  "method": "event.state_change",
  "params": {
    "target": "orchestrator",
    "sender": "build-runner-01",
    "timestamp": "2026-03-08T15:33:15.000Z",
    "correlation_id": "pipeline-run-42",
    "payload": {
      "event": "command_completed",
      "execution_id": "exec-78901",
      "exit_code": 0,
      "output_lines": 42
    }
  }
}
```

##### 2.4.6 command.inject

Inject keystrokes into the target session's PTY.

**Request (notification — no ack needed):**
```json
{
  "jsonrpc": "2.0",
  "method": "command.inject",
  "params": {
    "target": "repl-session",
    "sender": "pair-programmer",
    "timestamp": "2026-03-08T15:34:00.000Z",
    "payload": {
      "keys": [
        { "type": "text", "value": "print('hello')" },
        { "type": "key", "value": "Enter" }
      ]
    }
  }
}
```

For raw byte injection:
```json
{
  "jsonrpc": "2.0",
  "method": "command.inject",
  "params": {
    "target": "stuck-process",
    "sender": "remote-helper",
    "timestamp": "2026-03-08T15:35:00.000Z",
    "payload": {
      "keys": [
        { "type": "key", "value": "Ctrl+C" },
        { "type": "raw", "value": "AQID" }
      ]
    }
  }
}
```

##### 2.4.7 command.signal

Send a POSIX signal to the target session's foreground process.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "command.signal",
  "id": "01JF7K9MXY000007",
  "params": {
    "target": "build-runner-01",
    "sender": "orchestrator",
    "timestamp": "2026-03-08T15:36:00.000Z",
    "payload": {
      "signal": "SIGINT",
      "target_pid": "foreground"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "01JF7K9MXY000007",
  "result": {
    "status": "delivered",
    "pid": 12345,
    "signal": "SIGINT",
    "timestamp": "2026-03-08T15:36:00.100Z"
  }
}
```

##### 2.4.8 query.status

Query the status of the target session's process.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "query.status",
  "id": "01JF7K9MXY000008",
  "params": {
    "target": "build-runner-01",
    "sender": "orchestrator",
    "timestamp": "2026-03-08T15:37:00.000Z",
    "payload": {}
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "01JF7K9MXY000008",
  "result": {
    "status": "running",
    "pid": 12345,
    "foreground_pid": 12350,
    "foreground_command": "npm test",
    "cpu_percent": 45.2,
    "memory_mb": 128,
    "uptime_seconds": 3600,
    "timestamp": "2026-03-08T15:37:00.100Z"
  }
}
```

##### 2.4.9 query.output

Request a snapshot of the target session's output.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "query.output",
  "id": "01JF7K9MXY000009",
  "params": {
    "target": "build-runner-01",
    "sender": "orchestrator",
    "timestamp": "2026-03-08T15:38:00.000Z",
    "payload": {
      "lines": 50,
      "from": "end",
      "strip_ansi": false
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "01JF7K9MXY000009",
  "result": {
    "output": "PASS src/test.js\n  ✓ adds numbers (3ms)\n...",
    "total_lines": 1024,
    "returned_lines": 50,
    "cursor_position": { "row": 38, "col": 0 },
    "timestamp": "2026-03-08T15:38:00.100Z"
  }
}
```

##### 2.4.10 query.capabilities

Query what a target session supports.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "query.capabilities",
  "id": "01JF7K9MXY000010",
  "params": {
    "target": "build-runner-01",
    "sender": "orchestrator",
    "timestamp": "2026-03-08T15:39:00.000Z",
    "payload": {}
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "01JF7K9MXY000010",
  "result": {
    "capabilities": {
      "protocol_version": 1,
      "data_plane": true,
      "compression": ["zstd"],
      "max_frame_size": 16777216,
      "features": ["streaming", "injection", "signals"]
    },
    "terminal": {
      "term": "xterm-256color",
      "cols": 120,
      "rows": 40,
      "shell": "/bin/zsh",
      "pid": 12345
    },
    "timestamp": "2026-03-08T15:39:00.100Z"
  }
}
```

##### 2.4.11 event.state_change

Notification of session state change. Sent as JSON-RPC notification (no `id`).

```json
{
  "jsonrpc": "2.0",
  "method": "event.state_change",
  "params": {
    "target": "*",
    "sender": "build-runner-01",
    "timestamp": "2026-03-08T15:40:00.000Z",
    "payload": {
      "event": "exited",
      "exit_code": 0,
      "details": "Process completed normally"
    }
  }
}
```

Event types: `started`, `exited`, `resized`, `command_completed`, `command_failed`, `idle`, `busy`.

##### 2.4.12 event.error

Non-fatal error notification.

```json
{
  "jsonrpc": "2.0",
  "method": "event.error",
  "params": {
    "target": "orchestrator",
    "sender": "build-runner-01",
    "timestamp": "2026-03-08T15:41:00.000Z",
    "correlation_id": "pipeline-run-42",
    "payload": {
      "code": -32001,
      "message": "Output buffer overflow, oldest 1000 lines dropped",
      "severity": "warning"
    }
  }
}
```

#### 2.5 Error Codes

Standard JSON-RPC 2.0 error codes plus TermLink-specific codes:

| Code | Name | Meaning |
|------|------|---------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid request | Not a valid JSON-RPC 2.0 request |
| -32601 | Method not found | Unknown method name |
| -32602 | Invalid params | Required params missing or wrong type |
| -32603 | Internal error | Hub internal error |
| -32001 | Session not found | Target session does not exist |
| -32002 | Session busy | Target cannot accept commands (already executing) |
| -32003 | Capability not supported | Target lacks requested capability |
| -32004 | Message expired | TTL exceeded before delivery |
| -32005 | Injection failed | PTY write failed (process exited, pipe broken) |
| -32006 | Signal failed | Signal delivery failed (permission, no such process) |
| -32007 | Output unavailable | Scrollback buffer empty or not captured |
| -32008 | Rate limited | Too many requests from sender |
| -32009 | Authentication required | Session not authenticated (future use) |
| -32010 | Authorization denied | Session not authorized for this operation (future use) |

### 3. Data Plane Protocol

#### 3.1 Transport

- **Local:** Unix domain socket at `$XDG_RUNTIME_DIR/termlink/data.sock` (fallback: `/tmp/termlink-$UID/data.sock`)
- **Remote:** WebSocket at `wss://host:port/data` (binary frames)

#### 3.2 Frame Format

Every data plane message is a frame with the following layout:

```
Offset  Size     Field            Description
------  ----     -----            -----------
0       2        magic            0x544C ("TL") — sync marker
2       4        payload_length   Big-endian uint32. Payload size in bytes. Max: 16,777,216 (16 MiB).
6       1        ver_type         High nibble: version (0-15). Low nibble: frame type (0-15).
7       1        flags            Bit flags (see below).
8       2        reserved         Must be 0x0000. Ignored by parser.
10      8        sequence         Big-endian uint64. Monotonically increasing per channel_id.
18      4        channel_id       Big-endian uint32. Identifies the multiplexed stream.
22      N        payload          Raw bytes. Length = payload_length.
```

**Total header size: 22 bytes** (including 2-byte magic).

#### 3.3 Frame Types

| Value | Name | Payload |
|-------|------|---------|
| 0x0 | OUTPUT | Terminal output bytes |
| 0x1 | INPUT | Raw bytes to inject into target PTY |
| 0x2 | RESIZE | 4 bytes: uint16 cols + uint16 rows (big-endian) |
| 0x3 | SIGNAL | 1 byte: POSIX signal number |
| 0x4 | TRANSFER | Chunked bulk data. Use FIN flag for last chunk. |
| 0x5 | PING | 8 bytes: sender's monotonic timestamp (nanoseconds, big-endian uint64) |
| 0x6 | PONG | 8 bytes: echoed PING timestamp |
| 0x7 | CLOSE | 1 byte: reason code (0=normal, 1=error, 2=timeout, 3=evicted) |
| 0x8-0xF | — | Reserved. Receivers MUST skip unknown types (log + discard). |

#### 3.4 Flags

| Bit | Name | Meaning |
|-----|------|---------|
| 0 | FIN | Final frame for a multi-frame message (TRANSFER chunks). |
| 1 | COMPRESSED | Payload is zstd-compressed. Must be negotiated via control plane first. |
| 2 | BINARY | Payload contains non-UTF-8 binary data. Hint for logging/debugging tools. |
| 3 | URGENT | Process before queued frames. Used for Ctrl+C injection (interrupt a running stream). |
| 4-7 | — | Reserved. MUST be zero. |

#### 3.5 Chunking Rules

- Payloads exceeding `max_frame_size` (negotiated, default 16 MiB) MUST be split into TRANSFER chunks.
- All chunks except the last have `FIN=0`. The last chunk has `FIN=1`.
- All chunks share the same `channel_id` and have consecutive `sequence` numbers.
- Receivers reassemble by concatenating payloads in sequence order.
- If a gap in sequence numbers is detected, the receiver MUST discard all chunks for that transfer and log an error.

#### 3.6 Corruption Recovery

If a receiver encounters an invalid frame (bad magic, payload_length > max, unknown version):

1. Log the corruption event with byte offset.
2. Scan forward byte-by-byte for the magic marker `0x544C`.
3. Attempt to parse the header at that position.
4. If the header is valid (version known, type known, payload_length <= max), resume framing from that position.
5. If 64 KiB is scanned without finding a valid frame, close the connection and reconnect.

#### 3.7 Flow Control

For v0.1, flow control is simple:
- **Sender:** Write frames as fast as the socket accepts. Rely on kernel TCP/Unix socket backpressure.
- **Receiver:** Read frames as fast as possible. If processing cannot keep up, the kernel buffer fills and backpressure propagates.
- **No application-level flow control in v0.1.** This is a future enhancement (v0.2) if needed, likely using a credit-based scheme similar to HTTP/2 WINDOW_UPDATE.

#### 3.8 Keepalive

- Send PING frames every 30 seconds on idle connections.
- If no PONG is received within 10 seconds, send one more PING.
- If no PONG is received within 10 seconds of the second PING, close the connection with reason code 2 (timeout).
- PONG must echo the exact 8-byte timestamp from PING.

### 4. Size Limits

| Limit | Value | Rationale |
|-------|-------|-----------|
| Max control plane message | 1 MiB | JSON-RPC messages should be small; output snapshots are the largest |
| Max data plane payload | 16 MiB | Single frame payload limit (configurable via negotiation) |
| Max output snapshot lines | 10,000 | Prevent memory exhaustion from large scrollback requests |
| Max injection keys | 1,000 | Prevent runaway injection loops |
| Max sessions per hub | 1,024 | Prevents resource exhaustion. Configurable. |
| Heartbeat interval | 30s | Balance between liveness detection and overhead |
| Default TTL | 30s | Commands expire if not delivered promptly |

### 5. Wire Examples

#### Example: Full Agent Orchestration Flow

```
1. Orchestrator registers:
   -> session.register { name: "orchestrator", features: ["streaming"] }
   <- result { session_id: "orchestrator", negotiated: {...} }

2. Orchestrator discovers build runners:
   -> session.discover { filter: { name_pattern: "build-*" } }
   <- result { sessions: [{ name: "build-runner-01", ... }] }

3. Orchestrator dispatches test:
   -> command.execute { target: "build-runner-01", command: "npm test" }
   <- result { status: "accepted", execution_id: "exec-001" }

4. Build runner streams output (data plane):
   -> [TL][len][OUTPUT][seq=1][ch=1]["Running tests..."]
   -> [TL][len][OUTPUT][seq=2][ch=1]["PASS test1.js"]
   -> [TL][len][OUTPUT][seq=3][ch=1]["PASS test2.js"]

5. Build runner notifies completion:
   -> event.state_change { event: "command_completed", exit_code: 0 }

6. Orchestrator queries final output:
   -> query.output { target: "build-runner-01", lines: 100 }
   <- result { output: "...", total_lines: 42 }
```

#### Example: Live Pair Programming Flow

```
1. Pair programmer subscribes to output (data plane, channel setup via control plane)

2. Target terminal streams output continuously:
   -> [TL][len][OUTPUT][seq=N][ch=1][terminal output bytes...]

3. Pair programmer injects keystrokes (control plane for convenience):
   -> command.inject { keys: [{ type: "text", value: "fix_function()" }, { type: "key", value: "Enter" }] }

4. Or via data plane for low-latency:
   -> [TL][len][INPUT][seq=1][ch=1]["fix_function()\r"]
```

---

## Synthesis

The protocol design achieves three key properties:

1. **Minimal but complete.** 14 message types cover all 12 use cases. No type is unused; no use case is uncovered. The type set is extensible without breaking changes.

2. **Clean plane separation.** Control plane (JSON-RPC 2.0) handles structured request/response. Data plane (binary frames) handles streaming. They can share a transport or run independently. Each plane can evolve separately.

3. **Constitutional alignment.** D1 (antifragility): magic bytes enable corruption recovery; unknown types are skipped not crashed. D2 (reliability): fixed headers, sequence numbers, error codes. D3 (usability): JSON control plane is debuggable with curl/jq; symbolic key names avoid raw byte juggling. D4 (portability): zero external dependencies; implementable in any language with sockets and JSON.

### Key Design Choices Summary

| Choice | Decision | Rationale |
|--------|----------|-----------|
| Control plane format | JSON-RPC 2.0 | MCP compatibility, universal tooling |
| Data plane framing | Length-prefixed with magic sync marker | Zero deps, corruption recovery, minimal overhead |
| ID format | ULID | Sortable, unique, timestamp-embedded |
| Header size | 22 bytes fixed | No variable fields = predictable parsing |
| Key encoding | Symbolic names (control) + raw bytes (data) | Usability for humans, performance for machines |
| Versioning | Method namespacing (control) + header nibble (data) | Non-breaking additions, clean major bumps |
| Max frame payload | 16 MiB negotiable | Large enough for any terminal use, bounded for safety |

## Decision

Pending inception go/no-go. The protocol design is coherent and implementable. All go criteria are met:
- Covers all 12 T-003 use cases with 14 message types (under the 15-type threshold)
- Framing satisfies D1 (magic byte sync recovery) and D2 (fixed header, deterministic parsing)
- Message set is minimal but extensible (additive changes are non-breaking)

**Recommendation: GO** — proceed with implementation tasks for hub, client library, and CLI.

## Dialogue Log

### 2026-03-08 — Investigation started
- **Approach:** Sub-agent research dispatch

### 2026-03-08 — Protocol design research complete
- **Message taxonomy:** 14 types across 5 categories, covering all 12 use cases
- **Envelope:** JSON-RPC 2.0 (control) + 22-byte fixed binary header with magic sync marker (data)
- **Framing decision:** Length-prefixed binary with `0x544C` magic marker for corruption recovery
- **Key encoding:** Dual mode — symbolic names on control plane, raw bytes on data plane
- **Versioning:** Additive-safe with capability negotiation on connect
- **Spec produced:** v0.1 draft with full message catalog, byte-level layouts, error codes, and size limits
