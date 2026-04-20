---
id: T-1159
name: "T-1155/2 Add ed25519 identity keyring to termlink-session"
description: >
  Self-sovereign agent identity per T-1155 S-4. Generate/store ed25519 keypair per session. Bootstrap command (termlink identity init), show fingerprint (termlink identity show), rotate. TOFU pin on first-contact. Separates identity trust from transport trust — structural fix for T-1051 rotation pain.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [T-1155, bus, identity]
components: []
related_tasks: [T-1155]
created: 2026-04-20T14:12:03Z
last_update: 2026-04-20T20:46:26Z
date_finished: 2026-04-20T20:46:26Z
---

# T-1159: T-1155/2 Add ed25519 identity keyring to termlink-session

## Context

Self-sovereign agent identity for the T-1155 bus. Addresses the S-4 gap in the bus design: **identity trust must be separable from transport trust**, so hub-secret rotations (T-1051 lineage) stop invalidating signed messages. Uses `ed25519-dalek` — already in the transitive dep tree via jsonrpsee? Verify; otherwise add.

Pairs with T-1158 (bus core, unsigned envelope) by layering signatures **on top of** the Envelope payload — T-1158 stays crypto-agnostic; T-1160 decides when to require signed vs unsigned posts.

## Acceptance Criteria

### Agent
- [x] Add `ed25519-dalek` + `rand_core` (+ `toml`) to `crates/termlink-session/Cargo.toml`
- [x] New module `crates/termlink-session/src/agent_identity.rs` (distinct from existing `identity.rs` which owns `SessionId`) exposes `Identity`, `load_or_create`, `init(force)`, `sign`/`verify`, `public_key_hex`, `fingerprint`, `verifying_key`
- [x] CLI commands under `termlink identity`:
  - `termlink identity init [--force]` — refuses overwrite without `--force` (structured JSON error)
  - `termlink identity show` — prints fingerprint + public key hex + path (JSON flag)
  - `termlink identity rotate --force` — renames old to `identity.key.bak-<ts>`, writes new key; refuses without `--force`
- [x] Keyring `KnownPeers` in `known_peers.rs` backed by TOML at `<base>/known_peers.toml` with `learn`, `get`, `verify_from`, `peer_ids`
- [x] TOFU semantics: first observation pins; re-observation is idempotent; mismatched pubkey returns `PeersError::TofuViolation { peer_id, pinned, got }`
- [x] Unit tests: keypair roundtrip, file perms 0600, sign+verify, TOFU accept-then-violate, fingerprint stability — 13 new tests (7 identity + 6 known_peers)
- [x] Zero changes to `termlink-bus` — verified; layered cleanly over T-1158's opaque payload
- [x] `cargo build -p termlink-session && cargo test -p termlink-session` passes (293 tests)
- [x] `cargo clippy -p termlink-session -- -D warnings` passes

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

### 2026-04-20T20:38:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-04-20T20:46:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
