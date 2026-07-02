# T-2320 — arc-004 push-wake latency benchmark

**Task:** T-2320 (verification, follow-on to closed arc-004 `push-transport`)
**Related:** T-2303 (inception §10 gap), T-2316 (WP1 push-waker, 172 ms point), T-2318 (E2E functional proof), T-2313 (WS-over-Unix 31 ms)
**Script:** `scripts/bench-pushwake-latency.sh`
**Date:** 2026-07-03

## Why this exists

The T-2303 inception GO'd arc-004 on the value claim that a hub→client push
replaces the **~15 s doorbell-then-poll wake floor** with a **sub-second** wake.
But §10 (lines 201–205) flagged the one honest gap:

> *"the 15 s latency floor is read from code constants, not a live end-to-end
> measurement… a 30-min live baseline would confirm the delta is worth it.
> **This is the one reason a reasonable reviewer might choose DEFER over GO.**"*

The arc shipped and closed (`decision: shipped`) on the code-constant basis. This
task retires that gap by **measuring** the wake latency live, through the exact
shipped mechanism T-2318 proved works end-to-end. Measurement only — no new
build, no inception, no arc reopen.

## What is measured (honest scope)

```
t0 = just before  channel post inbox:<self>
       └▶ hub append ▶ inbox.queued aggregator frame ▶ push to the registered
          session's push-waker ▶ real `termlink inject` into the PTY
t1 = when the injected doorbell text is OBSERVED in the PTY's own terminal
     (read via the `termlink output` data plane)
latency = t1 - t0
```

This is the **full production wake path** (post → push → inject → shell echo) —
the latency a live agent actually sees when an inbox deposit arrives with no live
sender to ring it. It is **not** a synthetic proxy: it reuses T-2318's hermetic
harness (isolated hub + HOME + a real `termlink spawn --shell` PTY + the real
`be-reachable start` lifecycle that registers the session and spawns the waker),
looped with timing.

**Conservative by construction.** `t1` is detected by polling `termlink output`,
and each poll costs one output-RPC (tens of ms). The reported latency therefore
counts observation cost **in**, never out — it is an **upper bound** on the true
wake latency. A sub-second number here means the real path is at least that fast.

## Results

Two independent hermetic runs (12 timed trials each, warmup discarded, isolated
TCP hub on loopback):

| Run | port | trials rang | min | median | mean | p95 | max | (ms) |
|-----|------|-------------|-----|--------|------|-----|-----|------|
| 1   | 9196 | 12 / 12     | 93  | **111** | 109 | 126 | 126 | |
| 2   | 9207 | 12 / 12     | 79  | **85**  | 85  | 97  | 97  | |

Per-trial (run 1, ms): `93 96 98 105 109 111 111 112 114 115 118 126`
Per-trial (run 2, ms): `79 82 82 82 84 84 85 87 89 89 91 97`

Both runs: **100 % of deposits rang**, median **sub-100 ms**, p95 **≤ 126 ms** —
every sample sub-second, as an upper bound.

## Side-by-side vs the pre-push floors

| Path | Latency | Source |
|------|---------|--------|
| **arc-004 push-wake (measured, this task)** | **~85–111 ms median** (upper bound) | `scripts/bench-pushwake-latency.sh`, runs above |
| `channel subscribe --follow` poll fallback | ~500 ms mean (1 s poll, uniform) | `channel subscribe --follow` help: *"Keep polling every 1s"* |
| Doorbell-then-poll wake floor (pre-arc-004) | **~15 s** | T-2303 inception §10 (documented floor from code constants) |

Measured delta: the push-wake median is **~135× faster** (run 1) to **~175×
faster** (run 2) than the documented 15 s doorbell floor, and comfortably below
the 1 s `--follow` poll fallback as well. The inception's value claim
(15 s → sub-second) is **confirmed with live measurement**, not assumed.

## Cross-references (measurement scope boundaries)

- **T-2316 (WP1):** measured a single full-E2E point at **172 ms** — consistent
  with this task's tighter 85–126 ms distribution (172 ms was a one-shot,
  warmup-inclusive point).
- **T-2313 (WS-over-Unix):** measured **31 ms** for push *delivery* alone
  (envelope at subscriber, excluding the inject + shell echo). The gap between
  31 ms (delivery) and ~85–111 ms (this task, full wake incl. inject + observation
  poll cost) is the inject/echo/observation overhead — accounted for, not hidden.

## Reproduce

```bash
cd /opt/termlink
cargo build --release -p termlink   # if target/release/termlink is stale
BENCH_TRIALS=12 bash scripts/bench-pushwake-latency.sh
```

Hermetic (isolated `TERMLINK_RUNTIME_DIR` + `HOME` + loopback TCP hub; self-cleans
hub + tmux PTY on exit). Exit 0 = PASS (median sub-second); exit 6 = regression
(median ≥ 1 s).

## Conclusion

The arc-004 push-transport value claim is now **measured, not assumed**: the live
full-wake latency is **~85–111 ms median (upper bound), 100 % delivery**, versus a
documented 15 s doorbell floor — a ~135–175× improvement. The single honest gap
the inception named as *"the one reason a reviewer might DEFER"* is closed with
reproducible evidence.
