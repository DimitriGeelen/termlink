# T-2318 — arc-004 push-waker: LIVE end-to-end proof (real spawn + real inject)

**Arc:** arc-004 `push-transport` (verification of the T-2315 GO / Option A surface)
**Task:** T-2318 (test). Predecessors: T-2316 (WP1 push-waker), T-2317 (WP2 blip),
T-2315 (inception GO), T-2314 (active reconnect the waker inherits).
**Date:** 2026-07-03

## Why this task exists

WP1 (T-2316) and WP2 (T-2317) prove the waker *logic* — addressee filter, per-offset
dedup, blip-resume — but both invoke `be-reachable-pushwaker.sh` **directly** and
replace `termlink inject` with a **stub** that only appends the command to a log.
Two seams of the *shipped operator path* were therefore never exercised end-to-end:

1. the `be-reachable.sh cmd_start` wiring that actually **spawns** the waker as a
   detached process and records `pushwaker_pid` (and `cmd_stop` that reaps it); and
2. a **real** `termlink inject` landing in a **real** PTY-backed session on an inbox
   deposit — observed on the receiving session's own terminal, not a log stub.

This task closes both against an isolated hub + HOME (no real-fleet writes).

## Evidence — `scripts/demo-pushwaker-e2e.sh`

Hermetic: isolated `TERMLINK_RUNTIME_DIR` + `HOME` + loopback TCP hub (`127.0.0.1:9195`),
all helpers pinned to the built binary via `TERMLINK_BIN`. Steps: (A) `termlink spawn
--shell --backend tmux` a real PTY session; (B) run the real `be-reachable.sh start
--agent-id <self> --pty-session <pty>` and assert it spawned a live waker; (C) POSITIVE
deposit to `inbox:<self>`; (D) NEGATIVE deposit to `inbox:<other>`; (E) `be-reachable.sh
stop` and assert the recorded pid is reaped.

```
=== arc-004 push-waker LIVE end-to-end demo (T-2318) ===
binary:              target/release/termlink
hub:                 127.0.0.1:9195   (isolated, real TCP hub)
self agent/inbox:    e2e-884049 / inbox:e2e-884049
real PTY session:    e2e-pty-884049   (termlink spawn --shell, tmux backend)
push-waker pid:      884359   (spawned by be-reachable start)
doorbell marks:      before=0  after-self=2  after-other=2  (>=1 ring on self, no new ring on other)
session output tail (real inject landed here):
    | # /check-arc respond
    | /bin/sh: 1: /check-arc: not found
    | #

RESULT: PASS — be-reachable start spawned a live waker; a real inbox:<self>
        deposit drove a REAL termlink inject into a REAL PTY session (observed on
        its own terminal); an inbox:<other> deposit did not (no false wake); and
        be-reachable stop reaped the waker.
```

The load-bearing lines:
- **push-waker pid 884359, spawned by `be-reachable start`** — proves seam (1): the
  real operator entrypoint spawned the waker and recorded its pid in the state file.
- **doorbell marks before=0 → after-self=2** — proves seam (2): the deposit to
  `inbox:<self>` drove a real `termlink inject "/check-arc respond"` into the real
  PTY session; the session's *own terminal output* (read via `termlink output
  --strip-ansi`) now shows `# /check-arc respond` and `/bin/sh: 1: /check-arc: not
  found`. The inner `/bin/sh` genuinely received and executed the injected keystrokes.
- **after-other=2 (unchanged)** — the `inbox:<other>` deposit produced **no** new
  ring: the addressee filter holds against a real hub, not just a unit test.
- **stop reaped the waker** — `be-reachable stop` terminated the recorded
  `pushwaker_pid`.

## What this ADDS over T-2316 / T-2317

| Seam | T-2316/T-2317 | T-2318 (this) |
|------|---------------|---------------|
| waker spawn wiring | script invoked directly | via real `be-reachable start` |
| inject transport | **stub** (logs command) | **real** `termlink inject` |
| ring observation | grep a stub log | `termlink output` on a real PTY |
| filter under real hub | unit `pushwaker_decide` | live `inbox:<other>` deposit |
| stop/reap lifecycle | not exercised | real `be-reachable stop` |

It does **not** re-test blip/reconnect (that is T-2317's scope) and it does not add
any feature — it is verification of already-shipped code.

## Finding: stop-path orphan leak (filed as T-2319)

Building this harness surfaced a real bug the stub demos structurally could not.
`be-reachable-pushwaker.sh` holds its `channel subscribe inbox.queued --push` child
via `done < <(… --push)` **process substitution with no trap**. `cmd_stop` SIGTERMs
the waker *script* (and the recorded `pushwaker_pid` IS reaped — this demo's AC still
passes), but the subscribe child is **orphaned**, and with the T-2314 active reconnect
it loops against the hub forever. Reproduction: after this demo's cleanup kills the
isolated hub, a lingering `channel subscribe inbox.queued --push` process remains.
Fix tracked in **T-2319** (add a child-reaping trap + regression assertion); this demo
is its reproduction harness.

## Arc status

WP1 (logic) + WP2 (blip) + this E2E (real operator path) give the human three
independent layers of evidence for the sovereignty-gated `fw arc close push-transport`.
Remaining follow-on: the T-2319 leak fix (not a blocker for arc close — it is a stop
hygiene bug, not a wake-path correctness bug).
