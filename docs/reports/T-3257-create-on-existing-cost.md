# T-3257 — Hub cost of `channel.create` on an existing topic (C-34)

**Decision: NO BUILD.** `channel.create` on a topic that already exists costs the same
as a read RPC, within noise. A client-side "known topic" cache would save one cheap
round trip per `--ensure-topic` post. It would also add a new cache-invalidation
surface (a topic deleted or swept after it was cached), so there is nothing here to
justify building it. Per the T-3011 GO ("add a client-side cache ONLY if non-trivial"),
this closes C-34 with the measurement.

## Why the ratio looked alarming
C-34 observed `channel.create` ≈ `channel.post` (52,695 vs 53,538; ratio 0.98) against
only 45 topics. The CLI's `ensure_topic()` (`crates/termlink-cli/src/commands/channel.rs`)
issues `channel.create` before each post that asks for it, and the hub answers
`created: false` for an existing topic. So the ratio counts ensure-topic calls, not
topic churn. The question was only whether those calls cost anything.

## Measurement (2026-09-29, termlink 0.12.13)
Isolated hub: temp `TERMLINK_RUNTIME_DIR`, `hub start --tcp 127.0.0.1:19877`; no shared
hub was touched. Topic `bench` was created once, then 60 sequential CLI calls were made
per verb. Mean wall time per call includes client process start, TLS and auth, which are
identical across verbs, so the **differences** are the hub-side cost.

| verb | run 1 | run 2 |
|---|---|---|
| `channel info bench --json` (read baseline) | 13.0 ms | 13.5 ms |
| `channel create bench` (topic exists) | 15.5 ms | 13.5 ms |
| `channel post bench …` (append) | 30.7 ms | — |

Sanity: after the run `channel info` reported `count: 60`, so the posts landed in the
isolated hub and the harness measured the hub it meant to. The hub was then stopped and
its temp dir removed.

**Reading:** create-on-existing is 0–2.5 ms over a read, within run-to-run noise (the
second pass is equal to the baseline). A post, a real durable append, is about 2× either.
The hub-side cost of the "redundant" creates is negligible.

## Limits
- Single host, sequential calls, idle isolated hub. Contention on a busy shared hub was
  not measured. Since create-on-existing did no more work than a read here, contention
  would affect both alike.
- Measured through the CLI (process-per-call). An in-process client would make the
  absolute numbers smaller, not change their ratio.
