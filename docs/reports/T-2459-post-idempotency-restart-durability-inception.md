# T-2459 — Post-idempotency exactly-once does not survive a hub restart (round-14 F1 inception)

**Status:** inception, GO recommended (decision = human's).
**Date:** 2026-07-22.
**Origin:** round-14 adversarial correctness hunt of TermLink's exactly-once / lifecycle guarantees.
**Related:** T-2049 (the dedupe primitive), T-2286 (`--await-ack`), T-2051 (offline queue).

## Summary

TermLink advertises **exactly-once** post delivery and the substrate ADR §5 leans on
it: `channel post --await-ack` "reuses dedupe + the receipt frontier … (T-2049 dedupe
→ exactly-once)." Round-14 review found the guarantee is really **"exactly-once within
one hub lifetime AND within the 5-minute dedupe TTL; at-least-once across a hub restart
or a longer blip."** The gap is a durability asymmetry: the *client* persists the
idempotency key durably and replays it, but the *hub's* recogniser for that key is
in-memory and dies on restart.

## The mechanism

| Side | Component | Durable? | Evidence |
|---|---|---|---|
| Client | offline queue row incl. `client_msg_id` | **yes** (SQLite) | `offline_queue.rs:4,44-48` |
| Hub | `(sender_id, client_msg_id)` dedupe LRU | **no** (in-memory) | `dedupe.rs:34,49,141` |

Dedupe TTL is 5 min (`dedupe.rs:41`, `DEFAULT_DEDUPE_TTL_MS = 300_000`); the map is a
process-global `OnceLock<Mutex<HashMap>>` with no disk backing.

**Trigger sequence (double-apply):**

1. Spoke posts message K with `client_msg_id = c`. Hub commits it at offset N and
   records dedupe entry `(sender, c) → N`. The TCP ack is lost (blip).
2. Either (a) the hub **restarts** — the dedupe map is wiped — or (b) **≥5 min elapse**
   so `(sender, c)` TTL-expires. Meanwhile the durable offline-queue row for K survives.
3. The queue flush replays K with the **same** `client_msg_id = c`. The hub's dedupe
   lookup misses (map wiped / entry expired) → `try_record_or_lookup` returns `Newly`
   (`dedupe.rs:126`) → `bus.post` appends the **same payload again** at offset N+k.
4. Every subscriber to the topic now sees K **twice**, at two offsets.

The `dedupe.rs` module header itself lists "hub bounce" among the *closed* scenarios —
it is **not** closed across a process restart, because the committed-entry cache dies
with the process while the bus-log offset it guarded survives durably.

## Why it matters (and why now)

The AEF orchestrator (arc-011) uses `--await-ack` specifically for the messages that
must not be lost or duplicated: worker **completion / ledger** reports. A double-applied
completion can double-count finished work or double-advance a ledger cursor. This is the
exact traffic class the exactly-once guarantee exists to protect, so an overstated
guarantee here is load-bearing, not cosmetic.

The likelihood is bounded: the offline queue flushes every ~5 s, so in steady state the
replay lands within TTL and within one hub lifetime — no double-apply. The gap requires
a restart (or a >5 min outage) to coincide with a lost ack and a still-pending queued
replay. Infrequent, but a "single supervised durable hub" (ADR §8) *does* restart, and
"restart is a recoverable pause" is exactly the design stance that makes at-least-once-
across-restart plausible-by-design — which is why the boundary must be *decided*, not
assumed.

## The decision (what the human owns)

**Where should restart-durable idempotency live?**

- **(a) Substrate-persist.** Persist the dedupe entries (or just the `--await-ack`
  subset) to `runtime_dir` SQLite so the recogniser survives restart. Restores true
  exactly-once transparently. Cost: a durable write per tracked post + a GC/retention
  policy. Must live in `runtime_dir` (PL-111), never `/tmp` (PL-021) — else the
  persistence itself evaporates on reboot.
- **(b) Consumer-idempotent.** Correct the contract to state the real guarantee
  (at-least-once across restart) and require the AEF completion-ledger to dedupe on
  `client_msg_id` at the application layer — idempotent-by-construction, zero hub cost,
  and aligned with "restart is a recoverable pause."

The fix is bounded either way; the choice depends on whether the AEF ledger is (or can
cheaply be) idempotent — a §9 collaboration-seam soft-dependency the substrate cannot
see alone. **Recommended middle path if (a):** persist only the `--await-ack` subset
(IW-3) — scope the durable-write cost to the messages that actually need exactly-once,
rather than making a 5-min-TTL cache permanent for every fire-and-forget post.

## Proposed direction on GO (design, not build)

1. Decide IW-1's boundary. If persist: design the await-ack-subset schema in
   `runtime_dir` (bounded retention, e.g. 24h).
2. One-bug-one-task build: EITHER a persistent await-ack dedupe store, OR a contract
   correction (ADR + `docs/operations/substrate-post-idempotency.md`) plus an
   AEF-consumer idempotency requirement.
3. A regression test that survives a **simulated hub restart**: post → drop the dedupe
   map → replay the same `client_msg_id` → assert a single append (persist path) or an
   explicitly-documented second append with a consumer-dedupe note (contract path).

## Verified adjacent-but-CLEAN this round (do not re-review)

- Claim lease / find-idle anti-join: clock-consistent (single `now_ms` for LIVE cutoff
  and active-claimer anti-join), verified in the round-14 hunt.
- Governor / cv_index counters: honest (increment only on the committed reject branch);
  only residual is a panic-only `connections_active` leak (captured separately, LOW).
- The find-idle hint/walk saturation disagreement (a *separate* round-14 finding) was
  **fixed** this round as **T-2458** (commit a8b7069d).

## Related

- T-2049 — the `client_msg_id` dedupe primitive this generalizes the failure mode of.
- T-2286 / T-2285 — `--await-ack`, the exactly-once-critical consumer.
- T-2051 — the durable offline queue whose persistence exposes the asymmetry.
- PL-111 — restart-durable hub state belongs in `runtime_dir`.
- PL-021 — volatile `/tmp` wipes `runtime_dir`; the persistence must target the
  persistent runtime_dir.
- arc-011 — the AEF orchestrator whose completion-ledger sets the severity (IW-2).
