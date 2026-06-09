# T-2089: Broadcast-with-replay / current-value key — inception

**Status:** Inception in progress
**ADR:** [parallel-execution-substrate §6 #9](../architecture/parallel-execution-substrate.md)
**Scope:** Substrate primitive #9 — last gap (alongside #8) in the §6 build manifest.
**Created:** 2026-06-09

---

## 1. The problem

**ADR quote (§6 #9):**

> "Broadcast-with-replay / current-value key surfaced to a subscriber on registration — for late-joiner 'room state' without replaying an entire log."

Two related operator pain points exist today:

### 1a. Late-joiner state

When a fresh agent session subscribes to a topic (e.g. `agent-chat-arc`, `agent-presence`, a `dm:<a>:<b>` thread, an internal coordination topic), the only ways to see prior state are:

| Path | Cost | Race window |
|---|---|---|
| `channel.subscribe cursor=0` | O(N) envelopes streamed | Walks the entire log; for high-rate topics (presence: 200-msg cap, then ~thousands/day before T-1991) this is expensive |
| `channel.snapshot` | O(N) envelopes walked server-side | Better than client-side walk but the server still walks everything |
| `channel.state_since since=<ts>` | O(N) walked, filtered to recent | Better only if "recent" is small |

There is **no O(K) path** where K = the number of distinct "rooms" or "keys" with current state. For `agent-presence` with ~10 known agents and 10K heartbeats, the operator's "who's around right now?" query costs 10K envelope walks to produce 10 rows.

### 1b. Identity per use-case

Even the existing `channel.snapshot` doesn't address what a subscriber actually wants. The "current value" varies by topic:

| Topic | Current-value question | Today's path |
|---|---|---|
| `agent-presence` | "Latest heartbeat per agent_id" | Client walks last-N envelopes, groups by agent_id, applies LIVE/STALE/OFFLINE TTL |
| `agent-chat-arc` | "Last K posts as conversation tail" | Client walks last-K envelopes |
| `dm:<a>:<b>` | "Last read-receipt per sender; conversation tail" | Two walks |
| `topic-claims:<X>` | "Active claim per offset" (already supplied by `channel.claims-summary` since T-2042) | Server already maintains; this is the model |

The substrate already implements one cv-keyed read pattern (claims-summary), but it's hard-coded for claim semantics. Generalizing it would unlock the rest.

---

## 2. Design alternatives

### A. Tagged-post current-value (RECOMMENDED)

**Model.** Posters opt-in by adding `metadata.cv_key=<string>` to a post. The hub maintains a per-topic in-memory map `cv_index[topic] -> {cv_key -> latest_offset}`. On subscribe with `--include-current-value=true`, the hub looks up the map, fetches those specific envelopes by offset, and delivers them as a synthetic prefix to the live stream.

**Storage.** Pure index; rebuilt on hub restart by one O(N) scan per topic at startup (already done for `agent-presence` retention bookkeeping per T-1991). Steady-state add cost is O(1) — append-or-update on each post that carries `cv_key`.

**RPC surface.** Extend existing `channel.subscribe` with one optional param:
```
channel.subscribe(topic, cursor, limit, ..., include_current_value: bool=false)
```
When set, the server prepends the cv_index entries to the response (each carries its real envelope offset; client can dedupe vs live stream).

**Pros.**
- Minimal new state (one HashMap per topic).
- Bounded delivery — exactly K envelopes where K = distinct cv_keys for the topic.
- Posters control granularity.
- Backward compatible — old subscribers don't see anything new.
- Restart-safe — index rebuilt on first scan.
- Single-call semantics — eliminates the snapshot+subscribe race window.

**Cons.**
- Requires posters to opt-in via metadata. Doesn't help topics where no poster knows about cv_key.
- "Latest by cv_key" is monotone — a poster cannot un-set a key (would need a redaction or a sentinel tombstone value). Same constraint as channel.snapshot already has.

**Storage size.** For agent-presence with 50 agents at peak: 50 entries × ~50 bytes = 2.5 KB. For agent-chat-arc with no cv_keys: 0 bytes. Negligible.

### B. Last-N tail replay

**Model.** `channel.subscribe --include-last-n=K` returns the last K envelopes immediately, then the live stream.

**Pros.** Trivial implementation (inverse cursor). No opt-in from posters.

**Cons.** "Last N" is arbitrary — N must be tuned per topic. For agent-presence with 50 agents and round-robin heartbeats, N=50 gives one per agent. With 10 agents heartbeating at different rates, N=50 might give 49 heartbeats from one agent and 1 from another. Semantically wrong for the "current value per key" question.

**Verdict.** Useful as a complementary surface ("just give me recent context"), but does not solve #9.

### C. Compositional snapshot+subscribe

**Model.** Document a recipe: client runs `channel.snapshot` first, then `channel.subscribe cursor=<latest_offset_from_snapshot>`. No new server-side code.

**Pros.** Zero new state. Already possible.

**Cons.**
- Two round-trips.
- Race window — a post arriving between snapshot and subscribe is either double-delivered or lost depending on cursor handling.
- Snapshot still walks the entire log.
- Doesn't address the O(N) cost — just packages it.

**Verdict.** Not what §6 #9 calls for ("without replaying an entire log").

### D. Key/value sidecar

**Model.** Topics have an optional KV sidecar table: `topic_cv: {key: value, ...}`. Each post can update one or more keys via `metadata.cv_updates={key: value, ...}`. Subscribers receive the current KV map.

**Pros.** Pure room-state semantics. Collapsed-by-construction.

**Cons.**
- Significant new state surface — second SQLite table, migration, GC policy.
- "Latest envelope per key" is structurally equivalent (A), just with the bookkeeping pushed into the storage layer instead of an in-memory index.
- The value lives outside the message log — breaks the "everything is an envelope" invariant.

**Verdict.** Strictly larger than A with no functional gain. Reject.

---

## 3. Recommendation

**RECOMMEND GO on Design A (tagged-post current-value), implemented as a thin slice over `channel.subscribe`.**

### Why A wins

1. **Bounded build cost** — one in-memory HashMap, one optional param on existing RPC, one startup-scan addition. No schema migration. No new RPC.
2. **Backward compatible** — opt-in by both poster (cv_key metadata) and subscriber (include_current_value flag). Old clients are unaffected.
3. **Directly unlocks the operator pain point on `agent-presence`** — current `/peers` walks ~10K heartbeats to produce 10 rows. With cv_key=agent_id on heartbeat, becomes O(N_agents).
4. **Symmetric with the existing claim-index** — claims-summary already does this for claim envelopes; this generalizes the pattern.
5. **Composable with the obs arc** — `subscribe --include-current-value` is the write/read primitive; downstream verbs can build on it.

### Why not (B), (C), (D)

- (B) solves a different problem (tail context, not room state) — useful but not #9.
- (C) is what we have today; the ADR explicitly says this isn't enough.
- (D) is strictly larger than (A) with no gain.

### Decision deferred until the human approves the build slice

This is INCEPTION — the recommendation is for the human to approve via Watchtower or `fw inception decide T-2089 go`. The plan below assumes a GO decision.

---

## 4. Build slice plan (post-GO)

Mirrors the proven 5-slice substrate observability pattern (T-2078..T-2087, T-2062..T-2071). Each slice is one independent build task that closes a vertical.

| Slice | Task | Scope |
|---|---|---|
| 1 | T-2090 | **Hub-side cv_index**: in-memory `HashMap<topic, HashMap<cv_key, offset>>`. Wired into `channel.post` (on insert if `metadata.cv_key` present). Rebuild on startup scan. Unit tests for empty, single key, key update, multi-topic. |
| 2 | T-2091 | **`channel.subscribe --include-current-value=true`** wire. New param; on true, server prepends cv-indexed envelopes to the response. Doesn't break old clients. |
| 3 | T-2092 | **CLI: `channel subscribe --include-current-value`** + **MCP parity** `termlink_channel_subscribe` accepts the new flag. |
| 4 | T-2093 | **`channel cv-keys [<TOPIC>] [--json]`** read-only inspection verb — lists `cv_key -> offset` pairs. Operator-visible health check. |
| 5 | T-2094 | **Update `/peers` (`agent-listeners.sh`)** to emit `cv_key=<agent_id>` on heartbeat AND consume via include_current_value. End-to-end smoke: `/peers` becomes O(N_agents) instead of O(N_heartbeats). |

Each slice is sized to one session. Total ≈ five sessions, parallel-safe (slices 3 and 4 can land after slice 2 in any order).

---

## 5. Acceptance criteria (inception)

### Agent
- [ ] Problem clearly stated — late-joiner state-on-subscribe gap, with measured cost on `agent-presence`
- [ ] Four alternatives analyzed (A, B, C, D) with trade-offs
- [ ] Recommendation written with rationale
- [ ] Build slice plan included as the path forward if GO

### Human
- [ ] [REVIEW] Approve GO/NO-GO via `fw task review T-2089` or Watchtower

## 6. Open questions (mirrored in task IW-1..IW-5)

- **IW-1: Does cv_index need persistence beyond the startup scan?**
  - confidence: 2 (the per-topic startup scan is already cheap because the log is bounded by retention; for agent-presence T-1991 caps it at 200 msgs)
  - disposition: answered
  - rationale: SQLite is already the durability layer for envelopes. The index is a derivative view — rebuilding on startup is the same model as the existing presence-derivation and claim-index patterns. Adding a persisted sidecar table doubles the write path without functional gain.

- **IW-2: What's the semantics when a poster sets `cv_key=X` and a later poster also sets `cv_key=X`?**
  - confidence: 3
  - disposition: answered
  - rationale: Last-write-wins on offset — the index records `cv_key=X -> latest_offset`. Symmetric with how `agent-presence` LIVE/STALE/OFFLINE is computed today (latest heartbeat per agent_id wins).

- **IW-3: Does `cv_key` namespace overlap with any existing metadata fields?**
  - confidence: 2
  - disposition: answered
  - rationale: Existing metadata keys: `conversation_id`, `in_reply_to`, `cv` (claim), `redacts`, `capabilities`, `agent_id`. No collision; `cv_key` is a new key. Document the namespace in the schema before Slice 1 lands.

- **IW-4: What if a posted envelope is later redacted — does it still show as the current value?**
  - confidence: 2
  - disposition: answered
  - rationale: Redaction collapse already exists in `compute_state`. On subscribe with `include_current_value`, the hub MUST filter redacted offsets out of the prefix delivery (otherwise users see retracted content as "current"). Slice 1 includes this filter + unit test.

- **IW-5: Should the cv_index be bounded?**
  - confidence: 2
  - disposition: answered
  - rationale: Per-topic K (distinct cv_keys) is unbounded only if posters mint random cv_keys. For known use cases (presence: agent_id; DM: maybe `read_receipt:<sender>`; chat: maybe `pinned`) K is small. Add a per-topic cap (TERMLINK_CV_INDEX_MAX_KEYS_PER_TOPIC=1000) with LOUD-refuse (PL-204 invariant) — same shape as the dedupe cap (T-2049).

## 7. Related primitives + how this composes

- **Substrate primitive #1 (CLAIM, T-2019)** — the claim-index is the existing in-memory map. This generalizes the same shape.
- **Substrate primitive #2 (DISPATCH, T-2020)** — `agent find-idle` currently does the LIVE-set computation client-side; with cv_key=agent_id on heartbeats, it can be reduced to one O(N_agents) query.
- **Substrate primitive #5 (RESILIENCE, T-2023)** — offline-queue dedupe is hub-side last-K cache. cv_index is hub-side last-1-per-key cache. Same memory model.
- **Existing channel.snapshot/state** — full-log walks. cv_index is the O(K) alternative for the "current value per key" question, leaving snapshot for "everything up to this ts".

## 8. Dialogue Log

### 2026-06-09 — initial scoping (claude → claude)

Working from ADR §6 §"Supporting" tier. Verified status of every primitive in §6 against `.tasks/completed/`:
- #1 CLAIM closed (T-2019 + obs arc T-2072..T-2077)
- #2 DISPATCH closed (T-2020 + obs arc T-2078..T-2082)
- #3 PULL/ASSIGN closed (T-2021)
- #4 FS-WRITE deferred (T-2022)
- #5 RESILIENCE closed (T-2023 + obs arc T-2083..T-2087)
- #6 SYMMETRIC AUTH deferred (T-2024)
- #7 PRESENCE doc-only (T-2025 NO-GO)
- #8 AGENT-LAUNCH open
- #9 BROADCAST-REPLAY open ← this inception
- #10 BACKPRESSURE closed (T-2028 + obs arc T-2062..T-2071)

Picked #9 over #8 because:
- ADR §6 #9 is in the "Supporting" tier — bounded scope.
- Touches every channel topic, so unlocks broad operator value.
- Existing claim-index pattern (T-2019) is the proof-of-concept — same shape, different keying.
- #8 is structurally larger (agent.checkout/commit/publish — full git-aware orchestration surface).

Verified current channel.subscribe in `crates/termlink-hub/src/channel.rs:594` — cursor-based, no current-value concept. Verified channel.snapshot in `crates/termlink-cli/src/commands/channel.rs:6644` — walks full log via `walk_topic_full`. Verified agent-presence is a known high-rate topic (channel.rs:327, comment at 353 specifically cites T-1991/G-058 retention).

Decision tree for design A vs B vs C vs D: laid out in §2 above. Recommendation A — tagged-post current-value — was the natural fit because:
1. it generalizes the existing claim-index pattern,
2. opt-in by poster avoids breaking any consumer,
3. one in-memory HashMap is the minimum viable substrate state.

Slicing into 5 build tasks follows the established T-2078..T-2087 substrate-observability arc pattern. Each slice is one session, with Slice 1 (hub state) load-bearing for all the rest.

---

## 9. References

- ADR: [docs/architecture/parallel-execution-substrate.md §6 #9](../architecture/parallel-execution-substrate.md)
- Existing claim-index pattern: T-2019, `crates/termlink-hub/src/channel.rs:923` (`channel.claim`)
- Existing presence retention: T-1991, G-058 (`channel.rs:327`)
- Existing snapshot machinery: `crates/termlink-cli/src/commands/channel.rs:6644` (`cmd_channel_snapshot`)
- T-2018 ADR (this is part of)
- L-291 (toolchain hint for Rust build verification)
