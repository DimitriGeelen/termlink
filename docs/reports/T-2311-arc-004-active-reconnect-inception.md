# T-2311 — arc-004 follow-on inception: active reconnect-to-WS with backoff

**Type:** inception (one question, go/no-go)
**Arc:** arc-004 `push-transport` (follow-on, past the GO(scoped) surface)
**Agent recommendation:** **GO** (build as one bounded slice, Option B). Final decision: human.
**Date:** 2026-07-02

---

## The one question

Today, `channel subscribe <topic> --push` (S3b / T-2309) tries the WebSocket path
**once**; on any drop it degrades to the poll loop and **stays on poll until the
process restarts**. Should `--push` instead **retry the WebSocket after a drop**
(with backoff), so a long-lived agent returns to the sub-second path after a
transient blip — or is permanent degrade-to-poll the right v1?

## Why this matters (problem)

- `--push` exists for **long-lived live agents** — the sessions reachable for hours.
  Those are exactly the ones most likely to see a transient blip (hub restart, NAT
  rebind, brief partition).
- One blip today permanently downgrades such an agent from sub-second push
  (~90 ms measured, T-2310) to the **1 s poll floor** — ~10× — with no recovery
  short of a process restart the agent may never do.
- The failure is **silent to the agent**: a single stderr degrade line, then it
  quietly runs slow forever.

## What is NOT the problem (correctness is already safe)

From reading `crates/termlink-cli/src/commands/channel.rs` (the `--push` branch at
~8488 and the poll loop below it):

- On WS end/error, control falls through to the **existing poll loop**, which reads
  from the **durable `cursor`**. The poll loop is the authoritative floor and
  **never misses** events — the WS is a faster transport layered on top, never a
  source of truth (the arc's core invariant).
- So this is a **latency-robustness** gap, not a data-loss gap. Any reconnect
  design must **preserve** that no-miss guarantee.

## The subtlety a reconnect design must respect

The hub aggregator is **live-only** — a freshly (re)subscribed WS stream starts from
"now" and does **not** replay events posted during the gap. A naive "just reconnect
the WS" loop would **miss** every event posted while the socket was down. The
correct shape must drain the gap from the durable cursor on each reconnect.

## Options considered

### Option A — concurrent poll-floor + WS accelerator
Poll loop runs continuously as the durable floor; WS push runs concurrently as an
accelerator, deduped by offset; reconnect WS on drop with backoff.
- **Pro:** best UX, no gap, WS purely additive.
- **Con:** largest change (concurrent tasks + cross-stream dedup, two renderers of
  the same events, double-print surface). Heavier than v1 warrants.

### Option B — alternating reconnect loop with poll catch-up  ✅ recommended
```
loop {
    match run_ws_push(...) {           // fast path
        Unsupported => break to poll,  // Unix hub — existing behavior
        Ended | Err => {
            drain_poll_once(cursor);   // ONE poll pass from cursor -> no missed events
            sleep(backoff);            // exp backoff, capped, jittered
            if attempts > cap { break to steady poll }  // settle, don't spin
            continue;                  // retry WS
        }
    }
}
```
- **Pro:** bounded — reuses `run_ws_push` + existing poll logic; preserves no-miss
  (catch-up drains the gap from the durable cursor); restores sub-second after a
  blip; backoff window is exactly the acceptable degraded state.
- **Con:** brief poll-latency windows during reconnect (acceptable); must not
  tight-spin (backoff+cap) and must advance the cursor across catch-up to avoid
  double-render (IW-3, proved by the RB2 test).

### Option C — stay-on-poll + periodic WS probe
Remain in poll; periodically re-attempt WS; switch back on success.
- **Pro:** poll stays the spine.
- **Con:** an inverted variant of B; probe cadence competes with poll cadence; no
  advantage over B.

## Recommendation — GO, build as Option B (one slice)

**GO** to implement active reconnect as **Option B**. It closes a real robustness
gap in a feature whose entire value is sub-second latency for long-lived agents,
**preserves** the no-miss correctness guarantee, is a **bounded** consumer-loop
change (no new transport, no protocol/hub change), and is safe by construction
(backoff+cap; Unix still degrades immediately).

### Suggested build slices (post-GO, separate build tasks)
- **RB1** — outer reconnect loop + backoff/jitter/cap around `run_ws_push`; emit a
  "reconnected — back on push" notice symmetric to the degrade notice.
- **RB2** — poll catch-up pass from cursor on each reconnect; assert no double-render
  and no missed offset across a scripted drop (extend `scripts/demo-ws-push.sh` with
  a reconnect segment as the wire evidence — mirrors T-2310).

### Open questions carried to build
- **IW-1** (should we reconnect?) — human go/no-go.
- **IW-2** (which shape?) — analysis favours B; confirm in build.
- **IW-3** (double-render across the gap?) — RB2 scripted-drop test.

## Scope guard

This is a **follow-on past arc-004's GO(scoped) surface**. Degrade-to-poll was the
deliberate v1 ("correctness over dedup; active reconnect is a follow-on", S3 code
comment). This inception exists precisely to get a human go/no-go before any build.
WS-over-Unix (the other documented follow-on) is a **separate** question, queued
behind this one.

## Dialogue Log

- **2026-07-02** — Operator selected "option 2" (follow-on scope decision) from the
  arc-004 completion report. Agent proposed active-reconnect first (hardens the
  shipped path) ahead of WS-over-Unix (new transport). Filed this inception (T-2311)
  with advisory **GO**. Awaiting human go/no-go via `fw task review T-2311` /
  `fw inception decide` (sovereignty-gated; agent cannot self-decide).
