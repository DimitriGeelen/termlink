# T-2341 — arc-004 WS re-probe recovery demo (proves T-2340, completes the hard-down recovery)

**Task:** T-2341 · **Arc:** arc-004 push-transport · **Date:** 2026-07-04
**Artifact type:** demo evidence + defect RCA (fix-forward on T-2340)

## What this proves

T-2340 added a periodic WS **re-probe** from the steady poll floor so a long-lived raw
`termlink channel subscribe --push` consumer regains sub-second push after a hub outage without
a process restart. Its evidence was unit-tested gating + a happy-path smoke. This task adds the
**end-to-end recovery demo** (`scripts/demo-ws-reprobe-recovery.sh`) — the sequel to
`scripts/demo-ws-push.sh` — that drives the *full hard-down path*:

1. live post → push (baseline: the WS is delivering);
2. **hard** hub-down long enough that the reconnect loop exhausts its 6-attempt anti-spin cap
   and prints `WS reconnect cap (6) reached — degrading to poll` (the distinguishing evidence vs
   `demo-ws-push.sh`, whose short blip never hits the cap);
3. hub restart (same runtime_dir → no secret/cert rotation);
4. the re-probe re-enters the reconnect loop from the poll floor and a DM posted after the
   restart is delivered to the **same** consumer process — no restart.

## The demo caught two real defects (fix-forward)

The demo **falsified** the claim that T-2340 alone achieved recovery. Running it surfaced two
`?`-exit bugs in the degrade/poll path that the unit tests and the happy-path smoke could not
see (they never exercised a consumer that reaches the poll floor while the hub is down):

1. **Poll-RPC `?`-exit.** After the reconnect loop degraded to the poll loop with the hub still
   down, the first `rpc_call_authed(... channel.subscribe ...).context("Hub rpc_call failed")?`
   propagated `Connection refused` and **killed the consumer process** — the T-2340 re-probe
   could never fire.
2. **Hub-level-error `?`-exit.** After surviving (1), the restarted in-memory hub had not yet
   recreated the aggregator topic, so `unwrap_result` propagated `-32013: unknown topic:
   inbox.queued` and again exited the process.

**Root cause:** the poll loop treated *any* transient hub condition (down / restarting /
topic-not-yet-present) as fatal via `?`, even for an inherently long-lived consumer.

**Fix (T-2341):** in a long-lived consumer (`--follow` **or** the inherently-live `--push`),
both the connection-level and hub-level poll errors are now ridden out — log, advance the
re-probe cadence counter, sleep, and continue — instead of exiting. A true single-shot
(`!follow && !push`) subscribe still errors as before. `--push` is also no longer subject to the
`!follow` single-shot early-return (it was already effectively infinite via the reconnect loop).
The re-probe was moved to the **top** of the poll loop so it fires on both the normal and the
hub-down-retry paths, and it now emits an explicit `re-probing WS from poll floor` line so a
*successful* (silent-streaming) re-probe is observable to operators.

**Why structurally allowed:** the degrade-to-poll path (T-2309/T-2314) was only ever exercised
by short blips where the hub returned inside the reconnect window; no test drove a consumer onto
the poll floor against a *down* hub. **Prevention:** this demo (a regression reproducer) plus the
env-tunable cadence (`TERMLINK_WS_REPROBE_POLL_CYCLES`) that makes it deterministic.

## Env knob (T-2341)

`TERMLINK_WS_REPROBE_POLL_CYCLES` (default 30, clamped 1..=3600) tunes the re-probe cadence in
poll cycles (~1s each). Mirrors the `TERMLINK_WEBHOOK_RETRY_INTERVAL_MS` clamp pattern. Lower =
faster recovery / deterministic demos; higher = less re-probe churn on a busy hub. Read once per
subscribe. The demo sets it to `2` so the re-probe fires ~2s after the degrade instead of ~30s.

## Demo run (2026-07-04, `TERMLINK_BIN=target/debug/termlink`)

```
=== arc-004 WS re-probe recovery demo (T-2341, proves T-2340) ===
binary:             target/debug/termlink
hub:                127.0.0.1:9198   (isolated runtime_dir, torn down on exit)
topic:              inbox:demo-reprobe-<pid>
re-probe cadence:   TERMLINK_WS_REPROBE_POLL_CYCLES=2 (default 30)
baseline push:      DM delivered live via WS push
degrade-to-poll:    [push] WS reconnect cap (6) reached — degrading to poll
re-probe fired:     yes — fresh [push] activity after the degrade (only the re-probe emits [push] from the poll floor)
push recovered:     yes — post-restart DM delivered to the SAME consumer, no restart

RESULT: PASS — the consumer hit the reconnect cap and DEGRADED to poll, then the
        T-2340 re-probe recovered live push after the hub returned, WITHOUT a restart.
```

Isolation contract (same as `demo-ws-push.sh`): runs entirely under a temp
`TERMLINK_RUNTIME_DIR` + temp `HOME`; never touches the shared `:9100` hub or `~/.termlink`; hub
torn down on exit. Exit codes: `0` PASS · `2` binary missing · `3` hub start failed · `4` no
baseline push · `5` no degrade-to-poll · `6` push did not recover.

## Verification

- `cargo build -p termlink` — clean.
- `cargo test -p termlink --bin termlink` — **958 passed / 0 failed** (PL-238 full suite; WS/poll
  path substantially restructured). Includes 2 new `clamp_reprobe_cycles_*` tests + the existing
  `ws_reprobe_*` tests updated to the `(cycles, threshold)` signature.
- `scripts/demo-ws-reprobe-recovery.sh` — PASS (exit 0), output above.

## Files

- `crates/termlink-cli/src/commands/channel.rs` — env-tunable cadence
  (`clamp_reprobe_cycles`/`ws_reprobe_poll_cycles`), poll-loop hub-down resilience
  (`follow || push`), re-probe moved to loop top + observable log line, `--push` no longer
  single-shots the poll floor.
- `scripts/demo-ws-reprobe-recovery.sh` — the recovery reproducer.
- `docs/reports/T-2341-arc-004-ws-reprobe-recovery-demo.md` — this report.
