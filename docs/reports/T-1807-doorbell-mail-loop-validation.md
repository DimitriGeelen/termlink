# T-1807 — Doorbell+Mail Loop: Multi-Turn End-to-End Validation

**Task:** T-1807 (T-1800 build #4 / spike S-5)
**Date:** 2026-05-25
**Status:** transport + ritual validated deterministically; live two-real-claude
soak split to follow-ups (blockers found — see below).

## What was validated

A ≥3-turn structured conversation over the doorbell+mail runtime, end to end,
using only the shipped primitives:

- sender: `scripts/agent-send.sh` (T-1804, with the T-1808 offset-aware fix)
- receiver ack+reply: `scripts/agent-respond.sh` (T-1805)

All three sender turns reported **DELIVERED** (exit 0), each detecting its OWN
fresh receipt (offsets 1 → 4 → 7, not a stale earlier one).

### Transcript (single conversation_id, captured via `channel subscribe --json`)

| offset | msg_type | meaning | up_to |
|---|---|---|---|
| 0 | turn    | sender turn 1   | — |
| 1 | receipt | acks turn 1     | 0 |
| 2 | turn    | listener reply 1| — |
| 3 | turn    | sender turn 2   | — |
| 4 | receipt | acks turn 2     | 3 |
| 5 | turn    | listener reply 2| — |
| 6 | turn    | sender turn 3   | — |
| 7 | receipt | acks turn 3     | 6 |
| 8 | turn    | listener reply 3| — |

Counts: **6 turns** (3 sender + 3 listener) + **3 receipts**, all on one cid.

### Properties confirmed (T-1807 description)

- **Determinism — every turn acked.** Each sender turn (offsets 0/3/6) has a
  matching receipt whose `up_to` (0/3/6) equals that turn's offset. No silent
  drop; PL-011 satisfied per turn.
- **A-4 — content via `channel.*`, PTY never scraped.** Delivery was confirmed
  purely from `channel` receipt envelopes. The doorbell `inject` targeted a
  non-existent session (non-fatal) and contributed nothing to delivery
  detection — the PTY was never read.
- **T-1808 multi-turn correctness.** Receipt watermarks 0/3/6 prove the
  offset-aware fix: turn 2 did NOT match turn 1's stale receipt.

## Blockers found for the live two-real-claude soak

The original AC envisaged a live `claude` listener woken by the doorbell. Two
real constraints surfaced during materialization:

### G-a — nested `claude` under root can't use `--dangerously-skip-permissions`

`termlink spawn ... -- claude --dangerously-skip-permissions` exits immediately:

```
--dangerously-skip-permissions cannot be used with root/sudo privileges for security reasons
```

A hands-free listener on this host (root) must instead **allowlist
`Bash(termlink:*)`** in `.claude/settings.local.json` so a plain spawned
`claude` runs `agent-respond.sh` without a permission prompt. The allowlist
currently lacks this entry. (Recipe updated — see
`docs/operations/injectable-listener-spawn-recipe.md`.)

### G-b — the doorbell text `/check-arc` does not signal respond mode

`agent-send.sh` injects `/check-arc` as the doorbell. But `/check-arc` defaults
to **browse mode (read-only)** — a woken claude has no way to tell it was rung
by a peer (and should ack+reply) versus invoked manually by an operator (and
should stay read-only). Without a respond-mode signal in the doorbell, a live
claude listener would read the turn but never post a receipt, so the sender
would never see DELIVERED. The mechanical responder sidesteps this by calling
`agent-respond.sh` directly; a live claude needs an explicit signal.

→ Filed **T-1809** (respond-mode doorbell signal) and **T-1810** (live
two-real-claude soak, blocked on T-1809 + the allowlist).

## Conclusion

The doorbell+mail **transport and respond ritual are correct and multi-turn-safe**
— the conversational runtime works end to end. The remaining piece — two live
reasoning agents driving the same loop autonomously — is gated on T-1809/T-1810,
which are now tracked with concrete, reproduced blockers rather than guesses.
