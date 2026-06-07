# T-2019: Exclusive-delivery / claim semantics — inception research

**Status:** Inception (started-work, owner: human, focus). Research in progress.
**Arc:** `arc-parallel-substrate` (arc-001).
**ADR:** [`docs/architecture/parallel-execution-substrate.md`](../architecture/parallel-execution-substrate.md) §6.1.
**Predecessor research:** T-2007 (substrate-reality-as-it-is).
**Authoring rule:** Per C-001, this file is updated incrementally during the inception; it IS the thinking trail. Conversations are ephemeral, this file is permanent.

---

## 1. Problem (verbatim from §6.1)

> Topics are broadcast: two subscribers to a "work" topic both see every post, so the naive design (post tasks to a queue, idle agents grab the next) would have two agents grab the same task. There is no lock/lease/CAS/claim verb. Needed for any safe handoff of a unit of work to exactly one consumer.

The orchestrator cannot be a passive pull-queue **until and unless** a claim primitive exists. Until then, exclusive assignment must be the orchestrator's explicit act.

## 2. IW questions to resolve (from task file)

- **IW-1:** Semantics — hold-claim-until-ack vs lease-with-renewal vs CAS-on-offset?
- **IW-2:** Storage — in-memory (lost on hub restart) vs persisted to SQLite (durable)?
- **IW-3:** Interaction with persistent cursors — does claim advance the cursor, or is it orthogonal?
- **IW-4:** New RPC verb or a flag on `subscribe`?

The Recommendation block must dispose all four with confidence + rationale before GO is justified.

## 3. Substrate map (read-side, what exists today)

### 3.1 Channel topic storage + append-log

- **Post handler:** `crates/termlink-hub/src/channel.rs:362` — `handle_channel_post_with(bus, id, params)`. Verifies ed25519 signature (lines 430-444), enforces sender_id ↔ pubkey fingerprint match per T-1427 (lines 446-461), calls `bus.post(&topic, &env)` at line 485.
- **Records table** (`crates/termlink-bus/src/meta.rs:291-298`): `(topic, offset, byte_pos, length, ts_unix_ms)`, PK=(topic, offset).
- **Offset assignment:** monotonic per-topic via `offsets` table at `meta.rs:287-290`; each `record_append()` (97-122) bumps `next_offset` transactionally.
- **On-disk log:** `<bus-root>/topics/<sha256(topic)>.log` (8-byte BE length prefix + JSON envelope).
- **Envelope** (`crates/termlink-bus/src/envelope.rs:19-29`): `topic, sender_id, msg_type, payload, artifact_ref, ts_unix_ms, metadata`. **No consumer/claim/lock-related columns anywhere.**

### 3.2 Subscription mechanism + cursor management

- **Subscribe handler:** `crates/termlink-hub/src/channel.rs:535` — `handle_channel_subscribe_with(bus, id, params)`. Params: topic + cursor + limit + timeout_ms + conversation_id + in_reply_to. Long-poll uses `tokio::sync::Notify` per topic.
- **Cursors table** (`meta.rs:281-286`): `(subscriber_id, topic, last_offset)`, PK=(subscriber_id, topic). **Per-subscriber state is fully independent.**
- **Cursor mutations:** `put_cursor()` / `get_cursor()` at `meta.rs:196-225`. No ack mechanism beyond advancing the cursor.
- **No exclusive-delivery semantics:** filtering (conversation_id, in_reply_to) is read-side only. No server-side "mark as delivered to this subscriber" or "lock this offset" mechanism.

### 3.3 RPC dispatch surface (where a new verb slots in)

- **Dispatcher:** `crates/termlink-hub/src/router.rs:67-116` — exhaustive match on `req.method.as_str()`. Existing arms: CHANNEL_CREATE, CHANNEL_POST, CHANNEL_SUBSCRIBE, CHANNEL_LIST, CHANNEL_TRIM, CHANNEL_RECEIPTS, DIALOG_PRESENCE.
- **Method constants:** `crates/termlink-protocol/src/control.rs:102-144`. All Tier-A (opaque payload, drift-tolerant per T-1133).
- **Adding a new verb** = exactly 3 surfaces:
  1. Add constant in `termlink-protocol/src/control.rs::method`
  2. Implement `async fn handle_channel_xxx(id, params) -> RpcResponse` in `termlink-hub/src/channel.rs`
  3. Add match arm in `router.rs:67-116`
- **Response pattern:** `Response::success(id, json!({...}))` or `ErrorResponse::new(id, code, msg)`.
- **Per-verb auth scopes do NOT exist today** — T-1159 on-hold. **Out of scope for T-2019**; flagged as cross-cutting dependency.

### 3.4 Existing claim/lock/lease/CAS patterns

**Zero matches in application code.** Search covered `claim`, `lease`, `lock`, `cas`, `compare_and_swap`, `acquire`, `exclusive`, `reserve`, `SELECT ... FOR UPDATE`, `BEGIN IMMEDIATE`. Only hit: `pidfile::acquire` (process-level, not message-level). **T-2007's finding stands.**

### 3.5 Forcing constraints surfaced by the map

1. **No background threads by design (T-1155).** This kills the "claim-reaper task" approach. Expiry must be **lazy** (computed at read/claim time, not by a sweeper).
2. **Cursors are per-subscriber, fully independent.** A separate `claims` table is required for topic-level exclusivity. Cannot piggyback on cursors.
3. **Bus already uses SQLite for records + cursors.** Schema migration adds a new table; consistent with existing pattern (no novel storage technology).
4. **Schema init lives in one place** (`bus/src/meta.rs::init_schema()`). Migration risk is low.

## 4. Three semantic shapes — sketches

### 4.1 Hold-claim-until-ack

**Sketch.** Subscriber calls `claim(topic, offset)` → hub records `claimed_by=<subscriber_id>, claimed_at=<ts>`. Topic post at that offset is invisible to other subscribers' subsequent fetches until either `ack(topic, offset)` (claim → consumed, cursor advances) or `release(topic, offset)` (claim removed, post visible again).

**Failure modes.**
- Subscriber dies after claim, before ack: post is stuck. Mitigation requires a separate timeout sweep — but timeout = lease, see §4.2.
- Concurrent claim of same offset: hub must serialize, last-writer-loses or first-wins-only.

**Cost.** Lowest mechanical complexity. Hub-side state: one claimed_by per (topic, offset).

### 4.2 Lease-with-renewal

**Sketch.** `claim(topic, offset, lease_secs)` → hub records `claimed_by, claimed_until=<ts + lease_secs>`. Subscriber must `renew(claim_id, lease_secs)` before `claimed_until` or the claim expires and the post becomes claimable again. `ack` ends the lease and advances cursor.

**Failure modes.**
- Subscriber dies: lease expires naturally — self-healing. Good.
- Renewal-during-blip: client-side reconnect (T-2023) must integrate or lease expires while spoke is online but disconnected.
- Lease TTL choice: too short = renewal storm; too long = slow recovery from worker death.

**Cost.** Higher complexity (renewal RPC, hub-side sweeper). Hub-side state: claim_id, claimed_by, claimed_until per claim.

### 4.3 CAS-on-offset

**Sketch.** No new verb. Existing `ack(topic, offset)` becomes compare-and-swap: `ack(topic, expected_offset, new_cursor)` succeeds only if the subscriber's current cursor is still at `expected_offset`. Multiple subscribers race; one wins, others get retry. The "claim" is the successful ack; no separate state.

**Failure modes.**
- Worker fetches post but hasn't ack'd yet — another worker can also fetch and ack first. Worker B does the work, worker A's later ack fails → A must detect and skip.
- Requires every worker to be idempotent on the work itself (do-then-ack: if ack fails, the work was duplicated).
- For non-idempotent work this shape is unsafe.

**Cost.** Lowest hub-side state (zero — uses existing cursors). Highest client-side complexity (idempotency requirement).

## 4.4 Sketch evaluation against substrate map

### 4.1 hold-claim-until-ack — **rejected**

Substrate has no background threads (T-1155). Without a sweeper, a dying worker's claim sits forever. Could be salvaged with a max-claim-age check at every subsequent claim — but at that point the semantics ARE a lease with implicit infinite-or-bounded TTL. Choosing 4.1 over 4.2 just means hiding the lease behind imprecise naming. No.

### 4.3 CAS-on-offset — **rejected**

Two distinct failures:

(a) Per-subscriber cursors are independent. Two workers can both pass their local CAS (each ack'ing their own cursor) without ever competing over the SAME offset slot. There is no shared "consumed offset set" in current state, so CAS does not actually serialize.

(b) Even if we add a shared consumed-offset table to make CAS topic-level, the work-then-ack pattern leaves the dying-worker case unsolved (ack-before-work duplicates on death; work-before-ack races on completion). CAS without a TTL is just "permanent claim" with the same dying-worker hole as 4.1.

CAS-on-offset is only safe when the work is **idempotent**, and AEF agent work — file writes, git commits, task ledger mutations — is not.

### 4.2 lease-with-renewal — **selected**

- **Lazy expiry honors T-1155 no-background-threads.** Compute `WHERE claimed_until IS NULL OR claimed_until < now` at every claim/list — no reaper needed.
- **Dying-worker self-heals** — lease expires, post becomes claimable again.
- **Renewal-during-blip integrates with T-2023** (client-side reconnect + outbound queue) — the renewal RPC rides the existing resilience layer; no special-case path.
- **Trade:** higher mechanical complexity than 4.1 (one renewal verb, lazy-expiry queries) but the only shape that survives the substrate's structural constraints.

## 5. IW disposition

### IW-1: Semantics — **lease-with-renewal, lazy expiry**

**Confidence:** 3 (verified against substrate map + use-case analysis).
**Disposition:** answered.
**Rationale:** §4.1 rejected (dying-worker hole + no reaper); §4.3 rejected (per-subscriber cursors don't serialize + idempotency assumption fails for AEF work); §4.2 is the only shape that honors T-1155 (no background threads) and self-heals on worker death. Lease semantics emerge as forced, not chosen.

### IW-2: Storage — **persisted SQLite, new `claims` table**

**Confidence:** 3.
**Disposition:** answered.
**Rationale:** Bus is SQLite-backed by design (records, cursors, offsets all live there). In-memory claims would lose state on hub restart, breaking the durability promise that channel logs + inbox spool already meet — and would interact poorly with T-2025 (persistent presence) and T-2023 (reconnect) which presume hub restart is a recoverable pause, not loss. Schema migration adds one table via `bus/src/meta.rs::init_schema()`. Proposed shape:
```sql
CREATE TABLE claims (
  claim_id        TEXT PRIMARY KEY,
  topic           TEXT NOT NULL,
  offset          INTEGER NOT NULL,
  claimed_by      TEXT NOT NULL,
  claimed_at      INTEGER NOT NULL,
  claimed_until   INTEGER NOT NULL,
  UNIQUE(topic, offset)
);
CREATE INDEX claims_topic_until ON claims (topic, claimed_until);
```

### IW-3: Cursor interaction — **orthogonal**

**Confidence:** 2.
**Disposition:** answered (refinement allowed at build time).
**Rationale:** Cursors are per-subscriber read-position; claims are topic-level exclusivity — different axes. A subscriber's cursor MAY advance past a claimed offset (they're allowed to walk forward looking at other offsets) without releasing the claim. `release(claim_id)` is the explicit unclaim. `ack(claim_id)` is shorthand for release + cursor-advance-past-this-offset. Open at build: whether `release` and `ack` are the same verb with a flag, or two verbs (likely two — different intent).

### IW-4: New verb vs subscribe flag — **new verbs (three)**

**Confidence:** 3.
**Disposition:** answered.
**Rationale:** A flag on `subscribe` conflates read with side-effect, breaking the read-only-iteration invariant the existing subscribe handler relies on (long-poll, conversation filters, in-reply-to filters). Three new RPC verbs in `crates/termlink-protocol/src/control.rs::method`:
- `channel.claim` — params: `{topic, offset, lease_secs, claimed_by}`; returns `{claim_id, claimed_until}` or `claim_taken` error.
- `channel.renew` — params: `{claim_id, lease_secs}`; returns `{claimed_until}` or `claim_expired`/`not_found` error.
- `channel.release` — params: `{claim_id, ack: bool}`; if `ack=true` also advances the claiming subscriber's cursor past the released offset. Returns `{ok}`.

Each adds the 3-surface change pattern from §3.3. Bounded scope.

## 6. Recommendation

**Recommendation:** **GO**

**Rationale:** The §6.1 problem statement holds against the substrate map. The path to a safe claim primitive is forced by structural constraints (T-1155 no-background-threads + per-subscriber-independent cursors + non-idempotent AEF work) into one shape — lease-with-renewal, lazy expiry, persisted SQLite claims table, three new RPC verbs. No design freedom remains on the load-bearing decisions; the IW questions all dispose cleanly with confidence ≥2. Build scope is bounded: one new SQLite table, three new RPC handlers, three new method constants. Backward-compat: existing subscribe/post/cursor flows are unchanged; new verbs are additive.

**Evidence:**
- Substrate map verified the §6.1 capability absence (§3.4 = zero existing claim patterns).
- §3.5 surfaced two forcing constraints (T-1155, per-subscriber cursors) that eliminate 4.1 and 4.3.
- §3.3 confirmed the verb-addition surface is 3 files (low blast radius).
- §3.5 confirmed schema migration is low-risk (single init_schema function).

**Build estimate (rough):** 1-2 days for slice-1 (schema + claim/release verbs + tests); +1 day for slice-2 (renew + lazy-expiry queries); +1 day for client-side helpers + integration tests. Sub-5-day primitive.

**Carve-outs (NOT in T-2019 scope):**
- **Per-verb auth scopes** (T-1159 on-hold). Claim verbs SHOULD be scope=execute when scopes land, but adding scope enforcement isn't blocking T-2019.
- **Consumer-group / fan-out semantics** (multiple workers + load-balancing policy). Out of scope; covered by T-2021 (pull/assign) which builds on T-2019.
- **Worker liveness signal** (notify hub when worker dies to release claim eagerly). Lazy expiry is sufficient; eager release is optimization, not correctness.
- **Cross-hub federation** (claims visible across hubs). Out of scope; single-hub semantics first per ADR §8.

**Open refinements (allowed at build time):**
- Whether `ack` is a flag on `release` or its own verb (IW-3 §). Resolve at build.
- Default lease TTL (60s? 300s? — measure under real worker load).
- Whether `claim` includes a wait-mode (block until claimable) or strict NOWAIT (return error if all offsets claimed). Probably NOWAIT for simplicity; wait-mode is composable via long-poll.

**Anti-recommendation triggers (re-evaluate if these surface):**
- A new substrate fact emerges that lazy expiry is unsound (e.g., concurrent claim/expire race that SQLite alone can't serialize).
- A T-2020 design constraint forces a different shape on `claimed_by`.
- AEF orchestrator design (separate repo) reveals that claim semantics need to be cluster-wide before single-hub ships.

## Dialogue Log

*(captures human dialogue per C-001. Each entry: who asked, what was answered, what was decided.)*

### 2026-06-07 — inception opening

- **Operator:** "1" (proceed into T-2019 spike)
- **Agent:** opened this artifact per C-001; dispatching Explore agent for substrate map; planning to fill §3 from findings, then evaluate §4 sketches against the IW questions, then write the recommendation.
