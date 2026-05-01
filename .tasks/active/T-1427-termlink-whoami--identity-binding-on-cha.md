---
id: T-1427
name: "termlink whoami + identity binding on channel post"
description: >
  Pick #4 from T-1425 RFC. Independent of inception outcome. ~50 lines: (a) new 'termlink whoami' subcommand returning canonical sender_id from identity.key + optional human label registered locally; (b) hub-side validation on channel.post that rejects metadata.from=<x> if it doesn't resolve through the connection's authenticated identity. Strict mode is option A in T-1425 Q4 — if Q4 lands on B/C, this task tightens or loosens accordingly. Backward compat: posts without metadata.from continue working unchanged.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T21:18:04Z
last_update: 2026-05-01T19:41:44Z
date_finished: null
---

# T-1427: termlink whoami + identity binding on channel post

## Context

T-1425 RFC §3.2 invariant 2 — "identity authoritative via whoami". Today,
`channel.post` accepts a client-claimed `sender_id` RPC param with no
validation that it matches the cryptographic identity of the signing
key. Concretely: a client could legally sign with their own key but pass
`sender_id="alice"`, and the hub would store the envelope under that
sender_id. The signature verifies (it's over canonical bytes that
DON'T include sender_id — see channel.rs:419-433), but the resulting
envelope is misattributed.

T-1436 plumbed `identity_fingerprint` into SessionMetadata. T-1440
surfaces it in `whoami`. T-1427 closes the loop hub-side: reject any
post where `sender_id != fingerprint_of(sender_pubkey_hex)`.

`fingerprint_of(VerifyingKey) -> String` already exists in
`agent_identity.rs:155` (first 16 hex chars of sha256(pubkey)). The
CLI default already sets `sender_id = identity.fingerprint()` (channel.rs:263),
so legitimate posts continue to work unchanged. Only impostor-style
forgery breaks.

Backward compat: change is hub-side only. Clients send sender_id that
matches their pubkey-derived fp by convention; the change just
enforces the convention. The `--sender-id` override flag continues to
work as long as the override matches the fp (i.e. the flag is now
documentation, not impersonation).

## Acceptance Criteria

### Agent
- [x] New error code `CHANNEL_IDENTITY_MISMATCH = -32014` added in `crates/termlink-protocol/src/control.rs` adjacent to `CHANNEL_TOPIC_UNKNOWN`, with doc comment crediting T-1427
- [x] `handle_channel_post_with` in `crates/termlink-hub/src/channel.rs` derives `expected_fp = fingerprint_of(&verifying_key)` after parsing the pubkey, and rejects with `CHANNEL_IDENTITY_MISMATCH` if `sender_id != expected_fp`. Error message includes both the claimed sender_id and the expected fp prefix (first 8 chars) so operators can debug.
- [x] Unit test: `handle_channel_post_with_rejects_mismatched_sender_id` — posts with sender_id="imposter" but a real key, expects -32014 + readable error message + T-1427 trace
- [x] Unit test: `handle_channel_post_with_accepts_matching_sender_id` — posts with sender_id = fingerprint_of(pubkey), expects success (offset returned)
- [x] Both new tests pass: `cargo test -p termlink-hub --lib handle_channel_post_with_` — 2 passed, 0 failed; full suite 296 passed
- [x] Workspace builds clean: `cargo build --workspace --release` finished in 5m35s with no errors
- [x] Live smoke on isolated test hub (port 9199, runtime_dir /tmp/t1427-smoke-rt): legitimate post (sender_id defaulting to identity fp d1993c2c…) → offset=0; forged post (--sender-id imposter) → rejected with `code=-32014 message=sender_id="imposter" does not match identity fingerprint d1993c2c… derived from sender_pubkey_hex (T-1427)`. Side hub torn down + hubs.toml restored.

### Human
- [x] [RUBBER-STAMP] T-1427 strict-reject lands in agent-chat-arc topic description (T-1430 update) — updated 2026-05-01T19:39Z, offset=40. New description reads "Strict identity-reject enforced (T-1427): sender_id must match fingerprint_of(sender_pubkey_hex)." `grep "lands in T-1427"` returns 0.
  **Steps:**
  1. After ship: `termlink channel info agent-chat-arc | grep -i "lands in T-1427\|strict"`
  2. If still mentions "lands in T-1427" — re-run `termlink channel describe agent-chat-arc "<updated text>"` removing the placeholder
  **Expected:** chat-arc description no longer flags strict-reject as future-work
  **If not:** edit description text and re-run channel describe

## Verification

cargo test -p termlink-hub --lib handle_channel_post_with_ 2>&1 | tail -5 | grep -q "test result: ok"
cargo build --workspace --release 2>&1 | tail -3 | grep -qv "error"
grep -q "CHANNEL_IDENTITY_MISMATCH" crates/termlink-protocol/src/control.rs

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

### 2026-04-30T21:18:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1427-termlink-whoami--identity-binding-on-cha.md
- **Context:** Initial task creation

### 2026-05-01T19:26:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
