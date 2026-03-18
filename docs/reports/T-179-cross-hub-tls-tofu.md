# T-179: Cross-Hub TLS Trust via TOFU — Research Artifact

## Created: 2026-03-18

## Problem

Hub-to-hub forwarding fails when Hub A needs to connect to a session on Hub B's machine. The TLS handshake fails because `Client::connect_addr()` (client.rs:34) looks for `hub.cert.pem` in the LOCAL runtime dir — it finds Hub A's own cert, which Hub B doesn't accept (different self-signed cert).

**Error observed:** "JSON error: expected value at line 1 column 1" — this is TLS handshake failure surfacing as a parse error (TLS alert bytes aren't valid JSON).

## Current Architecture

```
Hub A (macOS)                    Hub B (Linux)
  hub.cert.pem (A's cert)         hub.cert.pem (B's cert)
  hub.key.pem  (A's key)          hub.key.pem  (B's key)

Router sees remote session → Client::connect_addr(tcp://hubB:9100)
  → reads LOCAL hub.cert.pem (A's cert) for root store
  → TLS handshake against B's cert → FAILS (A's cert ≠ B's cert)
```

## Solution: TOFU (Trust On First Use)

SSH model: accept and store the remote cert fingerprint on first connect, verify on subsequent connections.

### Variant A: Custom ServerCertVerifier (Recommended)

Implement `rustls::client::danger::ServerCertVerifier` that:
1. On first connect: accept any cert, compute SHA-256 fingerprint, store in `~/.termlink/known_hubs`
2. On subsequent connects: compare fingerprint against stored value
3. If mismatch: reject connection (MITM detection, like SSH's "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!")

**File format** (`~/.termlink/known_hubs`):
```
# host:port fingerprint first_seen last_seen
192.168.10.107:9100 sha256:abc123... 2026-03-18T22:00:00Z 2026-03-18T23:00:00Z
```

**Changes needed:**
- `crates/termlink-session/src/client.rs` — new `build_tofu_connector()` replacing `build_tls_connector()` for TCP
- `crates/termlink-session/src/tofu.rs` (new) — `TofuVerifier` impl + `known_hubs` file management
- `crates/termlink-cli/src/main.rs` — `termlink hub fingerprint` and `termlink hub trust <host:port> <fp>` commands

**Estimated:** ~200-250 lines of Rust

### Variant B: Shared CA

Pre-share a CA cert between hubs. Both generate certs signed by the CA.

**Rejected:** Requires manual CA setup, key distribution, cert rotation. Overkill for LAN 2-node case.

### Variant C: Plain TCP (No TLS for Hub-to-Hub)

Skip TLS for inter-hub traffic. Use auth tokens only.

**Rejected:** Tokens travel in plaintext on the wire. Violates D2 (reliability/security).

### Variant D: Cert Exchange via Hub Discovery

Hubs exchange certs during `hub.discover` RPC. Each hub stores remote certs.

**Rejected:** Requires hub-to-hub discovery protocol (doesn't exist yet). TOFU is simpler.

## Technical Constraints

- `rustls::client::danger::ServerCertVerifier` is the extension point (requires `dangerous()` builder)
- `known_hubs` file must be outside runtime dir (runtime dir is ephemeral, cleared on reboot)
- Home dir `~/.termlink/` is appropriate (persistent, per-user)
- The TOFU verifier should be opt-in for `connect_addr` TCP path only (Unix sockets don't use TLS)

## Go/No-Go Assessment

**GO criteria met:**
- Clear implementation path (~250 lines)
- rustls provides the extension point (`ServerCertVerifier`)
- SSH has proven the UX model (known_hosts)
- Unblocks cross-machine communication (T-163)

**Risks:**
- `dangerous()` API is explicitly unsafe-by-design in rustls — requires careful implementation
- Fingerprint storage location (~/.termlink/) needs creation on first use
- No cert rotation story yet (future concern, not blocking)

## Recommendation

**GO** — implement TOFU verifier. The implementation is bounded, the API exists, and this unblocks the core cross-machine use case.
