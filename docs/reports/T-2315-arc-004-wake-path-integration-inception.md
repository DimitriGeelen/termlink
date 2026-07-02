# T-2315 — arc-004 inception: wake-path integration (consume the WS push in the live doorbell path)

**Type:** inception (one question, go/no-go)
**Arc:** arc-004 `push-transport` (integration follow-on — closes the gap between shipped
mechanism and delivered value)
**Agent recommendation:** **GO (advisory)** — Option A. Final decision: human (sovereignty-gated).
**Date:** 2026-07-02

---

## The one question

arc-004 shipped a working, proven hub→client WebSocket push (`channel subscribe <topic>
--push`): a DM produces an `inbox.queued` doorbell frame the instant it is posted
(~88 ms TCP / 31 ms Unix, T-2310/T-2313), it degrades to poll on drop, and it now
**auto-reconnects** after a blip (T-2314). But **nothing in the live agent wake path
consumes it** — the `--push` verb is exercised only by demo scripts. A real agent is still
woken by the doorbell+mail loop's poll, on the **15 s floor** the arc set out to remove
(T-2303 §10.1: "replace the 15 s-floor doorbell-then-poll wake+read").

**Should the live wake path adopt the WS push, and if so, how does a pushed
`inbox.queued` frame wake a Claude Code PTY session without regressing the T-1800
doorbell+mail durability?**

## Why this matters (problem)

- The arc's headline value — *instant* DM delivery to a live agent — is currently
  **demonstrated, not delivered**. Agents that `/be-reachable` still see the 15 s wake
  floor for every inbound DM.
- The mechanism, the CLI surface, the reconnect robustness, and the degrade path are all
  already built and proven. The only missing link is the ~one consumer that turns a
  pushed frame into a PTY wake.
- Leaving it unwired risks the classic "shipped but never load-bearing" decay (PL-168
  sibling): a correct mechanism nobody calls.

## What is NOT the problem (durability is already safe)

- The durable substrate (offline-queue / post-idempotency / delivery-confirm / journal /
  receipts) stays authoritative — this inception changes **only the wake/read transport**,
  exactly as T-2303 §8.4 / §10.3 constrained. WS is a latency optimization over the
  existing durable inbox, never a new source of truth.
- On any WS drop the consumer falls back to the current poll (T-2314 then reconnects), so
  a missed push is at worst 15 s-late, never lost.

## The wake path today (code-grounded)

- `scripts/be-reachable.sh` opts a session into `agent-presence` and records a
  `--pty-session` — the doorbell **ring target** (T-1834).
- `scripts/agent-send.sh` (sender) posts the DM + doorbell and **polls** for the receipt,
  injecting a wake into the receiver's PTY (T-1800/T-1804/T-1805 doorbell+mail loop).
- `scripts/agent-respond.sh` closes the receiver half after a `/check-arc`-surfaced unread.
- The 15 s floor lives in that poll cadence. `inbox:<self>` is the durable topic a DM
  lands on; the hub already emits an `inbox.queued` aggregator event for it (channel.rs:753,
  T-1637) — the exact frame `channel subscribe inbox.queued --push` streams.

## The opportunity

A background process running `channel subscribe inbox:<self> --push` (or the
`inbox.queued` aggregator filtered to self) receives the doorbell frame instantly and
**rings the existing PTY doorbell** — reusing `be-reachable`'s `--pty-session` target and
the T-1800 injection path unchanged. The durable inbox + receipts stay exactly as they
are; only *when* the ring fires changes (instant vs ≤15 s).

## Options considered

### Option A — background WS-push consumer that rings the existing doorbell  ✅ recommended
Add a push-driven waker to the `be-reachable` lifecycle: it holds a `--push` subscription
to the session's own inbox and, on each `inbox.queued` frame, fires the **same** PTY ring
`be-reachable`/`agent-send` already use. On WS drop it degrades to the current poll
cadence; T-2314 reconnects it.
- **Pro:** smallest change that delivers the value; reuses the proven ring + the proven
  push consumer; durability untouched; fully reversible (stop the waker → back to poll).
- **Con:** a second long-lived process per reachable session (bounded — one `--push`
  subscription); must dedupe against the existing poll so a DM woken by push isn't also
  woken by poll (idempotent ring / receipt-gated — IW-3).

### Option B — replace the sender-side poll in `agent-send.sh` with a push-driven receipt wait
Have the sender await delivery via a WS subscription to the receipt topic instead of
polling.
- **Pro:** removes a poll rather than adding a process.
- **Con:** re-architects the load-bearing doorbell+mail handshake (T-1800) on the *sender*
  side — higher blast radius on the exact path T-2285's ack-with-retry gap already flags as
  fragile. Heavier than the value warrants for v1.

### Option C — null (keep the 15 s poll)
- **Pro:** zero risk.
- **Con:** the arc's headline value is never delivered; the shipped mechanism stays
  demo-only.

## Recommendation — GO (advisory), build as Option A

Option A is the bounded, additive, reversible path that finally delivers the arc's value:
instant wake for reachable agents, reusing the already-proven push consumer and the
already-proven PTY ring, with the durable substrate and the 15 s poll both intact as the
fallback floor. It touches the sensitive wake path, so it needs a human GO — but the design
deliberately **adds** a faster trigger beside the existing one rather than replacing the
durable handshake.

### Suggested build slices (post-GO, separate build tasks)
- **WP1** — a `be-reachable` push-waker: background `channel subscribe inbox:<self> --push`
  that fires the existing PTY ring on `inbox.queued`, with receipt/idempotent dedup against
  the poll path; lifecycle bound to `/be-reachable start|stop`.
- **WP2** — loopback wire evidence: extend `agent-conversation-selftest.sh` (or a sibling)
  to prove a posted DM wakes the PTY sub-second via push, and that a WS drop falls back to
  the poll wake with no double-wake and no lost DM.

### Open questions carried to build
- **IW-1** (adopt WS for the wake path at all?) — human go/no-go.
- **IW-2** (which shape — add-a-waker (A) vs replace-the-poll (B)?) — analysis favours A;
  confirm in build.
- **IW-3** (double-wake / dedup?) — a DM must not wake the PTY twice (push + poll). The
  receipt/ack is the natural idempotency key; WP2 proves no double-wake. Ties to T-2285.

## Scope guard

This is an **integration** follow-on strictly within arc-004's GO(scoped) surface (T-2303
§10.1 named "replace the doorbell-then-poll wake+read"). It is **not** webhooks (A1, a
separate deferred external-only inception) and it does **not** touch the durability layer
(§10.3). One question: adopt the shipped push in the live wake path, yes/no + shape.

## Dialogue Log

- **2026-07-02** — After building both prior arc-004 follow-ons (T-2313 WS-over-Unix,
  T-2314 active reconnect), the agent identified that the shipped WS push is still
  demo-only — no live agent consumes it, so the arc's 15 s→sub-second payoff is
  undelivered. Filed this inception (T-2315) with advisory **GO** (Option A). Awaiting
  human go/no-go via `fw task review T-2315` (sovereignty-gated; agent cannot self-decide).
