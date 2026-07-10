# T-2396 — WAKE consumption-confirmation: proof + fix (G-083 / PL-253)

**Date:** 2026-07-10
**Operator directive:** "prove-first, then build" — do not ship another unproven
mechanism; demonstrate the failure against the live hub, then close it.

## The question

Field evidence (aef↔workflow-designer, thread T-175 offset 21) showed a wake
"delivered by PTY" that silently died — the recipient never read it. Why does
agent-to-agent comms stop without a manual nudge, and where exactly does a
message die?

## Root cause (PL-253)

A heartbeat proves the **process** is alive, not that the **session** is
listening. The `agent-presence` heartbeat comes from a background script
(`listener-heartbeat.sh`) decoupled from the claude session's availability. A
PTY-inject wake into a session that is busy on its own task or in manual-accept
mode lands in the input box **UNSUBMITTED** and is discarded on the next
`claude --continue` — message durably written (offset N), never read.

## The decisive code finding

`scripts/agent-send.sh` (lines ~499–588) ALREADY contains a consumption
detector: it rings up to `max_rings`, waits `timeout` for a receipt whose
`up_to >= post_offset`, and on no receipt **FAILS LOUD** (`exit 3`,
"receiver never acked", line 586). A receipt is what `/check-arc respond`
(`agent-respond.sh`) posts — so a manual-mode session that never runs check-arc
produces no receipt and agent-send already reports FAILED.

**Therefore the silent failure is a BYPASS, not a missing mechanism:** the paths
people actually reach for skip that receipt-wait.

## Live proof (local hub, 2026-07-10)

**Test A — agent-send detects non-consumption.** Sent via `agent-send.sh` to a
target session that does not exist / runs no `/check-arc respond`:

```
agent-send: ring 1/2 -> inject '/check-arc respond --rail dm:relayproof:t --cid ...'
agent-send: WARN ring 1 — inject ... failed (session missing?); turn already posted, still awaiting receipt
agent-send: ring 2/2 -> inject ...
agent-send: FAILED — no receipt for cid=... after 2 ring(s) (turn posted at offset=0; receiver never acked)
exit code: 3
```

→ The detector fires. Loud, exit 3. (Also confirms B1 rail-stamp is live in the
binary: the doorbell carried `--rail dm:relayproof:t --cid ...`.)

**Test B — raw `termlink inject` has no confirmation.** The same delivery via
raw inject returns immediately with **no receipt-wait and no CONSUMED/FAILED
verdict** — whether it exits 0 (session present, text unsubmitted) or 1 (session
missing), it never confirms consumption. This is the "delivered by PTY" path the
designer used; it is silent by construction. Thread posts (offset 21 on T-175)
bypass equally — no doorbell, no await at all.

**Conclusion:** detector EXISTS in agent-send; raw-inject and thread posts
BYPASS it → silent rung-but-not-consumed.

## The fix

1. **`scripts/wake-confirm.sh`** — the receipt-wait extracted into a standalone
   verb any path can call after a raw ring or thread nudge:
   `wake-confirm.sh --topic <rail> --cid <cid> --since-offset <posted-offset>` →
   exit 0 CONSUMED (receipt acking the offset) or exit 3 NOT-CONSUMED with the
   loud "rung but not read — busy/manual-accept; message unread; remedies…"
   diagnosis. Carries the T-1808 stale-receipt guard (a receipt for an earlier
   turn does not count).
2. **`agent-send.sh` delegates** its per-ring receipt-wait to `wake-confirm.sh`
   → one consumption-confirmation implementation. Existing DELIVERED/FAILED
   output + exit codes preserved (regression-proven: Test A still exits 3 after
   the refactor).
3. **Guidance** in `check-arc.md`: a bare `termlink inject` / thread post has NO
   consumption confirmation — deliver via `agent-send.sh` or follow a raw ring
   with `wake-confirm.sh`.

## What this does and does NOT fix

- **Does:** makes rung-but-not-consumed **loud and reusable** — the silent class
  G-083 named is now observable from any delivery path.
- **Does NOT:** make a busy / manual-accept session *consume* a wake. That is a
  deployment property (recipient must run under `tl-claude.sh --reachable`
  auto-accept, T-2388/PL-237) and, deeper, the architectural tension that an
  interactive session doing its own work is not a reliable instant responder.
  Those remain operator/design decisions — this task closes the *observability*
  gap so the failure can never again be silent.

## Tests

- `tests/relay-wake-confirm.sh` — CONSUMED (acking receipt) vs NOT-CONSUMED
  (none / stale, T-1808 guard), via the `TERMLINK_WAKECONFIRM_TEST_JSON` seam.
- Live Test A re-run post-refactor: agent-send still exits 3 on non-consume.
- `tests/relay-b1-doorbell-rail.sh` + `tests/relay-b2-send-hops.sh` still green.
