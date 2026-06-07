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
- **MCP parity (T-2033):** TBD — `termlink_channel_{claim,release,renew}` tools for AI agents.
