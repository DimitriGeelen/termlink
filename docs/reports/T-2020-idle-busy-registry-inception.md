# T-2020: Hub-owned idle/busy agent registry — inception research

**Status:** Inception (started-work, owner: human). Research in progress.
**Arc:** `arc-parallel-substrate` (arc-001).
**ADR:** [`docs/architecture/parallel-execution-substrate.md`](../architecture/parallel-execution-substrate.md) §6.2.
**Predecessor research:** T-2007 (substrate-reality-as-it-is); T-2019 (claim semantics — landed).
**Authoring rule:** Per C-001, this file is updated incrementally during the inception; it IS the thinking trail.

---

## 1. Problem (verbatim from §6.2)

> Hub-owned idle/busy registry. No hub-tracked agent state, no "next idle worker for role X," no in-flight counter. Today heartbeats land in a topic and classification is client-side. The orchestrator needs a reliable picture of who is free.

The orchestrator cannot make safe assignment of work-units without answering: *which agent is alive, idle right now, and capable of role X?* Without that, every assignment is either guesswork (push and hope) or a separate pull-based race over a broadcast topic (which T-2019's claim primitive can now serialize, but at the cost of making EVERY worker contend for EVERY work-unit regardless of whether they want it).

## 2. IW questions to resolve (from task file)

- **IW-1:** Hub-tracked vs server-side-derived from heartbeat topic?
- **IW-2:** Granularity — by agent_id, by role, by capability tag?
- **IW-3:** Update rate — pushed by worker on transition vs polled by hub on each assign?
- **IW-4:** Race resolution — worker says BUSY but hub thinks IDLE: who wins?

The Recommendation block must dispose all four before GO is justified.

## 3. Substrate map (what exists today)

### 3.1 Agent-presence topic and heartbeat

- **Topic:** `agent-presence` (per-hub, append-log, hub-local — G-060). Created on-demand by first emit; retention defaults to a finite N/D.
- **Emitter:** `scripts/listener-heartbeat.sh` (T-1832). Posts every `--interval` seconds (default 30s). Payload is a free-form role string ("listener", "responder", ...). Metadata carries `agent_id`, `pty_session`, `listen_topics`, optional `metadata.role` override.
- **Classification:** purely client-side. Per `listener-heartbeat.sh:23-25`:
  - heartbeat newer than 2×interval → **LIVE**
  - heartbeat between 2×interval and 5×interval → **STALE**
  - heartbeat older than 5×interval → **OFFLINE**
- **Reader scripts:**
  - `scripts/agent-listeners.sh` (T-1833): single-hub discovery. Walks the topic via `channel subscribe`, dedups by `agent_id` (latest envelope wins per agent), computes status by heartbeat age.
  - `scripts/agent-listeners-fleet.sh` (T-1837): cross-hub fan-out with status precedence `LIVE > STALE > OFFLINE`. Same client-side classification per hub.
- **Filter affordances** (today): `--filter-agent-id`, `--filter-role`, `--include-offline`. So role-aware discovery already works client-side.

### 3.2 What the presence topic does NOT carry

- **Busy/idle state.** A worker holding a claim looks identical to a worker between claims. The heartbeat says "alive", not "available for work".
- **Capability tags.** Only `role` (single string, free-form). No structured `capabilities[]` array.
- **In-flight counter.** No worker emits "I am currently processing offset X" — only that it is alive.

### 3.3 What the claim primitive (T-2019, shipped) does carry

- **claims table** (`crates/termlink-bus/src/meta.rs:587-598`): one row per active claim. Schema: `(claim_id, topic, offset, claimed_by, claimed_at, claimed_until)`. Lazy-evicted past `claimed_until`.
- **Per-topic affordance** (`channel claims-summary --all`, Slice 9): tells the operator "how busy is each topic" — but NOT "which workers are busy" globally.
- **What's NOT in claims:** which agents are alive but NOT holding a claim right now (i.e. the idle set the orchestrator needs).

### 3.4 The asymmetry surfaced by the substrate map

The presence topic and the claims table together carry *almost* everything needed for the registry, but with two gaps:

1. **Idle set is the SET DIFFERENCE** of (LIVE-on-presence) − (has-active-claim-in-claims-table). This is a join across two data sources. Today no RPC computes it server-side.
2. **Capability matching** is structurally weak — `role` is a single free-form string. A worker that can do `build` AND `test` either picks one or hopes naming convention captures both.

### 3.5 T-1991 lesson, re-read

The ADR cites T-1991 as "agent-presence topic bloated from heartbeating at ~20 agents, slowing discovery." Reading T-1991's actual inception (`.tasks/completed/T-1991-agent-presence-topic-bloat--discovery-sl.md`): A1 (topic bloat is the cost) was **DISPROVEN**. The regression was per-binary-version (0.11.473 vs 0.11.472 — a per-version concurrency bug in `channel info`), not topic-size. Subscribe is O(1) on cursor depth.

**Implication for T-2020:** the bloat concern is real (1493 envelopes on .122, 13441 on .107) but does NOT block the design. Retention/compaction is the right answer (T-2028 §6 #10), not "abandon the append-log model".

## 4. The four IW questions, disposed

### IW-1: hub-tracked vs server-side-derived from heartbeat topic?

**Disposed: DERIVE, plus server-side caching for read-path performance.**

**Rationale:** A new hub-tracked SQLite table would duplicate state that already lives in `agent-presence` + `claims`. Two sources of truth = drift surface (IW-4 question becomes intractable). The append-log model is the substrate's primary surface (§2); adding a parallel agent_state table contradicts the ADR's "one writer, serialized" stance (§5).

The derivation is:
```
idle_agents(role, capability) =
  { agent_id ∈ LIVE-from-agent-presence
    WHERE metadata.role matches `role`
      AND metadata.capabilities includes `capability`
      AND agent_id NOT IN (SELECT DISTINCT claimed_by FROM claims WHERE claimed_until > now) }
```

This is O(presence_topic_size) + O(claims_table_size) per call. At fleet scale (≤30 agents per ADR §1) both are tiny. Caching a derived snapshot in the hub is a future-optimization, not a launch requirement.

### IW-2: granularity — by agent_id, by role, by capability tag?

**Disposed: BOTH role AND capability, with capability as a structured array.**

**Rationale:** Today only `role` exists (single string). For real orchestrator queries ("give me an idle worker that can build AND publish"), single-string role is insufficient. The Federation-Don't-Converge pattern (T-1165) teaches that metadata fields scale better than naming conventions — adding `metadata.capabilities: ["build", "publish"]` to the heartbeat envelope is backward-compatible (old workers post nothing, new orchestrators treat missing as empty set).

By-agent_id remains the primary key (every worker has one). Role + capability are filter predicates. Capability is an OR-set: `capabilities: ["build"]` matches any worker that has `build` in its array.

### IW-3: update rate — pushed by worker on transition vs polled by hub on each assign?

**Disposed: PULL on each assign (no new write path).**

**Rationale:** Pushing busy/idle transitions adds a write hot-path the workers don't need today. Every claim/release already mutates the `claims` table — the hub can DERIVE busy/idle from that at read time. A worker's "I just became idle" event is implicit in `channel.release(ack=true)` and persists durably.

Pull-on-assign is also more failure-tolerant: the registry is always consistent with current truth at the moment of the query, with no "stale push from 5s ago" race. The cost is O(claims_table_size) on each `agent.find_idle` call — bounded by §6 retention budget (T-2028 will cap).

Future optimization: if the call rate becomes a hot path, cache the derived snapshot in hub memory with invalidation on `claim`/`release`. Not needed for launch.

### IW-4: race resolution — worker says BUSY but hub thinks IDLE: who wins?

**Disposed: HUB WINS (the orchestrator's view of truth is authoritative).**

**Rationale:** If the orchestrator's `agent.find_idle` returns worker-X as idle and then tries to assign work to it, and worker-X's local view is "I am busy on something else", the orchestrator's claim-on-behalf-of-worker-X will succeed (the hub doesn't know about worker-X's local view). Worker-X then sees the new assignment in its inbox and must either accept (release the local work and pick up the new) or reject (refuse the claim via not-yet-defined verb).

The asymmetry matters: the hub's view is consistent across orchestrators; the worker's local view is only consistent with itself. For the orchestration plane to be sound, the hub must be authoritative. Workers that disagree must reconcile by releasing local state, not by overriding hub state.

**Edge case:** worker-X is currently holding a claim (visible to hub) but the orchestrator's `find_idle` excluded it correctly. No race here — claim primitive already serializes this.

**Edge case:** worker-X just acquired a local lock that doesn't go through the hub. Out of scope — agents are expected to route ALL coordination through the hub per ADR §3 (strict star).

## 5. Recommended design

### 5.1 Surface — one new hub RPC

```
agent.find_idle(role? string, capabilities? [string], limit? u32) → [
  { agent_id, last_heartbeat_ms, role, capabilities, hub_id }, ...
]
```

Server-side derivation (no new persistent state):
1. Walk `agent-presence` topic from offset 0 (bounded by retention) → dedup by `agent_id`, keep latest envelope per agent.
2. Filter to LIVE (heartbeat newer than 2×interval). Apply role/capability predicate.
3. EXCLUDE every agent_id in `SELECT DISTINCT claimed_by FROM claims WHERE claimed_until > now`.
4. Sort by `last_heartbeat_ms` desc (freshest first) — minor optimization for "most-likely-still-alive".

### 5.2 Heartbeat schema additions

Extend the listener-heartbeat envelope's `metadata` block:
- `metadata.capabilities`: `[string]` (default empty). Free-form tags; convention emerges by use.
- Existing fields (`agent_id`, `role`, `pty_session`) unchanged.

Backward-compat: old workers omit the field. `agent.find_idle` treats missing as empty set (so `capabilities: ["build"]` predicate matches nothing on old workers — strict). Old readers ignore unknown metadata fields (already convention).

### 5.3 CLI + MCP parity (per established Slice 4-10 pattern)

- `termlink agent find-idle [--role R] [--capability C] [--limit N] [--json]` CLI verb.
- `termlink_agent_find_idle` MCP tool.

Both for symmetry with the §6 #1 surface — orchestrator AI agents need MCP, operators need CLI.

### 5.4 What's NOT in this primitive (intentionally)

- **Worker-side push of busy/idle.** Derive everything from claim activity + heartbeat.
- **Worker pool semantics ("reserve me a slot").** That's T-2021 (pull/assign verb), which builds on this query.
- **Capability discovery / advertising registry.** Capabilities are convention-driven, not validated server-side.
- **Cross-hub agent registry.** Per G-060, each hub is independent. Cross-hub finding is the orchestrator's job (walk each hub via `hubs.toml`).
- **Persistent registry state.** Heartbeats are the durable source of truth; the registry is a derivation.

## 6. Cost / risk analysis

### 6.1 Build size

- 1 new RPC verb (handler + protocol constant + router arm) — same triplet pattern as Slice 4-10.
- ~50 LOC of derivation logic in bus library (presence-walk + claims-anti-join).
- ~30 LOC CLI verb + ~30 LOC MCP tool.
- ~20 LOC docs update.
- **Total estimated:** ~150 LOC, ≤1 session.

### 6.2 Risk

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Presence-topic-walk slow at scale (T-1991 echo) | medium | medium | T-1991 finding: walk is O(1)-cursor — slowness was version-specific. Re-bench post-build. |
| Capabilities schema drift across workers | low | low | Free-form by convention. Lint via `fw doctor` if a problem emerges. |
| Worker death between heartbeat windows | low | low | LIVE threshold (2×interval = 60s) is conservative. Orchestrator gets a soft-fail on assign; T-2023 reconnect handles transient blips. |
| New verb on old hubs (mixed-version fleet) | medium | low | `MethodNotFound` (-32601) — client falls back to client-side compute (existing scripts). No regression. |

### 6.3 Dependencies

- **Upstream (must land first):** none. T-2019 is already done; the claims-table read is well-established.
- **Soft-dep:** T-2025 (persistent presence across restart) — without it, the first heartbeat-interval after a hub restart returns "everyone unknown". Acceptable degradation.
- **Downstream:** T-2021 (pull/assign verb) calls `agent.find_idle` as its first step. T-2027 (broadcast-with-replay) is the optimization path if the presence-topic-walk becomes too slow.

## 7. Recommendation

**Recommendation: GO with the revised-scope design above.**

**Key insight that justifies GO:** the registry collapses to a DERIVATION + ONE QUERY VERB, not a new persistent table. The substrate already has both data sources (presence topic + claims table); the only missing piece is the join. This is the kind of low-risk extension the §6 manifest implicitly invited — "small, orthogonal, builds on what shipped".

**Why not DEFER:** the §9 collaboration seam lists this as a hard-dep — AEF orchestration is blocked without it. Deferring means the orchestrator either ships a hard-coded role lookup or implements client-side derivation in every consumer (T-1833's exact problem at orchestrator scale).

**Why not NO-GO:** there is no alternative model that avoids this query — either the orchestrator does the join client-side (status quo, doesn't scale), or the hub does. The hub does it once, every orchestrator gets the answer for free.

**Build slice plan (mirrors T-2019's verticalization):**
- **Slice 1:** `agent.find_idle` RPC + bus library function + unit tests.
- **Slice 2:** CLI verb `termlink agent find-idle`.
- **Slice 3:** MCP tool `termlink_agent_find_idle`.
- **Slice 4:** Heartbeat schema extension (capabilities) + listener-heartbeat.sh update.
- **Slice 5:** Documentation update + runnable example showing orchestrator → find_idle → claim → release.
- *(Optional Slice 6):* hub-side derived-snapshot cache if benchmarks show the per-call cost is non-trivial. Defer until measured.

**Open follow-up tasks (file on GO):**
- T-XXXX: `agent.find_idle` Slice 1-5 build.
- T-XXXX: heartbeat schema migration coordination (consumers + AEF).

## 8. Open questions remaining for operator

These do NOT need to be resolved before GO but are operator-decisions for the build-track tasks:

- **Q1:** Should `agent.find_idle` accept a `cross_hub: bool` parameter, walking every hub in the caller's `hubs.toml`? Or is single-hub the right cut, with cross-hub as the orchestrator's responsibility? (Maps to T-1837 pattern, but server-side composition vs client-side.)
- **Q2:** Should heartbeat-payload `role` migrate to `metadata.role` for consistency with `metadata.capabilities`, or remain in payload? (Backward-compat impact depends on choice.)
- **Q3:** Should the derivation be exposed as a STREAM (`agent.subscribe_idle`) for orchestrators that want push-notifications, or pull-only is sufficient? (T-2027 broadcast-with-replay would be the structural fit.)

## 9. ADR alignment check

- §3 (strict star): preserved — single hub computes the join, no peer-to-peer.
- §4 (collision policy): n/a — read-only verb.
- §5 (two planes): preserved — registry is a governance-plane query; workers' code-plane is untouched.
- §6 #2 manifest entry: addressed in its entirety (no scope creep).
- §6 #1 (claim semantics, T-2019): builds on it directly — `claims.claimed_by` is half the input.
- §7 (transport unification): n/a — verb works on UDS or TCP equivalently.
- §9 seam: hard-dep, contract-up-front shape — RPC signature lands first, AEF builds against it.
- §10 invariants: all preserved (no new background threads — derivation runs inline per call; no peer-to-peer; durability unchanged).
