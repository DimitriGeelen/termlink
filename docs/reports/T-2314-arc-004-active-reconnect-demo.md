# T-2314 — arc-004 active reconnect-to-WS: demo evidence

**Arc:** arc-004 `push-transport` (follow-on, T-2311 GO / Option B)
**Task:** T-2314 (build). Predecessors: T-2309 (S3b live consumer), T-2310 (TCP demo),
T-2313 (WS-over-Unix), T-2311 (inception GO).
**Date:** 2026-07-02

## What changed

Before T-2314, `channel subscribe <topic> --push` tried the WebSocket **once** and,
on any drop, degraded permanently to the 1 s poll floor until the process restarted.
A long-lived agent that saw one transient blip ran ~10× slower forever.

T-2314 implements **Option B** (from `T-2311-arc-004-active-reconnect-inception.md`):
the `--push` branch now wraps the WS attempt in a bounded reconnect loop —

```
loop {
    run_ws_push(...)                       // fast live path
    ws_poll_catchup(cursor)                // RB2: drain the gap from the DURABLE cursor,
                                           //      advance it (no missed events)
    attempt = if healthy_session { 0 }     // long stable session resets the backoff
              else { attempt + 1 }
    if attempt > CAP { break to steady poll }   // bounded — never a tight spin
    sleep(ws_reconnect_backoff(attempt))   // exp base×2^(n-1), 8s cap, +25% jitter
}
```

- `ws_reconnect_backoff` is a pure, unit-tested fn (growth / cap / jitter-bounds /
  clamp): `cargo test -p termlink-cli ws_reconnect_backoff` → **1 passed**.
- The durable poll path stays authoritative; the WS is only a faster transport.
- Unix hubs (T-2313) reconnect the same way over the raw socket.

## Wire evidence

`scripts/demo-ws-push.sh` (isolated TCP hub, temp runtime_dir + HOME) was extended
with a **blip → restart → reconnect** segment. Live run on the fresh release binary:

```
=== arc-004 WS push + active-reconnect demo (T-2310 / T-2314) ===
1st post->push:    88 ms  (frame: [push] inbox.queued seq=0: {…"message_offset":0…})
blip notice:       [push] WS unavailable (WebSocket stream error: … peer closed
                   connection …) — catching up then reconnecting
reconnect notice:  reconnected — back on push
post-blip delivery: DM "after-blip-…" delivered after blip
RESULT: PASS — sub-second push (88 ms), reconnect loop engaged on drop,
        and live push RESUMED after the hub blip (no permanent degrade).
```

The load-bearing line is **post-blip delivery**: a DM posted *after* the hub was
killed and restarted is delivered to the still-running consumer over the *resumed*
live WS — proving it did not permanently degrade to poll. Re-run confirmed the same
result (90 ms) with clean report formatting.

### No regression on the sibling paths

- **Unix push (T-2313):** `scripts/demo-ws-push-unix.sh` → **PASS, 31 ms** over the
  raw Unix socket (no TLS / no token) — unchanged.
- **Session WS unit tests:** `cargo test -p termlink-session ws_` → **10 passed**.
- **Release build:** `cargo build --release -p termlink` → exit 0.

## Scope notes

- On the isolated **in-memory** hub, a restart resets topic offsets, so the demo
  re-posts the post-blip DM on a ~1 s cadence to guarantee one lands after the WS
  re-subscribes. A persistent production hub does not reset offsets; the catch-up
  pass drains the gap directly.
- Brief WS↔catch-up **overlap** on the first drop is the documented acceptable
  degrade ("correctness over dedup", arc S3 comment). The cursor advances so gap
  events are not re-drained on every subsequent reconnect cycle, and the eventual
  steady poll (after the cap) does not re-emit them.
