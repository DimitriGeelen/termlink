---
task: T-2285
title: "Substrate ack-with-retry — enforce delivery receipts for the parallel-exec harness (§9 hard-dep #5)"
arc: arc-parallel-substrate
status: exploration-complete (awaiting human GO/NO-GO)
created: 2026-06-25
companion_adr: docs/architecture/parallel-execution-substrate.md
aef_adr: /opt/999-Agentic-Engineering-Framework/docs/architecture/parallel-execution-aef.md
recommendation: GO (lean Design A — client-side retry helper, no hub-side delivery state)
---

# T-2285: Substrate ack-with-retry — inception research

## Inception Question

Does the substrate need a new ack-with-retry primitive (sender detects a dead
recipient and retries), and if so, at what layer? AEF ADR §5 names this as the
one open **hard** substrate dependency for the parallel-execution harness
("TermLink receipts are advisory today"); substrate ADR §6 #5 scopes it as "the
substrate half of the sender-side retry the AEF layer relies on."

## Spike findings (code-grounded, 2026-06-25)

All paths under `crates/`. Investigated the three relevant primitives.

### 1. The current receipt path is advisory telemetry, not a guarantee
- `channel.ack` (`termlink-cli/src/commands/channel.rs:2259`) appends an ordinary
  envelope `msg_type="receipt"` carrying `metadata.up_to=N`. That append is its
  *entire* effect — there is no separate ack table.
- `channel.receipts` (`termlink-hub/src/channel.rs:957`) walks the topic and
  returns the latest receipt per sender (`{sender_id, up_to, ts_unix_ms}`).
- **Nothing in the hub acts on a missing/late receipt** — no redelivery, no
  timer, no retry. `handle_channel_post_with` returns success the instant
  `bus.post` commits the offset (`channel.rs:662`). "Advisory" = recorded as a
  log envelope, never acted upon.

### 2. T-1485 `--ack-required` blocks but does not retry
- `cmd_agent_contact` ack branch (`termlink-cli/src/commands/agent.rs:1097`)
  polls the dm topic at 1s cadence up to `ack_timeout_secs` (default 60, clamped
  `[5,600]`), then fails (exit 10). **Single poll-through, no retry.**
- Its "ack" is a *liveness proxy*: `detect_ack_in_msgs` (`channel.rs:1010`)
  **excludes** `msg_type=="receipt"` and counts any free-text reply from the
  peer after send-time. So it does NOT use the formal receipt frontier.

### 3. T-2051 outbound queue is a hub-connectivity buffer (Gap B)
- `BusClient::post` (`termlink-session/src/bus_client.rs:163`) sends directly;
  only on *transport failure* does it enqueue (`PostOutcome::Queued`). The flush
  loop pops a row the instant the hub answers `Ok` to `channel.post`.
- **No "delivered to recipient" concept** — drains on hub-reachable, regardless
  of whether recipient X ever read/acked. It cannot, as-is, back a
  retry-until-recipient-acks loop.

### 4. T-2049 dedupe supplies the exactly-once leg (works)
- Hub LRU keyed `(sender_id, client_msg_id)` (`termlink-hub/src/dedupe.rs:19`),
  5-min TTL, 10K entries. A duplicate post returns the cached envelope with
  `deduped:true` and does **not** re-append (`channel.rs:639`).
- The offline queue persists `client_msg_id` per row and replays the **same**
  id, so a retry is absorbed exactly-once. **This leg is solved today.**

## The decisive nuance

The receipt **frontier** (`up_to >= message_offset`) IS a usable recipient-ack
signal — *if the recipient emits it*. Today receipts are emitted only when the
recipient voluntarily runs `channel ack`, and the one shipped waiter
(`--ack-required`) ignores them. But in the harness, the recipient is a
deterministic sidecar that already consumes the message — emitting one receipt
after consuming is a one-line convention. That collapses "Gap A" (no
machine-emitted recipient ack) from a substrate problem into an AEF-layer
convention.

## Two candidate designs

### Design A — advisory receipts + client-side retry helper (LEAN / RECOMMENDED)
- **Recipient** (AEF harness sidecar) auto-acks via the existing `channel.ack`
  after consuming a message — a §9-clean AEF-layer responsibility.
- **Sender** uses a new client-side helper: post with a `client_msg_id`, record
  `(dm-topic, offset, retry-at T)` in a small durable tracker (borrow the
  T-2051 SQLite/flush pattern), then poll `channel.receipts` until the
  recipient's `up_to >= offset` or the retry deadline; on deadline, re-post
  reusing the **same** `client_msg_id` (T-2049 absorbs the duplicate).
- **Substrate change: minimal-to-none in the hub.** Reuses receipts + dedupe
  wholesale; the new state is a client-side awaiting-ack tracker. Possibly a CLI
  convenience verb (`post --await-ack --retry`) is all that ships substrate-side.
- **Invariants:** preserved — strict star intact, hub stays delivery-stateless,
  append-log unchanged. Smallest blast radius.

### Design B — enforced hub-side redelivery (HEAVIER)
- Hub tracks per-message delivery state, redelivers until ack or dead-letters.
- Stronger guarantee, but the hub becomes delivery-stateful, blast radius is
  cross-subsystem, and it presses on the §10 append-log/star invariants.

## Recommendation: **GO**, build Design A

The spike shows ack-with-retry does NOT require hub-side enforced delivery. The
exactly-once leg already exists (T-2049); the durability pattern already exists
(T-2051); the ack signal already exists (receipt frontier) and only needs a
recipient that emits it — an AEF-harness convention, not a substrate feature.
The substrate's contribution is a small, reversible, client-side retry helper
(and possibly a CLI verb), preserving every §10 invariant. This is the
producer-side closure of §9 hard-dep #5 with the smallest viable footprint.

GO authorizes: a client-side awaiting-ack/retry tracker + helper, the recipient
auto-ack convention (documented for AEF), and an integration test proving
retry-after-dead-recipient is exactly-once. NO-GO if the human judges advisory
receipts + a pure AEF-side loop need zero substrate change (then this closes as
"AEF-layer concern").

## Open (co-discover with AEF — soft dependency)
- Retry policy numbers (poll cadence, retry deadline, max attempts, dead-letter
  sink) should align with AEF §6 heartbeat tick/threshold (lean 5s/30s). T-2323
  dialogue input.
- Whether the recipient auto-ack lands as an AEF sidecar convention or a
  substrate-provided `--auto-ack` subscribe option (the latter would make it
  reusable beyond the harness).
