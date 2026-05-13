---
response_id: RESP-T-1804
in_reply_to: PROP-T-1804
source_repo: termlink (DimitriGeelen/termlink)
source_task: T-1635
target_repo: 999-Agentic-Engineering-Framework
target_task: T-1804
status: draft
created: 2026-05-13
---

# TermLink-side response: cross-agent conversation substrate seam

This document answers the four decision points in AEF PROP-T-1804 from the
TermLink maintainer perspective.

---

## 1. Seam acknowledgement

**Agreed.**

The proposed split maps exactly to TermLink's existing ownership model:

| Layer | Owns | Evidence already in production |
|-------|------|-------------------------------|
| **TermLink** | Transport: channels, events, topics, inbox, delivery confirmation, cross-machine relay via `termlink remote`, session discovery, PTY inject | `agent-chat-arc`, `dm:<a>:<b>`, `termlink agent contact`, `termlink inbox`, TCP hub |
| **AEF** | Semantics: when to consult, task-context anchoring, spawn policy, audit trail, Watchtower surface | `fw termlink dispatch`, `fw bus post --remote`, Worker dispatch envelopes |

The pattern is already live: `fw termlink dispatch` is an AEF semantic wrapper over
TermLink spawn; `agent-chat-arc` / `dm:*` channels are transport infrastructure AEF
addresses as the consumer. This proposal extends the same split to the wakeup case.

**No negotiation needed on the seam location.**

---

## 2. Wakeup choice

**Option (i) — with one refinement.**

The AEF proposal describes option (i) as "TermLink fires `$WAKEUP_CMD` on
undeliverable message." That exact form is vetoed (see §4). The refinement:
TermLink emits a **new event class** (`inbox.queued`) rather than executing a
consumer-supplied spawn string. AEF subscribes to this event class and decides
what to spawn.

**Why not (ii):** AEF-side long-running daemon is a design smell the framework
explicitly avoids. Not viable.

**Why the refinement over raw (i):** A `$WAKEUP_CMD` stored per-channel or
per-session makes the hub a command executor for consumer-specific spawn logic.
Two concrete problems:
1. TermLink does not know which consumers are AEF and which are not — storing a
   spawn string that works for one consumer breaks the domain-neutral principle.
2. The hub runs `$WAKEUP_CMD` on **its own host**. For cross-machine delivery
   (sender on host A, recipient on host B, hub on host B), this executes correctly.
   But if hub federation ever lands, the command-execution model becomes ambiguous.
   An event class has no such ambiguity.

**Why option (i)-via-event instead of pure (iii):**
AEF's (iii) framing ("TermLink emits event, AEF daemon subscribes") still requires
a persistent AEF subscriber to catch the event — otherwise the event fires into a
void. The distinction between (i) and (iii) collapses unless the subscriber is
itself cron-managed. With TermLink's existing `event.subscribe` long-poll RPC
(T-690: shipped, available today), AEF can run a cron job that long-polls
`--topic inbox.queued --timeout 30` for up to 30 seconds then exits. Next cron
tick re-enters the long-poll. This gives near-zero wakeup latency (~event arrival
time) without a daemon. The cron job IS the AEF subscriber bridge — it just
doesn't run continuously.

**Concrete event/hook surface:**

New event class: **`inbox.queued`**

Emitted by: hub, inside the inbox delivery path, when a message is enqueued for
an addressee session with no currently-registered live consumer connection.

Payload (minimal — no message body):
```json
{
  "topic": "inbox.queued",
  "addressee_session_id": "<session-id-or-fingerprint>",
  "channel": "dm:<a>:<b>",
  "message_offset": 42,
  "enqueued_at": 1747123456789
}
```

Subscription: existing `termlink event poll --topic inbox.queued` or
`event.subscribe --topic inbox.queued --timeout 30000` (long-poll, T-690).

No new CLI verb needed. No new config field needed. `event.subscribe` MCP tool
(`mcp__termlink__termlink_event_subscribe`) works today if the event class is
registered.

---

## 3. Bounded cost estimate

**TermLink-side (this repo):**

| Deliverable | Location | Est. lines | New surface? |
|-------------|----------|------------|-------------|
| `inbox.queued` event class constant | `termlink-protocol/src/events.rs` | ~5 | No — adds to existing event taxonomy |
| Hub inbox delivery: emit `inbox.queued` when no live consumer | `crates/termlink-hub/src/…` (inbox/delivery path) | ~15 | No — uses existing `EventBus::emit()` |
| Integration test: verify event fires on no-consumer inbox delivery | `tests/` | ~20 | No |

**Total TermLink cost: ≤1 new event class, ≤40 lines changed/added, no new CLI
verb, no new config field, no new daemon, no new protocol version required.**

Note: `event.subscribe` long-poll RPC is already in the binary (T-690). MCP
`termlink_event_subscribe` tool is already registered. AEF can use both today
once the event class exists.

**AEF-side (not TermLink's concern but included for completeness):**

- 1 subscriber bridge: cron-managed long-poll job (~30 lines), exits after
  catching 0-N `inbox.queued` events, spawns responders via `claude-fw -p`,
  re-enters on next cron tick.
- 1 audit channel under `.context/conversations/` (per PROP-T-1804 — AEF-owned).

---

## 4. Constraints / vetoes

### Veto: no `$WAKEUP_CMD` per channel or session

TermLink will not store and execute consumer-specific spawn strings. Reasons:

1. **Security surface.** A configurable command string executed by the hub is a
   shell injection foothold. TermLink already limits `command.execute` via an
   allowlist; adding a per-channel hook with arbitrary commands inverts that.
2. **Domain neutrality.** The hub serves many consumers. AEF's `claude-fw` spawn
   string is meaningless to a non-AEF consumer and should not be stored in
   hub-side state.
3. **Cross-machine ambiguity.** On which host does `$WAKEUP_CMD` run? On the hub
   host for cross-machine delivery, this is correct — but the event approach is
   cleaner and does not require TermLink to answer that question.

The `inbox.queued` event class achieves option (i)'s goal (zero-latency wakeup
signal from TermLink) without these risks.

### Constraint: `inbox.queued` payload must not include message body

The event payload contains addressee, channel, offset, and timestamp only.
The message body remains in the inbox, fetched by the spawned responder via
normal `inbox.list` / `channel.subscribe`. This prevents the event bus from
becoming a second copy of the message store and avoids size surprises on the
event ring buffer.

### Constraint: event is hub-local, not relayed cross-hub

`inbox.queued` is emitted by the hub that owns the inbox delivery (i.e., the
recipient's local hub). It is NOT relayed across `termlink remote` boundaries.
This is the correct behavior — see §5 for why this works cross-machine.

### Open question: T-243 dialog heartbeat interaction

T-243 (GO 2026-04-26) is building a `dialog.heartbeat` typed RPC and extending
`channel.post` with `metadata.conversation_id` + `metadata.event_type`. A dialog
message arrival for a non-running responder should also trigger `inbox.queued`
(or a dialog-specific variant). We propose that the same `inbox.queued` event
works for both plain channel posts and dialog turns — the `channel` field in the
payload identifies the source, letting AEF's subscriber bridge route to the right
responder. If T-243 introduces a `dialog.queued` variant later (for semantics
like "yield waiting for response"), that is a follow-on addition; the base
`inbox.queued` is sufficient for v1 peer-consult wakeup.

---

## 5. Cross-machine question

**Option (i) via `inbox.queued` event is fully cross-machine correct without
requiring cross-hub event relay.**

Delivery path for cross-machine peer-consult:

```
Sender (host A)
  └─ termlink remote send → hub:A → TCP → hub:B
                                          └─ inbox delivery (host B)
                                             └─ emit inbox.queued on hub:B's EventBus
                                                └─ AEF cron subscriber on host B
                                                   catches event → spawns responder on host B
```

Key properties:
- `inbox.queued` is emitted by hub:B (the delivery hub) — always on the
  correct machine for spawning
- AEF's subscriber bridge subscribes to its **local** hub only
  (`termlink event poll --topic inbox.queued` with no `--hub` flag)
- No cross-hub event relay needed
- No per-machine configuration of AEF spawn paths on TermLink's side

**Comparison to raw option (i) `$WAKEUP_CMD`:** Also hub-local by construction
(hub:B runs the command on host B). The event approach and the hook approach are
equivalent for cross-machine delivery. The event approach wins on the security
and domain-neutrality grounds in §4.

**Single-host case:** Trivially correct — sender, recipient, and hub are all on
the same machine.

---

## Summary: answers to PROP-T-1804's four decision points

| # | Question | Answer |
|---|----------|--------|
| 1 | Accept seam (transport-only on TermLink, semantics on AEF)? | **YES — agreed, no negotiation needed** |
| 2 | Prefer (i), (ii), or (iii)? | **(i) refined — new `inbox.queued` event class, not $WAKEUP_CMD** |
| 3 | Is new event class acceptable scope for next minor version? | **YES — ≤40 lines, no new CLI verb, fits within T-243 delivery window** |
| 4 | Cross-machine wakeup — machine-local or relayed? | **Machine-local (hub-B emits for hub-B inbox). No cross-hub relay needed.** |

---

## Next steps

**TermLink side (if AEF concurs):**
1. Create build task: add `inbox.queued` event class + hub emission in inbox
   delivery path + integration test. Tag: `arc:peer-consult`.
2. Coordinate with T-243: ensure `dialog.*` turns also trigger `inbox.queued`
   so the wakeup primitive covers both plain channel posts and dialog rounds.

**AEF side (per PROP-T-1804, not TermLink's concern):**
- Record joint decision in ADR-0004
- Build `fw consult` semantic layer + subscriber bridge + audit channel
- No TermLink change blocked — `event.subscribe` with the new event class is
  the only dependency

**Coordination:** AEF may proceed with subscriber bridge design immediately
(the `inbox.queued` event class contract is agreed here). TermLink build task
ships independently; AEF bridge can use `inbox list --poll` as fallback until
`inbox.queued` is live.
