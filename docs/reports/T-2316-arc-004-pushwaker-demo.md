# T-2316 — arc-004 push-waker (WP1): demo evidence

**Arc:** arc-004 `push-transport` (WP1 of the T-2315 GO / Option A)
**Task:** T-2316 (build). Predecessors: T-2315 (inception GO), T-2314 (active
reconnect the waker inherits), T-2309/2310/2313 (WS push + demo), T-1800/1834
(doorbell+mail + PTY ring reused unchanged).
**Date:** 2026-07-02

## What changed

Before T-2316, the shipped hub→client WS push (`channel subscribe … --push`) was
proven end-to-end but **demo-only** — no live agent consumed it, so the arc's
headline value (instant DM wake, replacing the receiver's poll-cycle floor) was
*demonstrated, not delivered* (the "shipped but never load-bearing" risk, PL-168
sibling).

T-2316 adds a **background push-waker** to the `be-reachable` lifecycle
(`scripts/be-reachable-pushwaker.sh`). It holds
`termlink channel subscribe inbox.queued --push` and, on each `inbox.queued`
frame whose `addressee_session_id` matches this session's inbox id, fires the
**same** ring `agent-send.sh` already uses:

```
termlink inject <pty_session> "/check-arc respond" --enter
```

`be-reachable.sh start` spawns it detached (only when a `pty_session` is bound),
records `pushwaker_pid` in the state file; `stop` tears it down; `status` reports
it.

## Code-grounded scoping (refined from the inception one-liner)

The hub emits `inbox.queued` **only** for posts to `inbox:<id>` topics
(`crates/termlink-hub/src/channel.rs:748/752`); a `dm:*` post does **not** emit it
(asserted by the test at `channel.rs:3034`). Separately, the `dm:*` doorbell+mail
rail already wakes the receiver instantly via the sender's ring-1 `inject`
(`agent-send.sh:366`). So the waker's delivered value is the **inbox-deposit /
store-and-forward / no-live-sender** receive path — where the receiver would
otherwise wait for its own poll cycle (the 15 s floor T-2303 §10.1 targets).

**Durability unchanged.** WS is a faster *trigger*, never a source of truth. On WS
drop the CLI's built-in active reconnect (T-2314) resumes push; the durable inbox
+ sender-ring + receiver `/check-arc` cadence remain the floor. A single deposit is
deduped per `(addressee, message_offset)`; cross-rail double-wake (push + a live
sender's ring) is bounded by `/check-arc respond` idempotency.

## Evidence

### Pure filter/dedup unit test — `scripts/test-pushwaker-filter.sh`

```
ok: extract strips push prefix
ok: frame matches self -> RING <offset>
ok: frame for other addressee -> SKIP
ok: missing addressee -> SKIP no-addressee
ok: missing offset -> SKIP no-offset
ok: first sighting rings (no last)
ok: duplicate offset within ttl skips
ok: same offset after ttl rings again
RESULT: PASS
```

### Loopback wire demo — `scripts/demo-pushwaker.sh` (isolated hub + HOME)

A stub `termlink` wrapper logs `inject` calls (proving the ring) and passes
`subscribe`/`post`/`create` through to the real isolated hub — so the ring is
asserted without a live Claude PTY session.

```
=== arc-004 push-waker loopback demo (T-2316) ===
binary:           target/release/termlink
hub:              127.0.0.1:9198   (isolated runtime_dir, torn down on exit)
self inbox:       inbox:pw-3968542   ring target pty: fakepty-3968542
positive ring:    172 ms  (INJECT inject fakepty-3968542 /check-arc respond --enter)
rings after self: 1
rings after other:1  (must equal rings-after-self — no false wake)

RESULT: PASS — inbox deposit rang the PTY sub-second (172 ms) via push,
        and a deposit to another inbox was filtered (no false wake).
```

The load-bearing lines: **positive ring 172 ms** (a deposit to `inbox:<self>`
rang the PTY sub-second over the WS push) and **rings-after-other == rings-after-self**
(a deposit to a *different* inbox was pushed to the same waker but filtered — no
false wake). This is the first end-to-end proof that the shipped WS push actually
*wakes a session*, not just a demo consumer.

## Scope / follow-on

- WP1 targets the clean `inbox.queued` doorbell frame (the proven demo path). A
  waker on `dm:<self>:*` direct pushes (the WS consumer already accepts that topic,
  `ws_consumer.rs:244`) is a natural follow-on but broadens envelope parsing +
  self-fp resolution — deferred.
- The exact production binding of a session's inbox id (defaulted here to the
  `agent_id`) and a live-PTY (non-stub) proof of degrade-to-poll + no-double-wake
  under a real WS drop are the WP2 slice.
