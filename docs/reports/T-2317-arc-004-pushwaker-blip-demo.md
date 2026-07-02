# T-2317 — arc-004 push-waker (WP2): blip / reconnect demo evidence

**Arc:** arc-004 `push-transport` (WP2 of the T-2315 GO / Option A)
**Task:** T-2317 (build). Predecessors: T-2316 (WP1 push-waker), T-2315 (inception
GO), T-2314 (active reconnect the waker inherits).
**Date:** 2026-07-02

## What changed

WP1 (T-2316) proved the happy path — an inbox deposit rings the PTY sub-second via
push, a non-matching deposit is filtered. WP2 hardens that under a **WebSocket
drop**: because the waker's `channel subscribe inbox.queued --push` subprocess
inherits the T-2314 active reconnect, the waker survives a hub blip and keeps
ringing. No code change to the waker was needed — WP2 is the *wire evidence* that
the inherited reconnect actually holds end-to-end, plus the honest scope boundary.

## Evidence — `scripts/demo-pushwaker-blip.sh` (isolated hub + HOME, stub-inject)

The demo starts the waker, kills the hub mid-stream, restarts it on the same
runtime_dir + port (inside the reconnect window), then — after a settle — posts
**exactly one** fresh deposit to `inbox:<self>` and measures the rings.

```
=== arc-004 push-waker blip/reconnect demo (T-2317) ===
binary:            target/release/termlink
hub:               127.0.0.1:9197   (isolated, killed + restarted mid-run)
self inbox:        inbox:blip-72829   ring target pty: fakepty-72829
rings before post: 0
rings after post:  1   (INJECT inject fakepty-72829 /check-arc respond --enter)
rings after +5s:   1   (must equal rings-after-post — no double-wake)

RESULT: PASS — waker RESUMED after the hub blip (post-blip deposit rang the PTY),
        exactly once (no double-wake), and did not re-ring on catch-up overlap.
```

The load-bearing lines:
- **rings before post = 0 → rings after post = 1** — a deposit posted *after* the
  hub was killed and restarted rang the PTY. The waker could only deliver this by
  re-subscribing over the resumed WS: proof it **resumed** (did not permanently die
  or degrade on the drop) and the DM was **not lost**.
- **exactly one** ring for the single deposit — no double-wake.
- **rings after +5s = 1** (unchanged, no new post) — the CLI's catch-up poll
  re-delivering the same offset across the reconnect is collapsed by the waker's
  per-offset dedup: **no spurious re-ring**.

## Honest scope boundary (documented, not a bug)

A **prolonged** outage past the T-2314 reconnect cap (~6 fast failures ≈ 15 s of
backoff) degrades the waker's subprocess to a poll on `inbox.queued` — an
aggregator/ephemeral topic that does not deliver new deposits by durable cursor. At
that point the durable floor takes over: the receiver's own `/check-arc` cadence +
the sender's ring on the live rail. WS is a faster **trigger**, never the source of
truth. The blip demo restarts the hub immediately to exercise the WS-resume path;
the cap-degrade behaviour is the intended fallback, consistent with the arc's
durability constraint (T-2303 §8.4 / §10.3).

The test inbox receives **no pre-blip deposit**, so its post-blip offset is fresh —
the in-memory hub resets offsets on restart, and a pre-blip TTL dedup entry for the
same offset would otherwise mask the post-blip ring. The strict per-offset
exactly-once behaviour is separately locked by the WP1 unit test
(`scripts/test-pushwaker-filter.sh` → `pushwaker_dedup_ok`).

## Arc status after WP2

WP1 + WP2 close the T-2315 GO build surface: the shipped WS push is now
load-bearing for a live agent's wake (WP1) and proven robust across a socket drop
(WP2). Remaining follow-ons (noted in T-2316, not blocking arc close): a live-PTY
(non-stub) end-to-end proof, and a `dm:<self>:*` direct-push waker variant.
