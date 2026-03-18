# T-163: Cross-Machine Agent Communication Research

> Inception research for bidirectional agent-to-agent communication via TermLink TCP hub.

## Research Questions

1. **TCP hub readiness** — Does the existing TCP hub support real-time cross-machine messaging?
2. **Security model** — What auth/encryption exists? What gaps for LAN exposure?
3. **File transfer** — How to transfer files between machines via TermLink?
4. **Agent protocol** — What message format for agent-to-agent request/response?

## Findings

### Q1: TCP Hub Readiness — MOSTLY READY

**What works (built in T-145/T-146/T-147):**
- TCP listener: `termlink hub start --tcp 0.0.0.0:9100` — dual Unix+TCP select loop
- Remote session registration: `session.register_remote` RPC with auto-generated session IDs
- Heartbeat + TTL: 5-minute TTL, 30s reaper sweep
- Hybrid discovery: `session.discover` returns both local and remote sessions, filterable by tags/roles
- Hub forwarding: `forward_to_target()` resolves local first, then remote store, connects via TCP
- Tests: `hub_dual_listen_unix_and_tcp`, `register_remote_and_discover`, `forward_to_remote_session_via_tcp`

**Gaps:**
- **Events over TCP untested** — broadcast/collect resolve remote targets but no integration test validates event delivery to a remote TCP session end-to-end
- **Data plane not bridged to TCP** — binary frames (attach/stream) only work over local Unix sockets
- **Remote supervision passive** — remote sessions rely on TTL expiry (5min lag), no active liveness probes

**Assessment:** Core infrastructure present. Events over TCP are the critical untested path.

### Q2: Security Model — CRITICAL GAPS FOR TCP

**What's implemented (4-phase model):**
1. UID-based auth via `SO_PEERCRED`/`LOCAL_PEERCRED` — **Unix sockets only, not TCP**
2. Permission scopes: Observe → Interact → Control → Execute, enforced per-method
3. Capability tokens: HMAC-SHA256, session-scoped, expiry, nonce — `auth.token` RPC upgrades scope
4. Command allowlist: prefix-based restriction on `command.execute`

**Critical TCP security gaps:**

| Gap | Severity | Description |
|-----|----------|-------------|
| **No TCP authentication** | CRITICAL | TCP connections bypass all auth, go straight to `handle_connection()`. Code comment: "no auth — LAN-only" |
| **No TLS** | HIGH | Tokens and payloads sent in plaintext over TCP. Sniffable on shared LAN |
| **Token not enforced** | HIGH | Token system is opt-in per session. Legacy sessions (no `token_secret`) grant Execute scope to everyone |
| **No rate limiting** | MEDIUM | Unlimited TCP connections and requests accepted |

**Attack surface:** Unauthorized LAN user can connect to hub TCP port and:
- Discover all sessions (names, PIDs, tags)
- Read terminal output (`query.output`)
- Inject keystrokes (`command.inject`)
- Execute arbitrary shell commands (`command.execute`) — **RCE as session owner**
- Broadcast/collect events across all sessions

**Minimum fix before cross-machine use:**
1. Require `auth.token` on all TCP connections before accepting any RPC
2. Default TCP connections to zero scope (no access until authenticated)
3. Add TLS (rustls) to prevent token sniffing
4. Make `token_secret` mandatory when TCP hub is enabled

### Q3: File Transfer — BASE64 EVENTS RECOMMENDED

**Existing mechanisms assessed:**

| Mechanism | Capacity | Cross-machine? | Suitable? |
|-----------|----------|----------------|-----------|
| Data plane (binary frames) | 16MB/frame, Transfer type 0x4 defined | No — local only, not bridged to TCP | Not yet |
| Event payloads | JSON (`serde_json::Value`), 1024-event ring buffer | Yes — works over TCP hub | Yes, with chunking |
| KV store | In-memory `HashMap<String, Value>` | Via RPC forwarding | Small data only |

**Four approaches evaluated:**

| Approach | Pros | Cons | New code needed |
|----------|------|------|-----------------|
| **A: SCP/rsync + signals** | Proven, no protocol code | Requires SSH keys, 2-step | None |
| **B: Base64 events (chunked)** | Native, works cross-machine today, no external deps | +33% overhead, ring buffer limits, receiver must reassemble | Medium (~5.5h) |
| **C: Shared mount + signals** | Zero TermLink code | Requires NFS/SMB infrastructure | None |
| **D: HTTP server + signals** | Simple, large file support | Extra service, port management | None |

**Recommendation: Approach B (Base64 events)** for most files (<100KB typical), with **Approach A (SCP) as fallback** for large files. Protocol sketch:
- `file.send.init` — metadata (filename, size, sha256, chunk count)
- `file.send.chunk` — base64-encoded data with sequence number
- `file.send.complete` — completion signal
- Receiver polls, collects, decodes, verifies hash

### Q4: Agent Message Protocol — NEEDS DESIGN

No agent-to-agent messaging protocol exists. Events are the transport (typed topic strings, JSON payloads), but there's no schema for:
- Request/response patterns (request_id, status, result)
- Task delegation (agent A asks agent B to do work)
- Status reporting (progress, completion, failure)

The event delegation schemas in `termlink-protocol/src/events.rs` provide a starting point:
- `TaskDelegate`, `TaskAccepted`, `TaskProgress`, `TaskCompleted`, `TaskFailed`
- These are defined but not yet used in production

**Proposed agent protocol (over events):**
```
agent.request  → { request_id, from, to, action, payload }
agent.response → { request_id, from, status, result }
agent.status   → { from, state, context_budget, current_task }
file.send.*    → { file_id, filename, chunk_seq, data, sha256 }
```

## Dialogue Log

### 2026-03-18 — Initial request
- **Human:** "I want the framework agent running on another machine to communicate with you"
- **Clarification:** Two-way messaging, same LAN, near real-time, file transfer needed
- **Human:** "Consider security"

## Assumption Assessment

| Assumption | Status | Evidence |
|------------|--------|----------|
| A1: TCP hub bridges sessions across machines | **CONFIRMED** | Dual listener, remote registration, hub forwarding all tested |
| A2: Events work over TCP for real-time messaging | **PARTIAL** | Events work hub-local; TCP delivery path exists but untested |
| A3: Session discovery works cross-machine | **CONFIRMED** | Hybrid discovery (local+remote) tested |
| A4: Capability tokens can secure TCP | **PARTIAL** | Token system exists but NOT enforced on TCP connections |
| A5: File transfer can be layered on events | **CONFIRMED** | JSON payloads + base64 chunking feasible, Transfer frame type also available |
| A6: Agents can learn message protocol | **CONFIRMED** | CLAUDE.md + skills can teach any protocol; delegation schemas exist in protocol crate |

## Recommendations

### Security First (Non-Negotiable)

Before any cross-machine agent communication:
1. **Enforce token auth on TCP** — zero scope until `auth.token` succeeds
2. **Add TLS** — prevent token sniffing on LAN (rustls, self-signed certs OK)
3. **Mandatory `token_secret`** when TCP hub is enabled

### Build Order (If GO)

| Phase | Task | Effort | Depends on |
|-------|------|--------|------------|
| **1** | TCP auth enforcement (token required on TCP) | 1 session | Nothing |
| **2** | TLS on TCP transport (rustls) | 1 session | Phase 1 |
| **3** | Cross-machine event delivery test (validate A2) | 0.5 session | Phase 1 |
| **4** | Agent message protocol (request/response over events) | 1 session | Phase 3 |
| **5** | File transfer (base64 chunked events) | 1 session | Phase 3 |
| **6** | CLI commands (`termlink agent send`, `termlink agent receive`) | 1 session | Phase 4+5 |

**Total: ~5.5 sessions** (bounded, incremental, each phase independently useful)

### Go/No-Go Assessment

**GO indicators:**
- Core TCP infrastructure works (A1, A3 confirmed)
- Security fix is additive (token system exists, just needs enforcement on TCP)
- File transfer doesn't need protocol changes (events carry JSON payloads)
- Build effort bounded (~5.5 sessions)
- Each phase delivers independent value

**Risk:**
- Events over TCP are untested (A2 partial) — but the code path exists, just needs validation
- TLS adds complexity — but rustls is well-supported in the Rust ecosystem
- Ring buffer (1024 events) may overflow during large file transfers — mitigated by chunking + acknowledgment

**Recommendation: GO** — with security (Phase 1+2) as hard prerequisite before any cross-machine messaging.
