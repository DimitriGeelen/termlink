# T-1155: Channel-based communication bus for TermLink agents

**Status:** inception / research in progress
**Task:** T-1155
**Started:** 2026-04-20
**Related:** T-163, T-690, T-908, T-1074, T-1101, T-945, T-1017, T-1018, T-1051..T-1058

## One-line framing

> **Can a channel-based bus subsume `event.broadcast` + `inbox` + `pickup envelopes` + `send-file` into one persistent, offline-tolerant abstraction, without adding a new liveness domain?**

This is the single go/no-go question. Deployment model (separate service vs inside hub), transport choice, and auth story are **outputs** of the inception, not inputs.

## Why now

Three operational pains, surfaced by the user, that a shared messaging primitive might unify:

1. **Liveness** — agents or hubs frequently offline. Current mitigations (cron jackset, startup units) are partial. Every abstraction that *requires* liveness compounds the pain.
2. **Auth/secret propagation** — rotations break trust between agents (T-1051 lineage). Every new trust domain makes it worse.
3. **Discoverability** — agents often don't know termlink is available, or what's going on across the fleet. No "fleet chat" equivalent exists.

The user's proposal: a shared comm bus — channels, 1:1, groups, messages + artifacts, like Signal/WhatsApp for agents. Deployed as a separate service.

## What we already have (fragmented partial bus)

| Primitive | Shape | Known gaps |
|---|---|---|
| `event.broadcast` + `event.poll` | Live pub/sub via hub | No persistence, no replay, no topic registry |
| `inbox` | Per-session mailbox | T-1017 silent drop, T-1018 stale chunks, per-session only |
| `kv` | Shared state | Not messaging, no change events except via poll |
| `pickup envelopes` | Async one-shot | No reply channel, one-way, no ordering |
| `send-file` | Artifact transport | T-953/PL-011 ok-on-accept ≠ ok-on-delivery |
| `termlink inject` | Direct prompt delivery | Requires receiver-up-at-send-time |

A bus done right **subsumes** these. A bus done parallel **duplicates** them.

## Assumptions to test (register via `fw assumption add`)

- **A-001** Agents want channels, not just 1:1 sessions. (Evidence we'd need: look at pickup/inbox traffic patterns for clustering.)
- **A-002** Persistent history + cursor is worth the storage+complexity cost. (Alternative: live-only pub/sub with no replay.)
- **A-003** A single trust anchor for the bus is achievable without making hub rotation harder. (Alternative: per-hub secrets inherited; then why a bus?)
- **A-004** Offline-tolerant posting is feasible — agents can queue when bus is down and replay on reconnect. (Alternative: bus becomes a new SPOF.)
- **A-005** Migration from `event.broadcast` + `inbox` + `pickup` is tractable. Need to count live call sites and estimate churn.

## Decision criteria (must all land for GO)

1. **Subsumption clear** — at least `event.broadcast` and `inbox` replaced by the channel abstraction; `pickup envelopes` and `send-file` either replaced or cleanly reduced to special cases of channel post.
2. **No new liveness domain** — the bus either runs inside the hub (already required) or is strictly optional (clients degrade to local queue when bus unreachable).
3. **Auth story plausible** — either (a) reuse hub secret per-hub with a federation layer, or (b) introduce a single fleet-wide identity that *replaces* per-hub rotation rather than adding to it.
4. **Migration path exists** — concrete plan for moving the ~N known call sites off the legacy primitives without a flag day.
5. **Storage model chosen** — log-append (Kafka-style, replay from cursor) vs rolling buffer (IRC-style) vs TTL queue (SQS-style). Each has different ops costs.

Any one of these unresolved = NO-GO / defer.

## Exploration plan (time-boxed spikes)

**S-1. Existing-primitives call-site census (30 min).**
- Count callers of `event.broadcast`, `inbox`, `send-file`, `pickup` across termlink + framework
- Classify by pattern: broadcast, 1:1, persistent-state-change, artifact-transfer
- Output: table of patterns → proposed channel shapes

**S-2. Persistence model decision spike (45 min).**
- Sketch three storage models (log-append, ring, TTL) with space/retrieval characteristics
- Map each to the 3 user pains and to the 4 subsumption candidates
- Output: ranked recommendation + disqualifiers

**S-3. Liveness/offline-tolerance spike (30 min).**
- Can clients queue posts locally when bus is down? What would a "local tail + sync on reconnect" look like?
- Does this mean every client has a mini-bus? Or an opaque queue?
- Output: viability verdict on A-004

**S-4. Auth integration sketch (30 min).**
- Three candidates: (a) bus trusts hub secrets transitively; (b) bus has its own secret per-channel; (c) fleet-wide identity issued by bus, hubs become relays
- Evaluate against T-1051 lineage + rotation pain
- Output: ranked recommendation + disqualifiers

**S-5. Migration scope estimate (15 min).**
- From S-1 output, estimate work to move each call site
- Output: go/no-go on criterion 4

Total exploration budget: **≤ 3h**. Hard stop — if not done, we're descoping, not extending.

## Scope fence

**IN:**
- Evaluating subsumption of existing primitives
- Exploring persistence, auth, liveness models at sketch level
- Migration-scope sizing

**OUT (separate tasks if GO):**
- Implementing the bus
- Specific wire protocol design
- UI/watchtower integration
- Channel ACL model beyond sketch
- Cross-cluster federation
- Non-agent uses (e.g., human-to-agent pager)

## Related prior art

- **Signal/WhatsApp** — the user's reference point. Session state, channels, artifacts, offline queue. But: centralized server, strong identity, E2E crypto. We'd reuse the UX vocabulary, not the architecture.
- **IRC** — simplest channel model. No persistence, strict liveness. Good lower bound for "how little could we do and still be useful."
- **Slack/Discord** — channels + persistence + reactions + threads. Too heavy for our use case as a baseline but useful as a vocabulary map.
- **Kafka** — log-append with per-consumer cursor. Best fit for "replay what you missed" but operationally heavy.
- **NATS/Redis pub/sub** — live-only pub/sub. Closest to current `event.broadcast`. Lower bound for bus-that-is-just-broadcast.
- **Matrix** — federated messaging protocol. Relevant if we want cross-hub without a central coordinator.

## Spike S-1: Call-site census (complete 2026-04-20)

### Production call sites

| Primitive | Files touched | Call sites | Scope |
|---|---|---|---|
| `event.broadcast` | events.rs, target.rs, tools.rs, auth.rs, router.rs, control.rs | **~2 producers** (CLI `events broadcast`, MCP pass-through) | All fan-out, no persistence, hub-local |
| `inbox.{list,status,clear}` | infrastructure.rs, remote.rs, tools.rs, router.rs | **18 call sites, 4 files** — local + remote variants of each method | Per-session queue, hub-local |
| `file.{send,receive}` | file.rs, remote.rs, main.rs, tools.rs | **~10 call sites, 4 files** — CLI + remote + MCP + 2 MCP tools | Chunked transport → inbox |
| `pickup` (shell only) | `.agentic-framework/lib/pickup.sh` | **442 lines, called from `fw`** — envelope-based on disk | Cross-project only, filesystem not hub |
| `agent.request` | events.rs (const) | **0 real use** — reserved protocol const | — |

Total Rust files affected: **8**. Total call sites: **~30**.

### Pattern classification → proposed channel mapping

| Pattern today | Example | Would become (channel model) |
|---|---|---|
| Live fan-out, no ACK | `event.broadcast {kind: "task-done"}` | `channel.post` to a broadcast topic (recipients fan-in via poll/subscribe) |
| Per-recipient queue, pull | `inbox.list target=sess-A` then `inbox.clear` | Channel with persistent history + per-recipient read cursor; `list` == "posts since my cursor" |
| Artifact transport | `file.send chunks → recipient.inbox` | `channel.post {type: artifact, payload: blob_ref}` (artifact becomes a typed message) |
| Cross-project envelope | `pickup` shell → YAML on disk | `channel.post` to an inter-project channel (or bridge shell-pickup at framework boundary) |
| 1:1 request/reply (future) | `agent.request` (unused) | `channel.post` + correlation_id; reply as another post on same channel |

### S-1 verdict

- **Subsumption mapping is clean** — every existing pattern collapses to a single primitive: `channel.post(topic, sender, type, payload, artifact?)`, with `channel.subscribe(topic, cursor?)` as the symmetric read side.
- **Call-site count is tractable** — ~30 sites across 8 Rust files + 1 shell lib. Not a large refactor.
- **Pickup is the interesting boundary** — it's shell + filesystem, not Rust + hub. If the bus is in-hub, pickup either (a) migrates up to the hub (breaks framework portability — now every framework consumer needs a hub) or (b) stays as-is and bridges via a pickup→channel adapter. Decision point for S-4/S-5.

**AC1 (S-1) ✓ CHECK**

## Spike S-2: Persistence model (complete 2026-04-20)

### Candidates

| Model | Shape | Catch-up? | Storage cost | Ops cost |
|---|---|---|---|---|
| **Log-append** | Append-only per-channel log; per-recipient cursor | Full replay from cursor | Grows until retention trigger | Retention policy + compaction |
| **Ring buffer** | Fixed-size circular log per channel | Tail-N only | Cheap, bounded | Trivial |
| **TTL queue** | Message lives until ACK or timeout | No (once read, gone) | Cheap, bounded | Per-message state machine |
| **Hybrid** (log + TTL) | Log-append for "durable" channels, TTL for "live" | Per-channel config | Medium | Complex |

### Map to user pains

| Pain | Log-append | Ring | TTL |
|---|---|---|---|
| Liveness (offline catch-up) | ✅ full | ⚠ partial | ❌ none |
| Auth | orthogonal | orthogonal | orthogonal |
| Discoverability | needs registry anyway | same | same |

### Map to 4 subsumption candidates

| Existing primitive | Log-append fit | Ring fit | TTL fit |
|---|---|---|---|
| `event.broadcast` | ✅ (replay is bonus, not needed) | ✅ | ✅ |
| `inbox` | ✅ (natural — cursor = "unread marker") | ❌ (loses messages for slow recipients) | ⚠ (read-and-forget is inbox today but pickup semantic needs history) |
| `send-file` | ✅ (durable until delivered) | ❌ (large files evict early) | ⚠ (TTL before ACK is brittle) |
| `pickup` | ✅ (pickup IS a log on disk — clean semantic match) | ❌ | ❌ |

### Storage medium

- **Per-channel append log file** on disk (e.g., `/var/lib/termlink/channels/<channel_id>/log`)
- **SQLite index** for cursor tracking and message metadata (avoids scanning the log for "give me since offset N")
- Same tooling the hub already uses (T-945 for persistent state); no new infrastructure

### Ranked recommendation

1. **🥇 Log-append + per-recipient cursor + per-channel retention policy.** Subsumes all 4 primitives cleanly. Storage cost manageable with retention (channels can be configured as `retain_forever | days=N | messages=N`).
2. **🥈 Hybrid (log-append for `inbox`/`pickup`/`send-file`, TTL for `event.broadcast`).** Only if storage cost of log-append-for-everything is actually prohibitive. Adds complexity — avoid unless measured.
3. **🥉 Ring buffer only.** Loses the pickup semantic. Disqualified.

### Disqualifiers

- Ring-only: loses "message arrived while recipient was offline" → doesn't fix liveness pain → misses core goal
- TTL-only: loses replay → misses pickup semantic → forces shell-based pickup to stay parallel, fragmenting the bus
- In-memory only (no persistence): bus restart loses all unread messages → anti-feature

**S-2 verdict: log-append with per-channel retention.** AC2 ✓

## Spike S-3: Liveness / offline tolerance (complete 2026-04-20)

### The question for A-004

Can clients post to the bus when the bus is down, and read when they come back?

### Sketch

**Client-side:**
- Small local SQLite queue: `pending_posts` table, `last_read_cursor` table per channel
- On `channel.post` call: try remote; on failure, enqueue locally and return OK
- Background sync task: periodic attempt to flush pending posts when bus is reachable
- On `channel.subscribe`: always serve from local cache first (if any), then reconcile with remote when available

**Server-side:**
- Bus is just append-log + cursor store — purely passive; clients drive all sync
- Idempotency: each post has `(sender_id, client_seq)` — duplicates deduped on bus
- Clock skew tolerated: bus authoritative for ordering, client timestamps advisory

### Viability

- SQLite queue is ~300 LOC in Rust (sqlx or rusqlite)
- Idempotent posts are a standard pattern (e.g., Stripe API `idempotency-key`)
- Cost of running: ~megabytes of client-side disk per agent, negligible

### What breaks if we don't do this

Agent A posts "I'm done" → bus is down → post is lost → agent B never learns → task stalls. This is **pain 1 manifested as pain 3**. The bus without offline tolerance is worse than the status quo because it centralizes the failure.

### S-3 verdict: offline posting is feasible and mandatory for GO.

A-004 **validated** — a bus without local queue + replay would be a regression. With it, bus outages degrade to bounded-latency posts instead of lost ones. AC3 ✓

## Spike S-4: Auth integration (complete 2026-04-20)

### Context: what the T-1051 lineage actually tells us

Per-hub HMAC secret + TLS fingerprint. On rotation, all pinned clients break. Heal path exists (T-1054/T-1055) but requires manual re-pinning. Root pain: **transport trust and identity trust are conflated in a single secret.** Rotating hub cert invalidates both.

### Candidates

**(a) Bus trusts hub secrets transitively.** Hub vouches for its local sessions.
- ⚠ Still per-hub. Rotation pain unchanged. Doesn't fix anything.

**(b) Bus has its own secret per-channel** (Matrix-style room keys).
- ⚠ Adds N secret domains. N times the rotation pain. Actively worse.

**(c) Fleet-wide identity issued by bus; hubs are relays.**
- ✅ Single rotation anchor. But creates a new SPOF (the bus is the trust authority).
- ⚠ Anti-antifragile: losing the bus = losing identity.

**(d) Self-sovereign agent identity (ed25519 keypair per agent).** Bus verifies signatures. Keys pinned TOFU on first contact; rotation by signing new key with old key (or out-of-band re-trust).
- ✅ Agents own identity. Bus is transport, not trust authority.
- ✅ Hub rotation invalidates *transport* (re-pin fingerprint), not *identity* (messages still verifiably from same agent).
- ✅ Separates the two problems that T-1051 conflates.
- ⚠ Adds crypto code to every client (but ed25519 is tiny — `ed25519-dalek` is ~few hundred LOC including dependencies).

### Ranked recommendation

1. **🥇 Self-sovereign agent identity (d).** Solves the pain rather than moving it. Each agent has a long-term keypair; bus verifies signatures on every post. Transport (hub TLS/HMAC) becomes *separately* concerned with "can I trust this relay", not "is this message really from agent X".
2. **🥈 Fleet-wide identity (c).** Simpler to implement but anti-antifragile — one authority, one failure point.
3. **🥉 Per-channel secret (b).** Disqualified — multiplies the rotation pain.
4. **Transitive hub trust (a).** Disqualified — leaves the pain exactly as-is, just renames it.

### Critical insight

This is **exactly why** the auth pain hurts today: current model conflates transport and identity into one secret. Bus done right **separates them structurally**. Hub rotation becomes a transport concern only. Agents' messages remain verifiable even across rotations.

### Disqualifiers

- Any model that keeps identity = hub secret → leaves T-1051 pain unresolved
- Any central authority for identity → new SPOF violates antifragility directive

**S-4 verdict: self-sovereign agent keys (d).** AC4 ✓

## Spike S-5: Migration scope (complete 2026-04-20)

### From S-1 counts

| Primitive | Sites | Files | Migration shape |
|---|---|---|---|
| `event.broadcast` | ~2 | 2 | Replace with `channel.post(topic="broadcast:global")`. Callers wrap, semantics preserved. |
| `inbox.*` | 18 | 4 | Replace with `channel.{post, subscribe}`; `target` becomes `recipient_id`. CLI + MCP + hub + remote — all 4 files. |
| `file.send/receive` | ~10 | 4 | Artifact becomes `channel.post {type: artifact}`; chunked transfer is implementation detail of bus. |
| `pickup` (shell) | ~framework wide | 1 shell lib | Keep shell pickup; add `pickup → channel bridge` at framework boundary. Shell stays portable. |
| `agent.request` (unused) | 0 | — | Define as correlated `channel.post`. Zero migration cost. |

**Total migration: ~30 Rust call sites + 1 shell adapter.** Estimated bounded effort.

### Effort estimate

| Component | LOC estimate | Notes |
|---|---|---|
| Bus core (append log + cursor + subscriber API) | 1500–2500 | New crate `termlink-bus` |
| Client local queue (SQLite + flush task) | 300–500 | In `termlink-session` |
| Ed25519 identity keyring | 200–400 | Reuse `ed25519-dalek` |
| Channel API (CLI + MCP surface) | 400–700 | New `commands/channel.rs`, new MCP tools |
| Migration shims (`event.broadcast`, `inbox`, `file.*` → `channel.*`) | 500–1000 | Per-call-site wrappers, keep legacy methods during transition |
| Shell pickup → channel bridge | 100–200 | One adapter script |
| Tests | 1000–1500 | Unit + integration + router + e2e |

**Total: ~4000–6000 LOC over 3–5 weeks of focused work** (one clean sprint). Well-bounded.

### Migration strategy (no flag day)

1. **Phase 1:** ship `termlink-bus` crate + `channel.*` API alongside existing primitives. Both work. New consumers use `channel.*`.
2. **Phase 2:** migrate internal callers one primitive at a time (`event.broadcast` first — smallest). Validate. Then `inbox`. Then `file.send`. Deprecate each as its callers move.
3. **Phase 3:** shell pickup bridge ships. Legacy filesystem pickup keeps working for external projects; new projects can post directly to bus channel.
4. **Phase 4:** remove legacy primitives (after N months of parallel operation + deprecation warnings).

**S-5 verdict: migration is tractable.** AC5 ✓

## Recommendation

> ### **GO — build the bus, in-hub, log-append, self-sovereign identity, offline-tolerant client.**

### Rationale

All 5 go/no-go criteria met:

1. **Subsumption clear** (S-1): all 4 existing primitives collapse cleanly to `channel.post` + `channel.subscribe`; ~30 call sites across 8 files.
2. **No new liveness domain** (S-3): bus runs inside the existing hub process; clients degrade to local SQLite queue + replay when bus unreachable. Offline posting is feasible and mandatory.
3. **Auth story resolves the underlying pain** (S-4): self-sovereign ed25519 agent keys separate *identity trust* from *transport trust*. Hub secret rotations (T-1051 lineage) stop invalidating messages. Per-hub rotation becomes a transport concern only.
4. **Migration path exists** (S-5): bounded, ~30 sites, phased over 4 milestones with no flag day. Shell pickup stays portable via bridge.
5. **Storage model chosen** (S-2): log-append with per-channel retention policy. Natural semantic for all subsumed primitives. Storage cost manageable.

### Evidence

- S-1: 30 call sites across 8 files — bounded refactor
- S-2: log-append is the only model that cleanly subsumes `pickup` + `inbox` + `event.broadcast` + `send-file`
- S-3: 300-LOC SQLite queue makes bus resilient to its own outages
- S-4: ed25519 identity decouples message authenticity from hub transport — structural fix for T-1051 lineage, not a workaround
- S-5: ~4000–6000 LOC effort, 3–5 weeks, phased migration

### Build scope (separate tasks if GO)

Proposed follow-up tasks to create after decision:

- **T-11XX** Build `termlink-bus` crate: log-append + cursor + subscribe API + retention engine (in-hub)
- **T-11XX** Add ed25519 identity keyring to `termlink-session` (with bootstrap + rotation commands)
- **T-11XX** Add `channel.{post, subscribe, list, create}` API surface (CLI + MCP + hub router)
- **T-11XX** Add client-side offline queue (SQLite + flush task) in `termlink-session`
- **T-11XX** Migrate `event.broadcast` callers → `channel.post(topic="broadcast:global")`
- **T-11XX** Migrate `inbox.*` callers → `channel.{post, subscribe}` to recipient channel
- **T-11XX** Migrate `file.send/receive` → `channel.post {type: artifact}` with chunked artifact transport
- **T-1165** Shell pickup → channel bridge (one adapter) — keeps framework portability. **Shipped 2026-04-24.** `lib/pickup-channel-bridge.sh` invoked from `pickup_process_one` after envelope moves to `processed/`. Capability-probes `termlink channel post` (Tier-A, T-1160) first; falls back to `termlink event broadcast` (present in all known lineages) when `channel` subcmd is absent. Silent no-op when termlink is missing or hub unreachable. SHA-256 dedup + `FW_PICKUP_CHANNEL_BRIDGE=0` opt-out. **Decision: one-way by design** — bus subscribers observe pickups but cannot inject new pickups via the bus (per T-956 pickup-distinct guidance). If bidirectional ever becomes desirable, open a separate task; doing it here would blur pickup-semantics with channel-semantics.
- **T-1168 B1** Learnings bus publisher — **Shipped 2026-04-24** (framework commit `550a9ce0`). `lib/publish-learning-to-bus.sh` invoked from `do_add_learning` immediately after `learnings.yaml` persist. Env-driven (L_ID/L_LEARNING/L_TASK/L_SOURCE/L_DATE), posts to `channel:learnings` with origin_project + origin_hub_fingerprint. Capability-probes `termlink channel post` first, falls back to `event broadcast`. `FW_LEARNINGS_BUS_PUBLISH=0` opt-out.
- **T-1217 / T-1168 B2** Learnings bus subscriber — **Shipped 2026-04-24** (framework commit `87d2ca2d`). `lib/subscribe-learnings-from-bus.sh` cron-drivable poller. Consumes via `event collect --topic channel:learnings --payload-only` — discovery spike confirmed receives broadcast payloads, but returns **one copy per listening hub session** per broadcast, so composite-key dedup `(origin_project, learning_id)` is load-bearing. Self-filter skips envelopes whose `origin_project` matches ours. Appends to `.context/project/received-learnings.yaml`. Recommended install: `*/5 * * * *` with 30-sec timeout. `FW_LEARNINGS_BUS_SUBSCRIBE=0` opt-out.
- **T-1218 / T-1168 B3** Watchtower "Received from peers" section on `/learnings` — **Shipped 2026-04-24** (framework commit `9bfdc5d5`). `load_received_learnings()` helper + pass-through in the discovery blueprint + `<details>` block in `templates/learnings.html`. Auto-hidden when the received list is empty; no new route, no nav change.
- **T-11XX** Retire legacy primitives after N months (separate decommission task)

### Open questions deferred to build phase (not blocking decision)

- Wire format: JSON (simple, debuggable) vs CBOR/msgpack (smaller, faster) — pick at build time based on throughput measurements
- Retention defaults: per-channel `retain=forever | days=90 | messages=10k` — sketch now, tune with real traffic
- Cross-hub federation (multi-hub-as-one-bus): **explicit OUT of scope** for first milestone; single hub is the MVP
- Channel ACLs beyond "anyone-who-authenticates can post": **OUT of scope** for first milestone

### Risks (cited, not blocking)

- **R1:** Migration takes longer than estimated (classic software risk). Mitigation: phased migration; legacy primitives keep working during transition.
- **R2:** ed25519 key bootstrap UX is awkward for humans. Mitigation: `termlink identity init` generates; `termlink identity show` prints fingerprint; TOFU on first-contact like SSH.
- **R3:** Offline queue bloats on long outages. Mitigation: cap local queue size, reject new posts when full (surfaces the problem loudly rather than silently growing).

## Dialogue Log

### 2026-04-20 — initial user prompt

> *User:* "We keep having difficulties with establishing reliable communication protocol between termlink agents. One cause is that often the agent or the hub is not online. Another issue is authentication, certificates, secrets not being being recreated, not being able to share between agents. A third one is the agent not knowing that it can use termlink. Can we create a shared communication bus, like WhatsApp or Signal for termlink agents? Where they can create channels they can subscribe to, post messages and artefacts in. Also one-to-one communication. Set this up as a separate service."

> *Agent reflection (summarized):* Three pains are distinct. Bus genuinely addresses discoverability; moves liveness pain unless it subsumes the hub; compounds auth pain unless it replaces per-hub secrets. Recommend inception scoped to subsumption question rather than the general "should we build it" framing. User: "yes".

### Next dialogue checkpoint

To be filled when spike S-1 output is ready for discussion.

## Build log — T-1160 (channel.* RPC surface)

**Landed:** 2026-04-20. Wires the T-1158 bus core and T-1159 identity into
a usable API — hub router, CLI, MCP tools.

### Frozen protocol wire format

All four verbs are **Tier-A** (opaque) per T-1133 — adding fields to the
JSON payload does not break older peers.

| Method | Params | Result |
|---|---|---|
| `channel.create` | `{name, retention: {kind, value?}}` | `{ok, name, retention}` |
| `channel.post` | `{topic, msg_type, payload_b64, artifact_ref?, ts, sender_id, sender_pubkey_hex, signature_hex}` | `{offset, ts}` |
| `channel.subscribe` | `{topic, cursor?, limit?}` | `{messages, next_cursor}` |
| `channel.list` | `{prefix?}` | `{topics: [{name, retention}]}` |

Retention kind is `"forever" | "days" | "messages"`; `value` is ignored
for `forever` and required for the other two.

**Error codes** (added to `error_code` in `termlink-protocol::control`):

- `-32012 CHANNEL_SIGNATURE_INVALID` — pubkey or signature failed to parse
  or did not verify against the canonical bytes.
- `-32013 CHANNEL_TOPIC_UNKNOWN` — post/subscribe targeted a topic the hub
  has no record of.

**Canonical signing bytes** (`control::channel::canonical_sign_bytes`):

```
u32 len(topic)    | topic bytes
u32 len(msg_type) | msg_type bytes
u32 len(payload)  | payload bytes
u32 len(artifact) | artifact bytes   (empty if absent)
i64 ts_unix_ms                       (big-endian)
```

Every field is length-prefixed big-endian so a future addition cannot
retroactively validate an older signature. `artifact_ref=None` and
`artifact_ref=Some("")` are intentionally equivalent on the wire — both
encode zero-length.

**Subsumption targets.** These four verbs cover the migration paths
queued in T-1162..T-1166:

- `event.broadcast` → `channel.post(topic=broadcast:global, msg_type=...)`
- `inbox.{list,status,clear}` → `channel.{subscribe,list}` on per-recipient topics
- `file.send/receive` → `channel.post(msg_type=artifact, artifact_ref=...)`

**Backward-compat.** `event.broadcast`, `inbox.*`, `file.*`, `event.emit`,
every Tier-B typed method — all continue to work unchanged.
`CONTROL_PLANE_VERSION` bumped from 1 to 2 as a presence flag; it does
NOT gate any existing method because every new method is Tier-A.

### Where to find each piece

| Concern | File |
|---|---|
| Protocol constants + sig layout | `crates/termlink-protocol/src/{lib,control}.rs` |
| Hub router handlers | `crates/termlink-hub/src/channel.rs` |
| Hub routing table | `crates/termlink-hub/src/router.rs` (in `route()`) |
| Server `init_bus` | `crates/termlink-hub/src/server.rs` |
| CLI verbs | `crates/termlink-cli/src/commands/channel.rs` |
| MCP tools | `crates/termlink-mcp/src/tools.rs` (doctor reports 73 tools, was 69) |

### Live smoke (local hub)

```
$ termlink channel create channel:smoke --retention messages:100
$ echo "hello bus" | termlink channel post channel:smoke --msg-type note
Posted to channel:smoke — offset=0, ts=1776721023139
$ termlink channel subscribe channel:smoke
[0] d1993c2c3ec44c94 note: hello bus
```

## Recommendation

*[to be written after spikes S-1..S-5 complete]*

## Decision

<!-- inception-decision -->

*[to be filled via `fw inception decide T-1155 go|no-go`]*
