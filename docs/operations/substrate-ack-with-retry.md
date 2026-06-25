# Substrate ack-with-retry (T-2285 Design A / T-2286)

**What it is.** The producer-side closure of §9 hard-dependency #5 of the
parallel-execution harness: a sender posts to a peer and *retries until that
peer actually acknowledges* — surviving a recipient that is briefly dead, slow,
or restarting, without ever double-applying the message.

**The decisive design fact (T-2285 inception).** This needs **no hub-side
delivery state**. The three pieces already existed in the substrate:

| Leg | Already shipped | Where |
|---|---|---|
| Exactly-once | hub-side `(sender_id, client_msg_id)` LRU dedupe | T-2049, `termlink-hub/src/dedupe.rs` |
| Durability | client-side SQLite store pattern | T-2051, `termlink-session/src/offline_queue.rs` |
| Recipient-ack signal | the `channel.receipts` frontier (`up_to >= offset`) | `channel.receipts` RPC |

T-2286 adds only the missing producer glue: a durable awaiting-ack tracker and
a retry loop, surfaced as `channel post --await-ack`. The hub stays
delivery-stateless; the strict-star and append-log invariants are untouched.

---

## The contract: two halves

Ack-with-retry only works if **both** sides hold up their end.

### Sender half — `channel post --await-ack` (this repo, shipped)

```bash
# Post to a peer and block until they ack, retrying on each 30s deadline,
# up to 3 attempts. Exits non-zero if no ack ever arrives.
termlink channel post dm:<you>:<peer> \
    --payload "unit T-1234 is yours" \
    --await-ack --retry
```

Flags (all inert unless `--await-ack` is set — the legacy post path is
byte-for-byte unchanged without it):

| Flag | Default | Meaning |
|---|---|---|
| `--await-ack` | off | After posting, poll the recipient's receipt frontier until `up_to >= offset`. Only valid on a `dm:<you>:<peer>` topic (the helper derives *whose* ack to wait for from the topic name). |
| `--retry` | off | On each deadline, **re-post reusing the same `client_msg_id`** (T-2049 dedupe absorbs it — exactly-once) up to `--max-attempts`. Without `--retry`, posts once and waits a single deadline. |
| `--ack-timeout-secs` | 30 | Per-attempt seconds to wait. Aligns with the AEF §6 heartbeat staleness threshold. |
| `--max-attempts` | 3 | Total attempts including the first. Clamped `>= 1`. |

Outcome:
- **Acked** — exit 0; prints `acked by <peer> (attempts=N)`.
- **Exhausted** — exit non-zero (loud, never silent); the post itself is
  durable and the awaiting-ack row is **retained** for a recovery sweep. JSON
  mode emits an `{"exhausted": {...}}` envelope on stdout before the non-zero exit.

### Recipient half — auto-ack convention (AEF-layer responsibility)

**The substrate does not make the recipient ack.** The recipient must, after
consuming a message, emit:

```bash
termlink channel ack dm:<you>:<peer>          # acks through the latest offset
# or, explicitly:
termlink channel ack dm:<you>:<peer> --up-to <offset>
```

In the parallel-execution harness this is a one-line step in the **sidecar's
message-consume path**: after it hands a message to its agent, it emits
`channel.ack up_to=<offset>`. This is an AEF-layer convention, *not* a substrate
feature — keeping it out of the hub is what lets Design A stay
delivery-stateless. If the recipient never acks, the sender's `--await-ack`
correctly reports non-delivery.

> A recipient that has never acked has **no receipt row** at all (the hub
> returns one row per sender that has acked). That absence — not a row with
> `up_to=0` — is what the sender reads as "still deaf". Offset 0 is a real,
> ackable offset.

---

## Why a retry is exactly-once

The retry loop reuses **one stable `client_msg_id`** across every attempt. The
first post appends the envelope and records `(sender_id, client_msg_id)` in the
hub's dedupe LRU; every re-post of the same id returns the cached offset and
**appends nothing**. So "the recipient was dead, we retried 3×" still lands
exactly one envelope on the topic.

This is proven in two halves, deliberately:
- **Hub-side dedupe** — `termlink-hub` test
  `dedupe_with_client_msg_id_duplicate_returns_cached_offset` (a duplicate post
  returns the same offset, `deduped:true`, one envelope).
- **Sender-side reuse** — `termlink-session` test
  `ack_retry::tests::retry_after_dead_recipient_is_exactly_once` (the recipient
  withholds its ack until after one retry; the loop re-posts the same id and the
  dedupe-honouring fake appends exactly once, then succeeds on attempt 2).

Together they cover the full chain with no live-hub integration harness needed.

---

## Durability & recovery

Each outstanding await is recorded in a SQLite tracker
(`~/.termlink/awaiting_ack.sqlite`, or `$TERMLINK_IDENTITY_DIR/awaiting_ack.sqlite`
under test isolation), mirroring the offline-queue conventions:

- `record` on the first post (idempotent on `client_msg_id`),
- `bump_attempts` on each re-post,
- `confirm` (delete) on ack,
- **retained** on exhaustion — so a client that crashes mid-await, or gives up,
  leaves a durable row a recovery sweep can act on (`AwaitingAckTracker::list`).

Inspect it directly:

```bash
sqlite3 ~/.termlink/awaiting_ack.sqlite 'SELECT dm_topic, msg_offset, attempts FROM awaiting_ack'
```

---

## Tuning

Defaults (5s poll cadence, 30s per-attempt deadline, 3 attempts) align with the
AEF §6 heartbeat numbers: the sidecar heartbeats on a ~5s tick and is judged
stale after ~30s, so polling every 5s with a 30s window matches the cadence at
which a real ack can arrive. Tune per workload via `--ack-timeout-secs` /
`--max-attempts`. The poll cadence itself is fixed at 5s (a local-cheap receipts
read; see the offline-queue recipe for the rationale on not over-polling the hub
— the same T-1991 traffic-class concern applies).

---

## See also

- `docs/architecture/parallel-execution-substrate.md` §6 #5 — the substrate
  contract this closes.
- `docs/operations/substrate-post-idempotency.md` — the T-2049 dedupe leg.
- `docs/operations/substrate-offline-queue-recipe.md` — the T-2051 durability
  pattern this tracker mirrors.
- `docs/operations/substrate-orchestrator-recipe.md` — the end-to-end
  work-stealing pattern that consumes ack-with-retry.
