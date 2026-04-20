---
id: T-1159
name: "T-1155/2 Add ed25519 identity keyring to termlink-session"
description: >
  Self-sovereign agent identity per T-1155 S-4. Generate/store ed25519 keypair per session. Bootstrap command (termlink identity init), show fingerprint (termlink identity show), rotate. TOFU pin on first-contact. Separates identity trust from transport trust — structural fix for T-1051 rotation pain.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [T-1155, bus, identity]
components: []
related_tasks: [T-1155]
created: 2026-04-20T14:12:03Z
last_update: 2026-04-20T14:12:03Z
date_finished: null
---

# T-1159: T-1155/2 Add ed25519 identity keyring to termlink-session

## Context

Self-sovereign agent identity for the T-1155 bus. Addresses the S-4 gap in the bus design: **identity trust must be separable from transport trust**, so hub-secret rotations (T-1051 lineage) stop invalidating signed messages. Uses `ed25519-dalek` — already in the transitive dep tree via jsonrpsee? Verify; otherwise add.

Pairs with T-1158 (bus core, unsigned envelope) by layering signatures **on top of** the Envelope payload — T-1158 stays crypto-agnostic; T-1160 decides when to require signed vs unsigned posts.

## Acceptance Criteria

### Agent
- [ ] Add `ed25519-dalek` (or verify existing transitive presence and take direct dep) + `rand_core` to `crates/termlink-session/Cargo.toml`
- [ ] New module `crates/termlink-session/src/identity.rs` exposes:
  - `Identity` struct wrapping `SigningKey` + `VerifyingKey` + cached fingerprint (sha256 of public key, hex, first 16 chars for display)
  - `Identity::load_or_create(path) -> Identity` — reads `<path>/identity.key` (32-byte raw seed, chmod 600) or generates + writes atomically with `tempfile + rename`
  - `Identity::sign(msg: &[u8]) -> Signature` / `Identity::verify(pk, msg, sig) -> bool`
  - `Identity::public_key_hex() -> String`, `Identity::fingerprint() -> String`
- [ ] CLI commands under `termlink identity`:
  - `termlink identity init [--force]` — bootstrap keypair at `~/.termlink/identity.key`; refuses overwrite without `--force`
  - `termlink identity show` — prints fingerprint + public key hex + path
  - `termlink identity rotate` — requires `--force`; renames old to `identity.key.bak-<ts>`, writes new key
- [ ] Keyring (other agents' public keys): `KnownPeers` struct backed by `~/.termlink/known_peers.toml` — maps `peer_id -> {pubkey_hex, first_seen, last_seen, fingerprint}`. API: `learn(peer_id, pubkey)`, `get(peer_id)`, `verify_from(peer_id, msg, sig) -> bool`
- [ ] TOFU semantics: first observation of a peer's pubkey is accepted + pinned; subsequent mismatches log `IDENTITY_TOFU_VIOLATION` with both fingerprints (mirrors the hub TOFU in `termlink-session::tofu`)
- [ ] Unit tests: keypair roundtrip, file permissions are 0600, sign+verify with fresh key, known_peers TOFU accept-then-reject-on-mismatch, fingerprint stability across serialize/deserialize
- [ ] Zero changes to `termlink-bus` — this crate layers over T-1158's opaque payload; the bus does not know about signatures
- [ ] `cargo build -p termlink-session && cargo test -p termlink-session` passes
- [ ] `cargo clippy -p termlink-session -- -D warnings` passes

### Human
- [ ] [REVIEW] Approve key storage location and format
  **Steps:**
  1. Verify default `~/.termlink/identity.key` matches your operator mental model (or override via env var)
  2. Confirm chmod 600 + atomic write is sufficient (no keyring / HSM integration this round)
  3. Check whether TOFU-on-first-use matches your trust policy for agent-to-agent auth
  **Expected:** Approval or a requirement to add stronger protection (keyring, encrypted at rest, etc.)
  **If not:** Note the required protection level; may require a follow-up hardening task

## Verification

cargo build -p termlink-session
cargo test -p termlink-session identity
cargo clippy -p termlink-session -- -D warnings
grep -q "ed25519" crates/termlink-session/Cargo.toml
test -f crates/termlink-session/src/identity.rs

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-20T14:12:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1159-t-11552-add-ed25519-identity-keyring-to-.md
- **Context:** Initial task creation
