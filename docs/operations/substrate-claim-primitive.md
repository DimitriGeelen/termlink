# Substrate: Exclusive-Delivery Claim Primitive

> **For:** Developers building workers that pull work off a TermLink topic
> and need exclusive-delivery semantics ("only one worker processes each
> offset, no duplicates, no losses"). Part of the arc-parallel-substrate
> ADR (T-2018) — the first of 10 primitives that let multiple AEF agents
> coordinate parallel work on a shared host without machine-state conflicts.

## What it gives you

Three coordinated RPCs that turn a TermLink topic into a work queue with
exclusive-delivery semantics:

| RPC | What it does | Surfaces as |
|---|---|---|
| `channel.claim(topic, offset, claimer, ttl_ms)` | Reserve an offset for exclusive processing | CLI `termlink channel claim`, Rust `LeasedClaim::acquire`, JSON-RPC -32015 on conflict |
| `channel.renew(claim_id, claimer, additional_ttl_ms)` | Extend the lease while still working | CLI `termlink channel renew`, Rust `LeasedClaim` auto-renew |
| `channel.release(claim_id, claimer, ack)` | Consume the claim — ack=true advances cursor, ack=false reopens slot | CLI `termlink channel release`, Rust `LeasedClaim::{ack,nack}`, `Drop` |
| `channel.claims(topic, include_expired?)` | Read-only listing — answers "what is currently claimed?" without forcing a claim attempt | CLI `termlink channel claims`, Rust `channel_claims`, returns `Vec<ClaimSummary>` |
| `channel.claims_summary(topic)` | Read-only aggregate — answers "how busy is this topic, is anything stuck?" in one O(1) call (active vs expired counts, oldest-active age, next free slot) | CLI `termlink channel claims-summary` (+ `--watch <secs>` for continuous monitor mode, Slice 8), Rust `channel_claims_summary`, returns `ClaimsAggregate` |

The exclusive-delivery guarantee comes from a `UNIQUE(topic, offset)` SQL
constraint on the hub-side claims table. Two workers claiming the same
offset: one wins, the other gets `-32015 CLAIM_CONFLICT` with structured
data naming the conflicting `(topic, offset)`.

The "ack vs nack" distinction is the cursor-advance pivot:

- **ack=true** (work done correctly) — hub advances the claimer's persisted
  cursor past the offset via a `MAX-monotonic` UPDATE so the same offset
  is never returned again.
- **ack=false** (work returned for retry) — cursor unchanged; the slot
  reopens immediately for the next worker.

The whole flow is brand-name-neutral: TermLink ships the primitives; your
worker decides how to use them.

## The four lifecycle paths

```
                    ┌────────────────────────────────────────┐
                    │                                        │
                    ▼                                        │
        ┌────────────────────┐                              │
        │ channel.claim      │── -32015 CLAIM_CONFLICT ────►(another worker
        │ topic+offset+ttl   │                              │ already has it)
        └─────────┬──────────┘                              │
                  │ success: claim_id, claimed_until        │
                  ▼                                          │
        ┌────────────────────┐                              │
        │ working on the     │                              │
        │ offset's envelope  │                              │
        └──┬──────┬──────┬───┘                              │
           │      │      │                                  │
   need    │      │      │ done                             │
   more    │      │      │ correctly                        │
   time?   │      │      │                                  │
           ▼      │      ▼                                  │
  ┌──────────────┐│ ┌──────────────────┐                    │
  │ channel.renew││ │ channel.release  │                    │
  │ refresh lease││ │ ack=true         │── cursor advances ─┤
  └──────────────┘│ └──────────────────┘                    │
                  │                                          │
                  │ failed / wrong worker /                  │
                  │ panicked (Drop)                          │
                  ▼                                          │
        ┌────────────────────┐                              │
        │ channel.release    │                              │
        │ ack=false          │── slot reopens, same offset ─┘
        └────────────────────┘   returned to next claimant
```

The four exit paths from a claim:
1. **Renew, then release(ack=true)** — happy path; long-running work that needs lease extension.
2. **Release(ack=true)** — fast happy path; work completes within initial TTL.
3. **Release(ack=false)** — explicit return-for-retry (operator nack'd it, or worker self-aborted).
4. **Drop / crash / process kill** — implicit return-for-retry. Either:
   - The Rust `LeasedClaim::Drop` fires fire-and-forget `release(ack=false)` (fast slot reopen).
   - Or the lease lapses past `claimed_until` and the next claimant lazy-evicts the stale row (slower).

## Picking a TTL

Pick `ttl_ms` to answer **"how long can this worker plausibly take to
either ack the work, nack it, or call renew?"**

| Worker shape | Suggested TTL | Why |
|---|---|---|
| Sub-second processor (DB write, queue forward) | 5_000 (5s) | Short-enough that crashed workers don't park slots; long enough to absorb network blips |
| Multi-second processor (LLM call, HTTP fan-out) | 30_000 (30s — the default) | The Rust `LeasedClaim` auto-renews at TTL/2 = 15s, so even an 8-minute job stays alive without operator intervention |
| Long-running batch (data migration, full re-index) | 300_000 (5min) | Manual renew checkpoints every 2-3 minutes; tolerates short pauses without lease-renew chatter |

Hub-side hard cap: **1 hour** (`60 * 60 * 1000` ms). Larger values clamp
silently — this is a safety against bugs (a worker that never renews and
never releases shouldn't be able to park a slot for a day).

## See it work in 60 seconds

Before reading anything else, run the demo. It spins up N tokio worker
tasks racing to claim sequential offsets on a topic — successful claims
"process" (sleep 100ms) then ack; conflicting claims (CLAIM_CONFLICT)
count and skip. The end-of-run summary proves exclusive-delivery.

```
# 1. start a local hub (if not already)
termlink hub start &

# 2. create a topic and add 20 posts so workers have offsets to claim
termlink channel create demo-work
for i in $(seq 1 20); do termlink channel post demo-work "item-$i"; done

# 3. fire 4 workers competing for 20 offsets
cargo run --release --example parallel_worker -- \
    /tmp/termlink-0/hub.sock demo-work 4 20
```

You'll see lines like `[worker-1] won claim on offset=3` interleaved
with `[worker-2] conflict on offset=3 (already claimed) — skipping`,
then a final summary showing `total_wins=20  total_conflicts=N`. Every
offset is processed exactly once.

The example source — under 200 lines — is at
`crates/termlink-session/examples/parallel_worker.rs`. Copy it as the
starting point for your own worker; everything in this runbook below
is the "why" for what the example does.

## Quick CLI tour

The three verbs are operator-callable end-to-end. Useful for diagnostics
("is this offset stuck on a dead worker?") or as wire-format demos.

### Claim an offset

```
$ termlink channel claim my-work-queue 42 --claimer operator-shell-1 --ttl-ms 60000
claim_id:      clm-1700000000000000000-my-work-queue-42
topic:         my-work-queue
offset:        42
claimer:       operator-shell-1
claimed_at:    1717800000000
claimed_until: 1717800060000
lease_ms:      60000
```

Save the `claim_id` — you need it to renew or release.

### Renew before it lapses

```
$ termlink channel renew --claim-id clm-... --claimer operator-shell-1 --additional-ttl-ms 60000
claim_id:      clm-1700000000000000000-my-work-queue-42
claimed_until: 1717800120000     # ← advanced by 60s
lease_ms:      60000
```

Refuses with **-32017 CLAIM_NOT_OWNED** if `--claimer` doesn't match the
original. Refuses with **-32018 CLAIM_EXPIRED** if the lease has already
lapsed (lazy-evicted by a competing claim attempt).

### Release with ack (work done correctly)

```
$ termlink channel release --claim-id clm-... --claimer operator-shell-1 --ack
claim_id: clm-1700000000000000000-my-work-queue-42
topic:    my-work-queue
offset:   42
ack:      true
```

The claimer's persisted cursor on `my-work-queue` advances past offset 42.
Subsequent `channel.subscribe` calls won't return it again.

### Release with nack (return for retry)

```
$ termlink channel release --claim-id clm-... --claimer operator-shell-1
# (no --ack)
```

Slot reopens immediately; cursor unchanged. The next worker calling
`channel.claim my-work-queue 42` succeeds.

### List live claims on a topic

```
$ termlink channel claims my-work-queue
  offset  claimer               claim_id                  remain_ms  state
       3  worker-A              clm-1717-my-work-queue-3      24317  active
      42  operator-shell-1      clm-1718-my-work-queue-…      54017  active
(2 row(s))
```

Read-only — does not attempt a claim, does not mutate any state.
Useful for answering "what is this topic doing right now?" mid-incident
without consuming an error. Pass `--include-expired` to also surface
rows whose `claimed_until` has lapsed (operator forensics —
"who held the offset before it expired?"). Pass `--json` for the
structured envelope.

### Aggregate claim state on a topic (Slice 6)

```
$ termlink channel claims-summary my-work-queue
topic "my-work-queue": active=12 expired=3 oldest_active_age=18402 next_expiry_ms=1730481923000
```

Same read-only contract as `channel claims` but **O(1)** at the hub
(single SQL aggregate over `idx_claims_topic_until`) — safe to call on
hot paths or from monitoring cron.

Three operator signals in one line:

- **`active` / `expired` counts** — load shape. A topic with `active=N`
  workers running near steady-state should show low `expired` (lazy-evicted
  on next claim attempt). A growing `expired` count with low `active`
  means workers have been dying without releasing — investigate.
- **`oldest_active_age`** — how long the longest-held lease has been
  outstanding. Compare to the worker's configured `ttl_ms`: if it's
  approaching TTL, the worker is either stuck or about to renew.
- **`next_expiry_ms`** — wall-clock when the next slot frees up without
  operator intervention. Useful for "when can I retry this offset?"

When `active_count == 0`, all three `*_ms` fields are `null`. Pass
`--json` for the structured envelope.

**Stuck-worker pattern.** Run `claims-summary` from cron every minute on
hot topics. A topic whose `oldest_active_age` keeps growing past TTL
while `active_count` stays pinned is a leaked lease — usually a worker
that panicked outside Drop's reach (e.g. an OS-level kill). Use
`channel claims` to identify the specific stuck claim, then
`channel release --ack=false` to reopen the slot.

**Live watch mode (Slice 8).** During incident triage, leaving a watch
loop running on a side terminal beats waiting for the next cron tick:

```
$ termlink channel claims-summary my-work-queue --watch 10
# channel claims-summary --watch | topic="my-work-queue" | interval=10s | 2026-06-08T00:30:14Z
topic "my-work-queue": active=12 expired=0 oldest_active_age=4203ms next_expiry_ms=1730482218000
```

The frame re-renders every N seconds (clamped to 5..=3600 — sub-5s
hammering the hub for stuck-worker detection is pointless) with a
fresh aggregate. Per-tick fetch errors don't kill the loop — they print
`# fetch error (will retry on next tick): <e>` and the next tick
retries. `--watch` is incompatible with `--json` (streaming text vs
one-shot envelope) — pick one. SIGINT exits cleanly.

**Fleet-wide sweep (Slice 9).** When the operator does not know which
topic to check — typical incident triage cold-start — `--all` sweeps
every topic on the hub in one shot. Topics with `expired_count > 0` OR
`oldest_active_age > 60s` get a `[POTENTIALLY STUCK]` annotation so the
list is visually scannable:

```
$ termlink channel claims-summary --all
topic "broadcast:global": no claims (clean)
topic "work-q1": active=1 expired=0 oldest_active_age=15707ms next_expiry_ms=1780873154463
topic "work-q2": active=0 expired=1 oldest_active_age=- next_expiry_ms=-  [POTENTIALLY STUCK]
topic "work-q3": no claims (clean)
(4 topic(s), 1 with potentially stuck claims)
```

Composes with `--watch` for a continuous fleet-wide stuck-worker
dashboard (`claims-summary --all --watch 30`) and with `--json` for a
machine-readable envelope `{ok, topic_count, stuck_count, topics: [...]}`
where each entry carries a `potentially_stuck: bool` flag. Per-topic
fetch errors during the sweep are non-fatal — printed inline (text
mode) or surfaced as `{ok: false, error: ...}` array entries (JSON
mode), and the iteration continues.

The `topic` positional and `--all` flag are mutually exclusive —
exactly one is required. Run `claims-summary --all` cold; once a
suspicious topic is identified, drill in with
`channel claims <topic>` for the per-claim breakdown.

**Stuck-worker intervention (Slice 11).** Detection (Slices 8 + 9)
surfaces the stuck claim; ordinary `channel release` refuses to clear
it because the operator is not the original claimer
(`CLAIM_NOT_OWNED` -32017). The intervention verb is
`channel claim-force-release`, which bypasses the ownership check —
semantics match `release(ack=false)`, so the cursor stays put and the
slot reopens for the next worker to retry the work:

```
$ termlink channel claims work-q2
   offset  claimer               claim_id                   remain_ms  state
        5  worker-7-pid-9919     74b3a8f1-c1d2-46e2-...     -2147483    EXPIRED
$ termlink channel claim-force-release --claim-id 74b3a8f1-c1d2-46e2-... --reason "worker-7 host rebooted"
claim_id:      74b3a8f1-c1d2-46e2-...
topic:         work-q2
offset:        5
forced_from:   worker-7-pid-9919
forced_reason: worker-7 host rebooted
(slot freed for next worker; cursor not advanced)
```

The complete operations loop is now `detect → diagnose → intervene`:

| Step | Verb | What it answers |
|------|------|-----------------|
| Detect | `channel claims-summary --watch <secs>` (Slice 8) or `--all --watch <secs>` (Slice 9 + 8) | "Is anything stuck right now?" |
| Diagnose | `channel claims <topic>` (Slice 4) | "Which claim_id is stuck and who owns it?" |
| Intervene | `channel claim-force-release --claim-id <id> --reason "..."` (Slice 11) | "Clear it now without waiting for TTL expiry." |

`--reason` is optional but encouraged — it is echoed in the response
under `forced_reason` for downstream audit-log forwarding (e.g. emit
to a `<topic>:claim-events` topic for retrospective via the standard
`channel subscribe` pattern). The forced original claimer is always
echoed in `forced_from`.

**Authorization scope.** The hub today trusts any authenticated caller
equally — there is no per-user authorization model. Per ADR §6 #6
(symmetric authentication across transports), authentication is
transport-level (UID-trust UDS for same-host, HMAC + cert pinning for
cross-host); `claim-force-release` is consistent with this model
(anyone who can reach the hub can break any claim). For a future
multi-tenant scenario this asymmetry would need addressing alongside
T-2024; tracked separately under G-064. For ring20 homelab /
single-operator-per-hub usage this is the deliberate trade.

## Worker pattern (Rust)

The `termlink-session` crate exports `LeasedClaim`, which wraps a claim
with auto-renew + Drop-fires-nack semantics. Use this for any worker
that needs the primitive — it handles the lease lifecycle for you.

```rust
use termlink_session::{LeasedClaim, ClaimError};
use termlink_protocol::transport::TransportAddr;

async fn process_offset(
    addr: TransportAddr,
    topic: &str,
    offset: u64,
    worker_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Acquire — fails fast on conflict.
    let lease = match LeasedClaim::acquire(addr, topic, offset, worker_id, 30_000).await {
        Ok(l) => l,
        Err(ClaimError::Conflict { .. }) => {
            // Someone else got there first. Skip.
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    // 2. Do the work. The lease auto-renews every 15s in the background —
    //    even a 5-minute job stays alive.
    let result = expensive_operation().await;

    // 3. Consume.
    match result {
        Ok(_) => {
            // Cursor advances; work won't be redelivered.
            lease.ack().await?;
        }
        Err(_transient) => {
            // Cursor unchanged; another worker (or this one on retry) gets it.
            lease.nack().await?;
        }
    }

    // (If `lease` falls out of scope without ack/nack — e.g. panic — its
    //  Drop fires fire-and-forget release(ack=false), so the slot reopens
    //  fast without waiting for the TTL to lapse.)
    Ok(())
}
```

### Common worker loop

```rust
async fn worker_loop(
    addr: TransportAddr,
    topic: &str,
    worker_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cursor: u64 = 0;
    loop {
        // Subscribe gives you offsets to attempt. Cursor advancement
        // happens via channel.release(ack=true), not subscribe.
        let envelopes = subscribe_since(&addr, topic, cursor).await?;
        for env in envelopes {
            if let Err(_skipped) = process_offset(addr.clone(), topic, env.offset, worker_id).await {
                // Log; don't abort the loop.
            }
            cursor = env.offset + 1;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
```

The shape mirrors a Kafka consumer group with explicit acknowledgement —
but cheaper to operate (no broker cluster, just the existing TermLink hub)
and embedded in the same wire protocol as everything else on the bus.

## Diagnostic recipes

### "Is this offset stuck on a dead worker?"

```
# 1. Try to claim it yourself.
termlink channel claim <topic> <offset> --claimer diagnostic-probe-1 --ttl-ms 5000
```

- If you get **`-32015 CLAIM_CONFLICT`**, another worker holds it — that's working as intended.
- If you succeed instantly, the slot is open — either no worker is processing this topic, or a stale claim was just lazy-evicted.

### "How long has this offset been held?"

The `claim_id` is the public handle. Operators can renew it as a probe:

```
termlink channel renew --claim-id <id> --claimer <original_claimer> --additional-ttl-ms 1
# Returns claimed_until; subtract claimed_at to see total elapsed.
```

### "Force-release a stuck claim"

There's no force-release verb by design — operator override would break
the ownership invariant that workers depend on. If a worker is truly
dead, wait for `claimed_until` to lapse and the next claimant's lazy
evict will clear it. For sub-minute leases this is typically <60s.

If you genuinely need an immediate override (e.g. operator-paged
incident), use the original `claimer` value with `channel release` — the
hub trusts ownership-by-claimer-string, not network-source.

## Error code reference

| Code   | Variant       | When                                                 |
|--------|---------------|------------------------------------------------------|
| -32015 | `CLAIM_CONFLICT`   | Another worker holds this offset (data: topic, offset)            |
| -32016 | `CLAIM_NOT_FOUND`  | claim_id never existed, was released, or lazy-evicted (data: claim_id) |
| -32017 | `CLAIM_NOT_OWNED`  | The caller's `claimer` differs from the original (data: claim_id) |
| -32018 | `CLAIM_EXPIRED`    | `claimed_until` has lapsed — renew refused (data: claim_id)       |

The Rust client surfaces each as a typed `ClaimError` variant. CLI
verbs surface them as `anyhow!` errors on stderr with non-zero exit.

## What's NOT in this primitive (intentionally)

- **Work distribution.** Claim is pull-based: workers decide what to claim. There is no orchestrator-side push or fair-share scheduler. (See T-2021 substrate inception for pull/assign verb.)
- **Worker discovery.** Hub doesn't track which workers are alive. A worker simply claims; if it dies, lazy-evict cleans up. (See T-2020 substrate inception for hub-owned idle/busy registry.)
- **Cross-hub work-stealing.** Claims are per-hub. Cross-hub coordination requires application-level federation. (Same caveat as G-060: TermLink hubs maintain independent state.)
- **Reserved-but-deferred semantics.** You can't reserve a claim "from offset N onwards" — claims are per-exact-offset. Work-list-style claims are an application-level concern.

These are intentional scope cuts to keep the primitive small and orthogonal. The other 9 primitives in the §6 manifest (T-2020..T-2028) address adjacent concerns.

## References

- **ADR:** `docs/architecture/parallel-execution-substrate.md` §4.2 (lease-with-renewal + lazy expiry) + §6 manifest first primitive.
- **Inception:** T-2019 (GO decision).
- **Slice 1 (T-2029):** claims table + `channel.claim` / `channel.release` RPCs.
- **Slice 2 (T-2030):** `channel.renew` + `CLAIM_EXPIRED` lazy-evict path.
- **Slice 3 (T-2031):** Rust `claim_client.rs` + `LeasedClaim` RAII helper.
- **CLI (T-2032):** `termlink channel claim/release/renew` verbs.
- **MCP parity (T-2033):** `termlink_channel_{claim,release,renew}` tools for AI agents.
- **Runnable example (T-2034):** `crates/termlink-session/examples/parallel_worker.rs` — copy-pasteable starter for parallel workers.
- **Slice 4 (T-2037):** `channel.claims` read-only listing RPC + CLI verb — answers "what's currently claimed?" without consuming an error.
- **Slice 5 (T-2038):** `termlink_channel_claims` MCP tool — read-only listing surface for AI agents.
- **Slice 6 (T-2039):** `channel.claims_summary` aggregate RPC + Rust client + CLI verb — answers "how busy / is anything stuck?" in one O(1) call. Operator signal for stuck-worker / load-pattern detection.
- **Slice 7 (T-2040):** `termlink_channel_claims_summary` MCP tool — agent-callable companion for AI investigators to query topic load + stuck-worker state without shelling out.
- **Slice 8 (T-2041):** `channel claims-summary --watch <secs>` continuous-monitor CLI mode — re-runs the aggregate every N seconds (clamped 5..=3600), clears the screen between frames, tolerates per-tick fetch errors. Hands-off form of the cron stuck-worker recipe; ideal for incident triage side terminals.
- **Slice 9 (T-2042):** `channel claims-summary --all` fleet-wide sweep — queries `channel.list` and per-topic calls `channel.claims_summary`, annotates `[POTENTIALLY STUCK]` on topics with `expired_count > 0` OR `oldest_active_age_ms > 60_000`, footer reports total + stuck counts. Composes with `--watch` (live fleet dashboard) and `--json` (`{ok, topic_count, stuck_count, topics: [...]}` envelope). Per-topic fetch errors during the sweep are non-fatal.
- **Slice 10 (T-2043):** `termlink_channel_claims_summary_all` MCP tool — symmetric closure of the fleet-wide sweep for AI investigator agents. Same envelope shape as Slice 9 (`{ok, topic_count, stuck_count, topics: [...]}` with `potentially_stuck: bool` per topic). Read-only; no auth, no network beyond hub UDS. The cold-start verb when an agent must answer "which topic has the stuck worker?" without shelling out.
- **Slice 11 (T-2044):** `channel claim-force-release` + `termlink_channel_claim_force_release` — operator-Tier-0 intervention verb that bypasses `claimed_by == claimer` ownership check. Closes the operations loop from observability (Slices 8/9/10) to intervention: detection → diagnosis → force-release. Semantics match `release(ack=false)`; cursor untouched, slot freed. Returns `{forced_from, forced_reason}` audit anchors. Single-operator-per-hub trust model documented under G-064.
