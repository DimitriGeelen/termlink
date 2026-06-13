# T-2211 — Substrate Concurrent-Drain Demo (arc-001 `demo_evidence`)

**Generated:** 2026-06-13T14:02:03Z
**Binary:** target/release/termlink (termlink 0.11.1293)
**Hub:** local (/var/lib/termlink)

## What this proves

arc-001 (`arc-parallel-substrate`) headline mechanic:
> *"Multiple agents execute disjoint work-units concurrently and merge cleanly
> without machine-state conflicts."*

`scripts/substrate-drain-demo.sh` spins up **N synthetic workers** that race to
drain an **M-unit work-queue** using ONLY the shipped substrate **claim primitive
(#1)**. It asserts the exclusive-delivery guarantee at the shell/operator level:
**every unit is won by exactly one worker** (disjoint union of per-worker win-sets
== the full unit set), with **zero double-claims**, under genuine contention
(dozens of `CLAIM_CONFLICT` rejections observed per run).

This is the operator-facing companion to two existing proofs:
- **Unit level:** `crates/termlink-session/tests/claim_client_integration.rs::concurrent_n_way_race_each_offset_won_exactly_once` (Rust, M offsets × N workers).
- **Single-unit composition:** `scripts/substrate-smoke.sh` (T-2151).

It composes only existing verbs — `channel create / post / claim / release /
claims-summary` — no new primitive, no hub change, no human-gated inception.

## How it works

1. Create (idempotent) a fixed bounded topic `substrate-drain-demo` (retention=messages); seed M work envelopes (offsets 0..M-1).
2. Launch N background workers. Each iterates the offsets in a **rotated order**
   (start = worker-id) so different workers collide on different units first →
   real contention. Each offset is attempted at most once per worker; on
   `CLAIM_CONFLICT` the worker moves on (never revisits).
3. Workers **hold** their claims through the race (no mid-race release), so a
   released-then-reclaimed slot can never inflate a unit's winner count — the
   assertion is purely about claim exclusivity under contention.
4. Assert: disjoint union == all M units (no double-claim, no gap). Exit 0/1.
5. Cleanup: release all held claims (`--ack`), remove temp dir.

## Captured runs (2026-06-13T14:02:03Z)

### Run 1 — 3 workers / 9 units
```
# substrate-drain-demo (T-2211) — proving the arc-001 headline mechanic
#   topic=substrate-drain-demo-1481552 workers=3 units=9 ttl_ms=60000
# seeded 9 units at offsets: 0 1 2 3 4 5 6 7 8
# distribution: worker-1=3 worker-2=4 worker-3=2
# total_wins=9 distinct_units_won=9/9 conflicts_observed=18
PASS: clean drain — 9 units, each won by exactly one of 3 workers, zero double-claims (conflicts under contention: 18)
```

### Run 2 — 5 workers / 15 units (stress)
```
# substrate-drain-demo (T-2211) — proving the arc-001 headline mechanic
#   topic=substrate-drain-demo-1483085 workers=5 units=15 ttl_ms=60000
# seeded 15 units at offsets: 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14
# distribution: worker-1=2 worker-2=3 worker-3=4 worker-4=3 worker-5=3
# total_wins=15 distinct_units_won=15/15 conflicts_observed=60
PASS: clean drain — 15 units, each won by exactly one of 5 workers, zero double-claims (conflicts under contention: 60)
```

### Run 3 — 4 workers / 12 units (--json)
```
{
  "ok": true,
  "topic": "substrate-drain-demo-1486453",
  "workers": 4,
  "units": 12,
  "total_wins": 12,
  "distinct_units_won": 12,
  "conflicts_observed": 36,
  "double_claims": "",
  "missing_units": "",
  "verdict": "clean-drain"
}
```

## Reading the evidence

- `distinct_units_won == units` and empty `double_claims` ⇒ **exclusive delivery held**.
- `conflicts_observed` (18–60 above) is the contention proof — workers genuinely
  raced for the same offsets and the hub serialized them via `CLAIM_CONFLICT`.
- Distribution across workers is roughly even because each worker yields after a
  win (simulated work). Skew is harmless — exclusivity, not fairness, is the
  guarantee under test. Load-balancing is the orchestrator's job
  (`scripts/orchestrator-backlog-drain.sh`, find-idle #2 + claim-transfer #3).

## Reproduce

```
TERMLINK_BIN=target/release/termlink scripts/substrate-drain-demo.sh --workers 4 --units 12
```

Exit 0 = clean drain; exit 1 = exclusive-delivery violated (double-claim or gap).
CI-safe: pure local hub, no network, deterministic assertion. Reuses one
retention-capped topic (hubs are append-only; no channel-delete verb) so runs
do not accumulate state. Use `--topic NAME` for isolated/parallel runs.
