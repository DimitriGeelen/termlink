# T-2025 Inception Research — Substrate primitive #7: hub-persistent presence + circuit-breaker state

**Status:** NO-GO as captured; re-scope as documentation-only.
**Artifact created:** 2026-06-08
**Why:** The "presence is in-memory" framing in §6 does not match the running system. Presence DATA is already durable; only the DERIVED view is in-memory, and reconstruction is O(presence_topic_size) — well below human-perceptible delay at fleet scale. Circuit-breaker persistence is arguably wrong as a default.
**See also:** T-2018 ADR §3 (durability), §6 #7 (the captured framing); T-2020 inception (which named T-2025 as a soft-dep).

## 1. The §6 framing

ADR §6 primitive #7: *"Presence and circuit-breaker state are in-memory today; reset on hub restart, so liveness inference resets to everyone-unknown for one heartbeat interval after every restart. Channel logs + inbox spool DO survive, so message durability is intact; only the liveness picture is fragile."*

The framing implies two concrete gaps to fill. The investigation below shows the first gap is not actually present in code, and the second is arguably a feature.

## 2. Ground-truth: presence persistence (already in place)

The `agent-presence` topic on the local hub:

```
Topic: agent-presence
Retention: forever
Posts: 13441
Description: (none)
Senders: 1
```

`agent-presence` is a normal channel — posts route through `Bus::post()` (`crates/termlink-bus/src/lib.rs:127`), which writes to the SQLite-backed channel log. Per ADR §3 ("a hub crash is a pause, not data loss"), channel data is durable across restart.

What IS in-memory is the **derived view**: each client computing `{LIVE, STALE, OFFLINE}` per agent by walking recent heartbeats with a TTL classifier (`scripts/agent-listeners.sh`, `scripts/agent-listeners-fleet.sh`). This view is **reconstructed on every query** — it isn't lost across restart, it's never stored in the first place.

The "blackout" §6 describes is the wall-clock time between a hub restart finishing and the next heartbeat round landing — typically ≤ one heartbeat interval (~30s, configurable via the listener-heartbeat emitter). During this window, a `find_idle` call would see the prior heartbeats from before the restart (still durable in the topic) and classify based on their ages. As long as the heartbeat interval is < the STALE threshold, the window is fully covered.

**Verdict:** No data persistence work needed. The substrate already satisfies the durability requirement implicit in the framing.

## 3. Ground-truth: circuit-breaker state

`crates/termlink-hub/src/circuit_breaker.rs` implements per-session circuit-breaker state:

```rust
struct CircuitState {
    consecutive_failures: u32,
    opened_at: Option<Instant>,    // <-- in-memory, not persisted
}

const FAILURE_THRESHOLD: u32 = 3;
const COOLDOWN: Duration = Duration::from_secs(60);
```

Stored in a `Mutex<HashMap<String, CircuitState>>` keyed by session ID. Reset on hub restart by construction (the HashMap is recreated).

The question is whether persisting this across restart is **correct**, not whether it is **possible**.

**Argument against persistence:** A hub restart is a recovery event. The most common cause of opened breakers is transient — a flaky peer host that has since rebooted, a network partition that has since healed, a misconfigured proxy that has since been fixed. Persisting opened breakers across restart causes them to stay open after the underlying problem is gone, blocking traffic to peers who would now work. The hub restart is the operator's signal that "we are starting fresh"; carrying breaker state forward defeats it.

**Argument for persistence:** A genuinely broken spoke would be re-tripped within `3 × failure-interval` seconds anyway. That's a small price. But the cost is bidirectional: a healed spoke gets back online immediately on restart rather than waiting for the hub to forget. The asymmetry favors not persisting.

**Verdict:** Restart-reset is the right default. If a specific deployment needs sticky breakers (e.g. an ops team that restarts the hub frequently for unrelated reasons), make it a hub-config flag, not a substrate primitive.

## 4. What the framing actually exposes

The §6 description was filed *while-fresh* per PL-203 — before T-2019/T-2020 design clarified the substrate's stance on "derived view vs persistent state". Re-reading it now:

- "Presence … in-memory today" → **partly inaccurate** (data persists; view is derived on demand)
- "Reset on hub restart" → **also partly inaccurate** (presence data isn't reset; the heartbeat-emitter cadence is what determines the window)
- "Liveness inference resets to everyone-unknown" → **accurate for the first heartbeat interval only**, then automatically reconstructed

The genuine concern at the time was probably: *"what happens to T-2020's `find_idle` during a hub restart?"* The answer is: **first call returns whatever heartbeats were already in the topic (with their pre-restart timestamps); subsequent calls return fresh data as the heartbeat cadence resumes.** The blackout is not a data-loss event; it's an information-recency event, bounded by the configured heartbeat interval.

## 5. Open questions, resolved

- **IW-1 (storage choice — SQLite alongside channel logs, or separate keyed store):** Moot. No new storage required. The channel log already holds the data. Confidence=4.
- **IW-2 (TTL semantics — STALE vs OFFLINE thresholds, recoverable vs evicted):** Already client-side policy in `agent-listeners.sh` (default: LIVE if last seen ≤ 35s, STALE if ≤ 90s, OFFLINE otherwise). Configurable per consumer. Substrate need not enforce a global TTL. Confidence=4.
- **IW-3 (circuit-breaker scope — per-spoke, per-topic, per-target-host):** Currently per-session-id (which conflates with per-connection on TCP). A future refinement task can split per-spoke vs per-topic if a real incident motivates it. Not blocking. Confidence=3 (the present scope is workable; refinement is conditional on observed need).

## 6. Cost / risk of building T-2025 as captured

If built as the §6 framing suggests:
- New SQLite table `agent_presence_memo (agent_id, last_seen_ms, metadata_json)` — duplicates the channel log
- New SQLite table `circuit_breaker_state (session_id, consecutive_failures, opened_at_ms)` — duplicates the in-memory HashMap with the wrong default semantics
- Two new write hot-paths: every heartbeat post triggers a memo update; every transport failure triggers a breaker update
- Net effect: more state, more contention, no observable user benefit on the typical restart timeline

This is the "two sources of truth" anti-pattern T-2020 also identified and avoided. T-2020 collapsed its primitive to a DERIVATION; T-2025 collapses to a NON-PRIMITIVE.

## 7. Recommendation

**NO-GO as captured.** Re-scope as documentation-only:

1. Update ADR §6 #7 description to reflect the actual state ("presence DATA is durable; derived view is in-memory but reconstructible; circuit-breaker reset is intentional, not a gap").
2. Add a short paragraph to `docs/operations/substrate-claim-primitive.md` documenting the post-restart blackout behavior — specifically that `find_idle` returns prior-heartbeat data immediately and refreshes within one heartbeat interval.
3. If a deployment surfaces real need for sticky circuit-breaker state, file a separate **optimization** task (not a substrate primitive) with measured evidence — e.g. "operator wants the hub to restart for upgrades N times/day without losing the OPEN classification on host X".

## 8. GO criteria evaluation (from §Go/No-Go Criteria)

- ❌ "Storage chosen and tested across hub restart" — no storage needed.
- ❌ "TTL semantics documented" — already client-side policy, not substrate.
- ✅ (negative) "T-2020 can build against this" — T-2020 already builds against the existing durable topic.

The GO criteria assume the gap is real. Investigation shows it isn't. The honest verdict is NO-GO; the captured framing was an artifact of capturing-while-fresh and is corrected by this analysis.

## 9. ADR alignment check

| ADR section | Alignment |
|-------------|-----------|
| §3 "a hub crash is a pause, not data loss" | ✓ Presence DATA satisfies this already. |
| §5 "one writer, serialized" | ✓ Not persisting circuit-breaker preserves this — there's no second writer of liveness state. |
| §6 #7 captured description | ✗ Description does not match running code. Update needed. |
| §9 "hard-dep for AEF" | ✓ AEF gets its liveness picture from the same channel; no new contract required. |

## 10. Open follow-up tasks to file on NO-GO

- Documentation task: update ADR §6 #7 description to match running system; add post-restart blackout paragraph to `substrate-claim-primitive.md`. (~30 lines of doc edits, no code, blast_radius=0.)
- *(Conditional)* Optimization task — only if observed: "presence memo for sub-O(topic_size) `find_idle`". File with measured evidence, not speculatively.
- *(Conditional)* Hub-config flag — only if observed: "sticky circuit-breaker across restart for deployments that restart frequently for unrelated reasons".
