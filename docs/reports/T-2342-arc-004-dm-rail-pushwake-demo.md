# T-2342 — arc-004 dm-rail push-wake isolated-hub regression demo

**Task:** T-2342 · **Arc:** arc-004 push-transport · **Date:** 2026-07-04
**Artifact type:** regression-coverage add (reusable E2E reproducer)

## What this closes

The arc-004 dm rail — S1 (`T-2323`) hub `dm.queued` emit + S2 (`T-2324`) waker
match on `addressee == self-fp` — was verified live exactly **once** (`T-2325`),
by hand, against the shared `:9100` hub after an operator-gated restart. Unlike
the inbox rail (`scripts/demo-pushwaker-e2e.sh`) and the WS reconnect path
(`scripts/demo-ws-reprobe-recovery.sh`), it had **no reusable reproducer**. That
is the exact coverage shape `T-2341` filled for `T-2340` (PL-240: E2E demos catch
integration defects unit tests + happy-path smokes miss). `scripts/demo-dm-rail-pushwake.sh`
adds the missing reproducer for the dm rail, runnable on any tree against an
isolated hub — no operator restart, no shared-hub dependency.

## What it proves (end-to-end, real WS push — no stub)

Against an **isolated** fresh-binary hub (temp `TERMLINK_RUNTIME_DIR` + temp
`HOME`, never touches `:9100` or `~/.termlink`):

1. **A — real PTY session.** `termlink spawn --shell` (tmux backend).
2. **B — real operator path.** `be-reachable.sh start --agent-id <rx>` resolves
   this session's per-agent self-fp (`agent identity --resolve`, the T-2324 /
   PL-236 path) and spawns the waker with **both** rails. The demo asserts
   `pushwaker_pid` is alive, the state file's `self_fp` equals the resolved
   `RX_FP`, and the be-reachable log shows `pushwaker: watching dm.queued for
   '<RX_FP>'` — the dm rail **enabled**, not the `dm rail disabled (no --self-fp)`
   fallback.
3. **C — POSITIVE.** A **non-live** sender (a *separate* per-agent identity that
   never registers a session) posts to `dm:<POSTER_FP>:<RX_FP>`. The hub
   (channel.rs `dm.queued` emit) addresses the frame to the non-sender half
   (`RX_FP`); the waker's dm rail matches and rings — `pushwaker: rang '<pty>'
   via dm.queued` — and `/check-arc respond` lands in the **real** PTY (observed
   on its own terminal).
4. **D — NEGATIVE.** The same poster posts to `dm:<POSTER_FP>:<OTHER_FP>`
   (addressee `OTHER_FP` ≠ `RX_FP`). No new dm ring — no false wake.
5. **E — NO-REGRESSION.** An `inbox:<rx>` deposit in the **same** session still
   rings via the inbox rail (`pushwaker: rang '<pty>' via inbox.queued`).

### The three-distinct-fingerprints trick

A single isolated host stands in for a poster and a receiver the hub sees as
different senders by minting distinct per-agent identities via `TERMLINK_AGENT_ID`
(precedence `FILE > AGENT_ID > DIR > shared-host-default`,
`crates/termlink-session/src/registration.rs`). Each agent-id yields its own
signing key ⇒ its own `sender_id`. This is load-bearing: the hub only emits
`dm.queued` when the poster's `sender_id` equals one half of `dm:<a>:<b>`
(else the "relay / third-party post" `None` branch fires and nothing wakes).

## Result (2026-07-04, `TERMLINK_BIN=target/release/termlink`)

```
=== arc-004 dm-rail push-wake isolated-hub demo (T-2342, proves T-2323/T-2324) ===
dm rail enabled:     yes
positive dm ring:    dm.queued rings 0 -> 1   (>=1 ring on dm:<poster>:<rx>)
doorbell in PTY:     marks 0 -> 2   (/check-arc landed via the dm ring)
negative (no wake):  dm.queued rings 1 -> 1   (unchanged on dm:<poster>:<other>)
inbox no-regression: inbox.queued rings 0 -> 1   (>=1 ring on inbox:<rx>)
RESULT: PASS
```

Exit codes: `0` PASS · `2` binary missing / predates S1 (no `dm.queued`) · `3`
hub/tmux/spawn failed · `4` waker not spawned or dm rail not enabled · `5` no
positive dm ring · `6` false wake (negative) · `7` inbox rail regressed · `8`
fingerprints not distinct.

## Honest outcome vs T-2341

T-2341's demo **falsified** T-2340 and caught 2 real process-death defects. This
demo **passed on the first substantive run** — the dm-rail code (T-2323/T-2324)
was already correct, having been manually validated in T-2325. The value here is
therefore the **reusable reproducer** (future regressions in the `dm.queued`
emit or the addressee-match now fail a script instead of shipping silently), not
a fresh defect find. One real bug was caught and fixed *in the demo itself*
during authoring — a `grep -c … || echo 0` that emitted `"0\n0"` on a zero count
and made the integer comparisons error out (the run "passed" by accident of those
errors being non-zero); fixed to a captured-variable form. Recorded for honesty,
not as a product defect.

## Verification

- `bash -n scripts/demo-dm-rail-pushwake.sh` — clean.
- `scripts/demo-dm-rail-pushwake.sh` — PASS (exit 0), output above. No stderr spam.
- No Rust changed (shell-only add) — no `cargo test` delta; the demo *exercises*
  the already-shipped T-2323/T-2324 code against a real hub.

## Files

- `scripts/demo-dm-rail-pushwake.sh` — the isolated-hub dm-rail reproducer.
- `docs/reports/T-2342-arc-004-dm-rail-pushwake-demo.md` — this report.
