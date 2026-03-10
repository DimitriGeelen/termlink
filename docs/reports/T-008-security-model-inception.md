# T-008: Security Model — Capability-Based Access (Inception)

## Problem Statement

Any local process can connect to any TermLink session's Unix socket and execute arbitrary RPC methods. The only trust boundary is socket directory permissions (0o700). Two critical gaps exist:
- **G-001**: Command injection in spawn (T-064 added input validation but no identity-based auth)
- **G-002**: No authentication on Unix sockets — `CommonParams.sender` is self-reported and never validated

## Current State Analysis

### Existing Security Infrastructure (unused)
- `AUTH_REQUIRED` (-32009) and `AUTH_DENIED` (-32010) error codes defined in `control.rs:40-41` — **never used**
- `CommonParams.sender` field in protocol — **defined but never validated**
- Session `uid` captured at registration — **session's own UID, not client's**

### RPC Method Inventory (17 methods)

| Method | Type | Risk Level | Notes |
|--------|------|-----------|-------|
| `termlink.ping` | read | low | Health check |
| `query.status` | read | low | Session metadata |
| `query.capabilities` | read | low | Capability list |
| `query.output` | read | medium | Exposes terminal output |
| `session.heartbeat` | read | low | Keepalive |
| `session.update` | **write** | medium | Mutates tags/name/roles |
| `command.execute` | read* | **critical** | Runs shell commands via `sh -c` |
| `command.inject` | read* | **high** | Injects keystrokes into PTY |
| `command.signal` | read* | **high** | Sends signals to processes |
| `command.resize` | read* | low | PTY resize |
| `event.emit` | read* | medium | Injects events into bus |
| `event.poll` | read | low | Reads events |
| `event.topics` | read | low | Lists topics |
| `kv.set` | **write** | medium | Mutates key-value store |
| `kv.get` | read | low | Reads KV |
| `kv.list` | read | low | Lists KV keys |
| `kv.delete` | **write** | medium | Removes KV entries |

*Note: `command.execute`, `command.inject`, `command.signal`, `event.emit` are classified as "read" in the dispatch lock model (they don't mutate SessionContext) but have **high security impact** — they affect the external world.*

### Permission Tiers (proposed)

| Tier | Methods | Description |
|------|---------|-------------|
| **observe** | ping, query.*, event.poll, event.topics, kv.get, kv.list | Read-only, no side effects |
| **interact** | event.emit, command.resize, session.update, kv.set, kv.delete | Mutates session state |
| **control** | command.inject, command.signal | Affects running processes |
| **execute** | command.execute | Runs arbitrary shell commands |

## Spike 1: Cross-Platform Socket Credentials

### Research Findings

**Linux — SO_PEERCRED:**
- `getsockopt(fd, SOL_SOCKET, SO_PEERCRED, &cred, &len)` returns `ucred { pid, uid, gid }`
- Available on all modern Linux kernels (2.2+)
- Works on connected Unix domain sockets
- Rust: available via `libc::ucred` + raw `getsockopt`

**macOS — LOCAL_PEERCRED:**
- `getsockopt(fd, SOL_LOCAL, LOCAL_PEERCRED, &cred, &len)` returns `xucred { cr_version, cr_uid, cr_ngroups, cr_groups }`
- **Does NOT return PID** — only UID/GID. PID requires `LOCAL_PEERPID` (separate call)
- Available macOS 10.4+
- Rust: `libc::xucred` + raw `getsockopt`, plus `libc::LOCAL_PEERPID` for PID

**Cross-Platform Abstraction:**
```rust
pub struct PeerCredentials {
    pub uid: u32,
    pub gid: u32,
    pub pid: Option<u32>,  // None on macOS without LOCAL_PEERPID
}

impl PeerCredentials {
    pub fn from_stream(stream: &UnixStream) -> io::Result<Self> {
        #[cfg(target_os = "linux")]
        { /* SO_PEERCRED → ucred */ }

        #[cfg(target_os = "macos")]
        { /* LOCAL_PEERCRED → xucred + LOCAL_PEERPID */ }
    }
}
```

**Tokio Compatibility:** `tokio::net::UnixStream` wraps `std::os::unix::net::UnixStream`. Can access raw fd via `AsRawFd` for `getsockopt` calls.

**Assessment:** A1 validated — SO_PEERCRED/LOCAL_PEERCRED work reliably on both platforms. PID available on both (macOS requires extra syscall).

## Spike 2: Capability Token Design

### Option A: Socket-Credential Auth (Recommended for Phase 1)

**How it works:**
1. On `accept()`, extract peer UID/PID via SO_PEERCRED/LOCAL_PEERCRED
2. Compare peer UID to session owner UID (stored in registration)
3. Same UID → full access (current behavior preserved)
4. Different UID → reject with AUTH_DENIED (or apply restricted permissions)

**Pros:** Zero configuration for single-user, no tokens to manage, immediate security improvement
**Cons:** Only UID-granular (can't distinguish agents by the same user)

### Option B: Capability Tokens (Phase 2, for multi-agent)

**How it works:**
1. Session owner generates scoped tokens: `termlink token grant --session S --scope observe,interact --ttl 1h`
2. Token is a signed JWT or HMAC-signed blob containing: session_id, scopes[], expiry, issuer_uid
3. Clients present token in RPC request (new `auth_token` field in CommonParams)
4. Session validates token signature + scope on each request

**Token Format:**
```json
{
  "session_id": "tl-abc123",
  "scopes": ["observe", "interact"],
  "expires_at": 1773080000,
  "issuer_uid": 501,
  "nonce": "random-bytes"
}
```

**Signing:** HMAC-SHA256 with per-session secret (generated at registration, stored in memory). No PKI needed for local-only deployment.

**Pros:** Fine-grained per-agent permissions, supports delegation patterns
**Cons:** Token management complexity, needs revocation strategy

### Option C: Hybrid (Recommended overall)

Phase 1: Socket-credential auth (UID check) — blocks G-002 immediately
Phase 2: Capability tokens layered on top — enables fine-grained multi-agent auth

### Assessment

A2 validated — capability tokens are more flexible than ACLs for delegation.
A3 validated — socket credentials preserve single-user UX with zero config change.

## Spike 3: Implementation Integration Points

### Where Auth Checks Should Go

**Connection level** (in `server.rs` accept loop):
```rust
let (stream, _addr) = listener.accept().await?;
let creds = PeerCredentials::from_stream(&stream)?;
// Reject if UID != session_owner_uid (Phase 1)
```

**Method level** (in `handler.rs` dispatch):
```rust
fn dispatch(&self, method: &str, params: Value, creds: &PeerCredentials) -> Response {
    let required_scope = method_scope(method);  // observe/interact/control/execute
    if !creds.has_scope(required_scope) {
        return error_response(AUTH_DENIED, "Insufficient permissions");
    }
    // ... existing dispatch
}
```

### Files to Modify (Phase 1)

| File | Change |
|------|--------|
| `termlink-session/src/auth.rs` | NEW — PeerCredentials, scope checking |
| `termlink-session/src/server.rs` | Extract creds on accept, pass to handler |
| `termlink-session/src/handler.rs` | Add creds param to dispatch/dispatch_mut |
| `termlink-protocol/src/control.rs` | Already has AUTH error codes (reuse) |
| `termlink-session/src/registration.rs` | Already stores owner UID (reuse) |

### Backward Compatibility

- **Single-user (same UID):** No behavior change — all methods allowed
- **Different UID without token:** Blocked at connection level (AUTH_DENIED)
- **E2e tests:** Continue to work (same user, same UID)
- **`--dangerously-skip-permissions`:** Not affected (that's Claude Code's flag, not TermLink's)

## Latency Assessment

SO_PEERCRED/LOCAL_PEERCRED is a single `getsockopt` syscall — **<0.1ms overhead per connection** (not per RPC call, since credentials are extracted once at connection time). Well within the <5ms budget from the no-go criteria.

## Go/No-Go Analysis

| Criterion | Result |
|-----------|--------|
| SO_PEERCRED/LOCAL_PEERCRED reliable cross-platform | **YES** — both Linux and macOS supported |
| Doesn't break single-user workflow | **YES** — same-UID check is transparent |
| Permission mapping clear and enforceable | **YES** — 4 tiers, 17 methods mapped |
| Latency < 5ms | **YES** — <0.1ms (single syscall) |
| Complexity proportional to threat model | **YES** — Phase 1 is minimal (one new module + 3 file changes) |

**Recommendation: GO**

## Proposed Build Tasks (if GO)

1. **T-NEW-1: Socket-credential auth (Phase 1)** — Add `auth.rs` module, extract PeerCredentials on accept, UID check, use AUTH_DENIED error code. ~2h build.
2. **T-NEW-2: Per-method permission scoping** — Map methods to tiers, enforce in dispatch. ~1h build.
3. **T-NEW-3: Capability token system (Phase 2)** — Token generation, HMAC signing, token validation in dispatch. ~3h build. Depends on T-NEW-1.

## Dialogue Log

- **Q (reflection fleet):** "Any process can connect and impersonate" → confirmed, CommonParams.sender never validated
- **Q (research):** "Can we get peer credentials cross-platform?" → Yes, SO_PEERCRED (Linux) + LOCAL_PEERCRED+LOCAL_PEERPID (macOS)
- **Q (design):** "Token-based vs credential-based?" → Hybrid: credentials for Phase 1 (zero-config), tokens for Phase 2 (fine-grained)
- **Decision:** 4-tier permission model (observe/interact/control/execute) maps cleanly to the 17 RPC methods
