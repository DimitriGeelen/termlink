# T-2050 Audit Report — T-1439 offline-queue flush-loop backoff parameters

**Task:** T-2050 (Substrate primitive #5 Gap B per T-2023 §4.B)
**Audit target:** `crates/termlink-session/src/bus_client.rs` —
the `BusClient::flush` drain pass + `connect_with_interval` tick loop.
**Date:** 2026-06-08

## Summary

The IW-3 disposition in T-2023 was "PARTIAL — `attempts` counter exists,
but full backoff/jitter/fail-permanent parameters need audit." Audit
result: **5 of 6 parameters are well-defined and load-bearing; 1
parameter (jitter) is missing and warrants a small follow-up
(≤30 LOC).** Recommend filing T-2055 for the jitter wire-in;
everything else is fine.

This is doc-only. No production code is changed by T-2050.

## Implementation map

The flush primitive has two halves:

| Half | Location | What it does |
|---|---|---|
| Tick loop | `connect_with_interval` lines 96-124 | Spawns a tokio task that wakes every `flush_interval`, calls `flush()`, repeats until shutdown. Default 5s. |
| Drain pass | `flush` lines 160-214 | Per-tick: walks the queue head-to-tail. For each row, attempts `channel.post`. On transport-fail → bump `report.failed`, break (preserve FIFO). On hub-reject → `bump_attempts`; if attempts crosses `POISON_THRESHOLD` (10), pop the row and continue. On success → pop, count. |

Both are in the same file; there is no other flush-style loop in
`termlink-session`. Grep confirms (`fn flush`, `flush_task`,
`flush_loop`, `fn drain` — only the BusClient site).

## Parameter table (T-2023 IW-3 disposition)

| Parameter | Value | Configurable | Score | Notes |
|---|---|---|---|---|
| Initial flush delay | `DEFAULT_FLUSH_INTERVAL = 5s` | YES — `connect_with_interval(.., flush_interval)` | ✓ ok | Tests use shorter intervals; production tunes at hub-client init. |
| Max delay | 5s (same as initial — no exponential backoff) | NO | ⚠ partial | Constant. Discussion below. |
| Jitter | NONE | NO | ❌ missing | Thundering-herd risk at fleet scale. Recommendation: file. |
| Per-row max attempts | `POISON_THRESHOLD = 10` | NO (const) | ✓ ok | Sensible default. Sub-50-LOC env override is trivial if ops asks; not blocking. |
| Dead-letter behaviour | Pop row, `tracing::warn!`, increment `report.dropped_poison` | Process-local only | ⚠ partial | Counter is in-memory per-BusClient; no external alert. Discussion below. |
| Transport-fail vs hub-reject | Distinguished (lines 167 / 206) | n/a | ✓ ok | Critical correctness — without this distinction, network blips would be treated as poison. |

## Why no exponential backoff is acceptable

The flush is a poll loop, not a per-row state machine. Each tick walks
the entire queue in FIFO order; a stuck head-of-queue row breaks the
loop AND is retried again on the next 5s tick. So the effective
behavior is:

- Transport down: queue accumulates. Every 5s a single retry of head.
  After hub returns, drains in one or two ticks.
- Hub up but row is poison: 5s × 10 retries = ~50s to surface
  `dropped_poison`. Sub-arc retries do not stack; one per tick.

Adding exponential backoff would require per-row scheduling state
(next-retry-at column in SQLite, sleep-by-row in flush). Real
production traffic from T-1439 onwards has not produced a single
incident traceable to "flush retried too aggressively." The 5s
poll-loop is good enough by behavioral evidence.

**Disposition:** DEFER — no follow-up. If real soak surfaces a hub
crushed by flush retries (would be visible in `rate_hits_total` after
T-2048 — RATE_LIMITED on `channel.post` from offline-queue retry
floods), reopen.

## Why jitter is the one real gap

The 5s tick interval is wall-clock anchored to `BusClient::connect`,
which fires at spoke startup. After a fleet-wide hub bounce, every
spoke that was up before the bounce reconnects within seconds, and
its flush task wakes at the SAME 5s offset modulo restart skew.

At 30-agent fleet scale, the cluster of 30 ticks every 5s is
manageable. At 300-agent scale (or with shared spoke processes that
each open multiple BusClients), the simultaneous flush pulses
hammer the hub. T-2048's `RATE_LIMITED` (-32008) will refuse
overflow, but that just bumps `attempts` and re-retries on the
next tick — same problem next pulse.

**Fix shape:** Add ±25% random jitter to the flush_interval, computed
per-tick:

```rust
// In connect_with_interval, before the sleep:
let jitter_ms = {
    use rand::Rng;
    let span = (flush_interval.as_millis() as f64 * 0.5) as i64;
    rand::thread_rng().gen_range(-span..=span)
};
let next_sleep = flush_interval
    .saturating_add(Duration::from_millis(jitter_ms.unsigned_abs() as u64))
    .saturating_sub(Duration::from_millis(if jitter_ms < 0 { (-jitter_ms) as u64 } else { 0 }));
tokio::time::sleep(next_sleep).await;
```

≤30 LOC including a unit test. `rand` is already a workspace dep
(via `termlink-session`).

**Filed as:** T-2055 (separate task per "one bug / one feature = one
task" rule).

## Why the dead-letter signal is doc-only

`dropped_poison` is a per-BusClient counter. It IS emitted via
`tracing::warn!` so a structured-log consumer can alert. The
absence of a cross-process counter (like T-2048's
`capacity_hits_total`) means an operator without log aggregation
can't easily ask "is the fleet dropping posts as poison?"

Two real fixes:

1. **Doc-only (chosen for T-2050):** mandate that ops grep
   `tracing::warn!` output for `dropping poison post after`. Already
   captured in `docs/operations/substrate-post-idempotency.md` indirectly;
   add explicit note in the new T-2051 offline-queue recipe doc.
2. **Structural (DEFER):** add a process-global
   `OFFLINE_QUEUE_POISON_TOTAL` counter, surfaced via a sibling
   `offline_queue.status` RPC. Worth it ONLY when fleet log
   aggregation isn't sufficient. Not load-bearing today; defer
   until an operator escalation requests it.

**Disposition:** doc-only. T-2051 will pick this up.

## Updated T-2023 IW-3 disposition

Inception said:

> **IW-3:** ⚠ Partial. `attempts` counter exists, but full
> backoff/jitter/fail-permanent parameters need audit. Confidence=2
> (audit will resolve to 4).

Post-audit:

> **IW-3:** ✓ Mostly-resolved. Backoff is constant-5s-poll-loop by
> design; max-attempts (10) + dead-letter (warn-log + pop) work
> correctly; jitter missing — one ≤30 LOC follow-up filed as T-2055.
> Confidence=4.

## Files referenced

- `crates/termlink-session/src/bus_client.rs` lines 49-54 (POISON_THRESHOLD),
  72 (DEFAULT_FLUSH_INTERVAL), 96-124 (connect_with_interval), 160-214 (flush).
- `crates/termlink-session/src/offline_queue.rs` — the queue itself (no
  scheduling logic; flush owns all of it).

## Related

- T-2023 inception report — Gap B framing
- T-2018 ADR §6 #5 — substrate primitive
- T-1439 — the implementation being audited
- T-2049 — Gap A (idempotency) — ships post-audit
- T-2051 — Gap C (operator recipe doc) — picks up dead-letter doc
- T-2055 — recommended follow-up: ±25% jitter (≤30 LOC)
