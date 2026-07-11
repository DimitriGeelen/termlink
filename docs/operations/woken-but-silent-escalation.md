# Woken-but-silent escalation (T-2402 Stage 5)

`scripts/agent-send.sh` posts a turn (mail), rings the recipient's doorbell, and
polls for a **receipt** — re-ringing up to `--max-rings`. Before T-2402, when the
recipient never acked, the loop just printed a stderr `FAILED` line and
`exit 3` — **invisible** to anyone not tailing that process. That is the sender
half of the T-2400 "off=7" silence: the message was durably delivered, the ring
fired, but a woken-yet-unanswering agent left nothing an operator could see.

Worse, the pre-T-2402 code claimed on the fallback path that "the unconfirmed-
delivery canary (T-2295) tracks it until acked." That was **false**: step-1 posts
the turn WITHOUT `--await-ack`, so no awaiting-ack obligation (`~/.termlink/awaiting_ack.sqlite`,
T-2286) is ever registered — nothing for T-2295 to read. Both give-up paths were
silent.

## The escalation

On exhaustion (BOTH the direct and fallback give-up paths), `agent-send.sh` now
calls `escalate_woken_but_silent`, which appends a framed entry to a
`*-canary.log` that **`/canaries` (T-2172) auto-discovers** (it globs
`.context/working/.*-canary.log`):

```
=== 2026-07-11T09:42:00Z ===
woken-but-silent: no receipt for cid=<cid> on topic=<topic>
  recipient=<agent-id> session=<pty> [hub=<peer_hub>]
  turn posted at offset=<n>; rings=<N>; reason=<why>
  remediation: confirm peer LIVE (/peers --all); re-send (/agent-handoff <peer> <task> "..."); or drop the thread if dead
---
```

- **Empty log = healthy** (append-only, written only on a genuine give-up) —
  same convention as every other canary.
- The log path resolves **relative to the script** (`$SELF_DIR/../.context/working/…`),
  not the caller's cwd, so the escalation lands in the same checkout whose
  `/canaries` surfaces it — regardless of where the sender ran. Override with
  `TERMLINK_WOKEN_SILENT_LOG`.

## Operator workflow on firing

`/canaries` shows the log FIRING (it has entries newer than any heartbeat; this
is an **event-written** canary, not a cron-swept one — `/canaries` classifies it
`NO_HEARTBEAT` + FIRING on content, which is correct here). For each entry:

1. Is the peer actually LIVE? `/peers --all` (look for the recipient with a
   `pty_session`). A dead/unreachable peer explains the silence.
2. If LIVE but silent — re-send: `/agent-handoff <peer> <task> "..."` (opens a
   fresh thread) or `/reply <peer> "..."` (answers the existing one).
3. If the thread is dead (task abandoned, peer decommissioned) — drop it.
4. Clear the handled entries (truncate the log) so it returns to empty=healthy.

## Relationship to the other stages

- **Stage 3** (`docs/operations/pushwaker-idle-gating.md`) makes the *receiver*
  side deterministic — the doorbell lands at an idle prompt instead of being
  swallowed mid-turn. Stage 5 makes the *sender* side loud when, despite a
  landed doorbell, no receipt comes back.
- **Stage 6** (pending) tightens the `/check-arc respond` obligation so a woken
  agent always acks — making a Stage-5 firing unambiguously a bug, never a valid
  unlogged "I chose not to reply."

## Tests

`scripts/test-agent-send.sh` Path B2 (against a live local hub): sends to a
non-existent session so the receipt never comes, then asserts the `ESCALATED`
stderr line AND a well-formed canary-log entry naming the cid. Runs alongside the
existing Path B (rc=3 + capped rings) — the escalation is additive, no regression.

## Related

- T-2400 — the off=7 blind-inject/silent-give-up demo this closes.
- T-2286 / T-2287 / T-2295 — the awaiting-ack obligation + unconfirmed-delivery
  canary (the durable-obligation mechanism; agent-send runs its own receipt loop
  in parallel and escalates to a canary log rather than that sqlite tracker).
- T-2172 `/canaries` — the auto-discovery surface that renders this log.
