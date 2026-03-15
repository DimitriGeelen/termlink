# T-144: TCP Hub Listener + Cross-Machine Session Discovery

> Inception research artifact — created 2026-03-15
> Status: exploring

## Agent Research Findings (2 parallel deep-dives)

### Q1: How should the hub add TCP listening?

**Recommendation: Option B — Opt-in via `--tcp` flag**

```bash
# Default: Unix-only (backward compat)
termlink hub start

# Opt-in TCP: explicit port binding
termlink hub start --tcp 0.0.0.0:9100
```

**Implementation (~58 lines):**

| Area | Change | Lines |
|------|--------|-------|
| CLI (`main.rs`) | Add `--tcp` to `HubAction::Start` | ~8 |
| `server.rs` | Dual-listen via `tokio::select!` over Unix + TCP | ~50 |
| Router | No changes (already transport-agnostic) | 0 |
| Tests | New `hub_dual_listen_tcp_unix` test | ~30 |

**Code shape (server.rs):**
```rust
pub async fn run(socket_path: &Path, tcp_addr: Option<&str>) -> io::Result<ShutdownHandle> {
    let unix_listener = UnixListener::bind(socket_path)?;
    let tcp_listener = match tcp_addr {
        Some(addr) => Some(TcpListener::bind(addr).await?),
        None => None,
    };
    // spawn run_accept_loop(unix_listener, tcp_listener, shutdown_rx)
}
```

**Select loop:**
```rust
tokio::select! {
    Ok((stream, _)) = unix_listener.accept() => { /* UID check, forward */ }
    Ok((stream, _)) = async { tcp_listener.as_ref().unwrap().accept().await },
        if tcp_listener.is_some() => {
        // No UID on TCP — deny-all until token auth added
    }
    _ = shutdown_rx.changed() => { /* drain */ }
}
```

**Security:** TCP has no peer credentials (unlike Unix UID). Start with
deny-all on TCP, then add token auth (T-079 capability tokens).

**Why not the others:**
- Option A (always-on): breaks backward compat, security exposure
- Option C (config file): adds TOML parsing dependency for a single boolean

### Q2: How should remote sessions register and be discovered?

**Recommendation: Option A — Hub-mediated registration**

**Flow:**
```
Remote session → TCP connect to hub → session.register_remote RPC
                                    → hub stores in memory (with TTL)
                                    → session heartbeats every 30s
                                    → hub auto-expires after 5 min

Local discovery → session.discover → returns local FS + in-memory remote
```

**Implementation (~500 lines):**

| Area | Change | Lines |
|------|--------|-------|
| `router.rs` | New `session.register_remote` + `session.heartbeat_remote` RPC | ~150 |
| `manager.rs` | `list_sessions_hybrid()` combining FS + remote map | ~50 |
| `liveness.rs` | `is_alive_remote()` via TCP ping probe | ~30 |
| CLI (`main.rs`) | New `register-remote --host --port` command | ~80 |
| In-memory store | TTL-based remote session map in hub | ~100 |
| Tests | Remote registration, heartbeat, expiry, discovery | ~100 |

**New RPC methods (hub-local):**
- `session.register_remote` — store remote session with TTL
- `session.heartbeat_remote` — refresh TTL
- `session.deregister_remote` — explicit removal

**Key insight:** `TransportAddr::Tcp` already serializes correctly. The
`Registration` struct supports TCP addresses TODAY — just no code path
creates them. The protocol doesn't need changes.

**Liveness for remote sessions:**
```rust
pub async fn is_alive_remote(addr: &TransportAddr) -> bool {
    client::rpc_call_addr(addr, "termlink.ping", json!({}))
        .timeout(Duration::from_secs(1))
        .await
        .is_ok()
}
```

**Why not the others:**
- Option B (shared FS): requires NFS mount, operational complexity
- Option C (federation): ~1500 lines, distributed consensus, overkill for now

### Q3: Auth strategy?

**Decision: LAN-only first, defer auth to Phase 2.**
- TCP starts with deny-all (returns "auth required" error)
- Phase 2: capability tokens from T-079
- Phase 3: optional TLS

## Build Task Decomposition

If GO, the work splits into 3 bounded build tasks:

| Task | Scope | Estimate | Depends on |
|------|-------|----------|------------|
| **T-145** | Hub TCP listener (`--tcp` flag + dual select!) | 1 session | Nothing |
| **T-146** | Remote session registration (register_remote RPC + heartbeat + TTL) | 1-2 sessions | T-145 |
| **T-147** | Cross-machine discovery (hybrid list + remote liveness) | 1 session | T-146 |

Total: ~3-4 sessions. Bounded.

## Go/No-Go Assessment

| Criterion | Status |
|-----------|--------|
| Hub can dual-listen without major refactoring (A1) | **YES** — `tokio::select!`, ~50 lines |
| Registration format supports TCP (A2) | **YES** — `TransportAddr::Tcp` already serializable |
| Router transport-agnostic (A3) | **YES** — confirmed, zero changes |
| TCP liveness sufficient for LAN (A4) | **YES** — 500ms connect timeout |
| Auth deferrable (A5) | **YES** — deny-all default, token auth Phase 2 |
| Effort bounded (≤3 sessions) | **YES** — 3 tasks, each ~1 session |

**All GO criteria met.**
