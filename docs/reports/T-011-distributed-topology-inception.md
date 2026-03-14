# T-011: Distributed Topology — Inception Report

> Task: T-011 | Date: 2026-03-14

## Problem Statement

TermLink is local-only (Unix sockets). For multi-machine agent coordination
(dev laptop + cloud VMs, container orchestration, CI workers), sessions need
to communicate across network boundaries.

## Current State (Post T-122)

T-122 delivered the transport abstraction layer:
- `TransportAddr` enum: `Unix { path }` / `Tcp { host, port }` (protocol crate)
- `Transport`, `Connection`, `TransportListener`, `LivenessProbe` traits (session crate)
- `UnixTransport` adapter wrapping existing Unix socket code
- `RegistrationAddr` with backward-compatible serde

**A3 is satisfied.** The trait-based transport abstraction is in place.

## Assumption Validation

### A1: SSH tunneling sufficient for 90% of cross-machine use cases ✅
**Validated.** Both TermLink planes are pure streaming protocols:
- **Control plane** (JSON-RPC): newline-delimited JSON over `BufReader + lines()`. No Unix socket semantics (no SCM_CREDENTIALS, no fd passing).
- **Data plane** (binary frames): 22-byte header + payload via `read_exact()`/`write_all()`. Pure bytes, stream-agnostic.

SSH socket forwarding works transparently:
```bash
# Socket-to-socket (OpenSSH 7.6+)
ssh -L /tmp/termlink-local.sock:/run/termlink/sessions/tl-xxx.sock user@remote

# Socket-to-TCP via socat
ssh -L 5555:localhost:5555 user@remote
```

No code changes needed for SSH tunneling — it's an infrastructure-only solution.

### A2: Container networking can use TCP without NAT traversal ✅
**Validated by architecture analysis:**
- Docker bridge networking: containers on same host share a bridge network. TCP works directly between container IPs.
- Kubernetes pod networking: flat network model, all pods can reach all pods by IP. TCP works without NAT.
- Docker Compose: service names resolve via DNS. `tcp://session-worker:9000` works.
- Only cross-cloud/cross-datacenter requires NAT traversal or relay — out of scope for initial distributed support.

### A3: Transport abstraction is prerequisite ✅
**Validated.** T-122 delivered. `Connection` is a blanket trait over `AsyncRead + AsyncWrite + Send + Unpin` — TcpStream auto-implements it. TcpTransport is ~50 lines following UnixTransport pattern.

### A4: Hub federation more complex than hub-spoke ✅
**Validated by analysis:**
- **Hub-spoke** (single hub, remote sessions connect via TCP): Hub is the only component that needs TCP awareness. Sessions register with their TransportAddr (Unix or TCP), hub routes to them. Complexity: medium (refactor client.rs + router.rs).
- **Federation** (multiple hubs peering): Requires hub-to-hub protocol, session ownership/routing tables, conflict resolution for session IDs, gossip or centralized registry. Complexity: high.
- **Recommendation:** Hub-spoke first. Federation is a future iteration if cross-datacenter coordination is needed.

## Topology Options

### Option 1: SSH Tunneling Only (Zero Code Changes)
- Users forward Unix sockets via SSH
- Works today with no code changes
- Limitations: manual setup, no service discovery across machines, each session needs its own tunnel
- **Best for:** Developer laptop ↔ cloud VM (1-2 machines, manual setup acceptable)

### Option 2: Native TCP Transport (TcpTransport Implementation)
- Implement `TcpTransport` adapter (~50 lines)
- Refactor `client.rs` to accept `TransportAddr` instead of `Path`
- Refactor hub `router.rs` to route by addr type
- Add `TcpLivenessProbe` (TCP connect with 500ms timeout)
- **Effort:** ~6 coupling points to refactor (medium, 1 session)
- **Best for:** Container orchestration, CI workers, multi-machine setups

### Option 3: TCP + TLS Transport
- Option 2 + TLS wrapper (rustls or native-tls)
- Certificate management (CA, rotation) adds operational complexity
- **Best for:** Production deployments across untrusted networks
- **Defer to:** After Option 2 proves the TCP path works

### Option 4: Hub Federation
- Multiple hubs peering via hub-to-hub protocol
- Session routing tables, ownership, conflict resolution
- **Defer to:** Future iteration, only if cross-datacenter needed

## Coupling Points for TCP (from codebase analysis)

| Component | Current | TCP Change | Effort |
|-----------|---------|-----------|--------|
| Transport traits | Designed for both | No change | 0 |
| TransportAddr enum | Supports both | No change | 0 |
| Registration serde | Supports both | No change | 0 |
| `client::Client::connect` | Takes `&Path` | Accept `&TransportAddr` | Medium |
| `client::rpc_call` | Takes `&Path` | Accept `&TransportAddr` | Medium |
| `hub/router.rs` forward | Uses `socket_path()` | Use `reg.addr` | Medium |
| `Registration.socket_path()` | Panics on non-Unix | Refactor 3 callers | Low |
| `LivenessProbe` | File existence check | TCP connect attempt | Low |

**Critical path:** `client.rs` refactor — everything flows through `rpc_call()`.

## Spike Results

### Spike 1: SSH Tunnel Forwarding
**Result: Works transparently.** Both control and data planes are stream-agnostic.
No Unix socket semantics used (no credentials, no fd passing, no socket options).
SSH `-L` forwarding is sufficient for ad-hoc cross-machine usage today.

### Spike 2: Docker Container TCP
**Result: Architecture supports it.** TcpTransport implementation + container
networking (Docker bridge / K8s flat network) would enable containerized agents.
Registration already supports `TransportAddr::Tcp` in serde. No protocol changes.

## Decision

**GO** — Phased approach:

1. **Phase 0 (now):** SSH tunneling works today with zero code changes. Document it.
2. **Phase 1 (build task):** Implement `TcpTransport` — ~6 coupling points, 1 session.
   Enables container orchestration and CI worker patterns.
3. **Phase 2 (future):** TLS wrapper for untrusted networks. Defer until needed.
4. **Phase 3 (future):** Hub federation. Only if cross-datacenter coordination emerges as a need.

**Go/No-Go criteria met:**
- SSH tunneling works transparently ✅
- Clear topology model (hub-spoke) with manageable complexity ✅
- Transport abstraction (T-122) is in place ✅

## Build Tasks to Create

- **T-XXX:** Document SSH tunneling setup (Phase 0)
- **T-XXX:** Implement TcpTransport + refactor client.rs (Phase 1)
- **T-XXX:** Add TcpLivenessProbe (Phase 1)
