# arc-011 — Agent-to-agent message delivery: mailbox to prompt

**Status:** draft · **Anchor:** T-3069 · **Created:** 2026-09-22

> An agent sends a message (with an optional binary blob) to a peer and learns, as
> two separate confirmed events, that it was **RECEIVED** into the peer's local
> store and later **INJECTED** into the peer's prompt with the peer verified as
> working on it — without either side needing to be continuously attached.

---

## Why this document exists

On 2026-09-22 I reported the rail "working end to end". It was not. Transport,
storage, the flag and a delivery receipt worked; the **queue, the prompt-free
check, the injection and the verification did not exist at all**. The report was
accurate in its details and false in its headline, which is worse than being
wrong, because the details made it credible.

The correction is not a better summary. It is this document plus a slice register
in `arc-011.yaml`, so "what is built" is a field someone can query rather than a
claim someone makes.

**The rule this arc is run under:** a slice is `built` only when the specified
**mechanism** carries it. A slice where something real happens by another route is
`partial`, never `built`. S5 is the worked example — the RECEIVED event is durable
and correct and rides the *hub*, not the sidecar API the spec calls for.

---

## The sequence, as specified by the operator

```
  SENDER                          RECEIVER
    |                                |
    |-- 1. send msg (+blob) -------->|   via a SIDECAR API (S1, S2)
    |                                |
    |                          2/3. store on local disk (S3)
    |                          4.   set flag (S4)
    |                                |
    |<-- 5. RECEIVED ----------------|   same sidecar API (S5)
    |                                |
  6. record in own ledger (S6)       |
    |                                |
    |                          7. cron: is the prompt free? (S7)
    |                             - independent of any LLM
    |                             - recovery path if the agent is NOT running
    |                                |
    |                          8. read queue, take highest priority (S8)
    |                          9. if urgent: inject immediately (S9)
    |                          10. INJECT + verify the agent is working (S10)
    |                                |
    |<-- 10. INJECTED ---------------|   same sidecar API
    |                                |
    |                          (agent works)
    |                                |
    |<== 12. reply: roles swap ======|   receiver becomes sender (S12)
```

Step 11 (the cron) drives 7→10 and is the reason none of this needs an LLM to be
online.

---

## Two events, not one

The distinction the whole arc rests on:

| event | means | may be claimed when |
|---|---|---|
| **RECEIVED** (L2 `stage=delivered`) | the bytes are in the peer's local store and will survive its restart | the journal row and the receipt exist |
| **INJECTED** (L3 `stage=read`) | the message is in front of the agent and the agent is working on it | the injector observed a free prompt, injected, and **verified work started** |

**Why the second is hard, and why it is the point.** A bare `termlink inject`
returns *before submission*: on a busy or manual-accept session the text lands
unsubmitted in the composer and is discarded on the next `--continue` (T-2396,
proven live). So "I injected it" is not evidence that anyone read it. `INJECTED`
requires an observation, which is why `notify-ack-read.sh` demands
`--evidence <kind>` and has no kind meaning "I sent it and hoped".

`evidence=wake-consumer` was accepted for exactly one day and has been **removed**:
a consumer that notices a flag has injected nothing, so a receipt carrying it
asserted a falsehood. Offset 110 on `dm:3bba15e681b3a078:d1993c2c3ec44c94` is such
a receipt and should be read as *noticing*, not reading.

---

## What exists today, honestly

| # | Slice | Status | Reality |
|---|---|---|---|
| S1 | sidecar API | **unbuilt** | sends ride hub RPC; the sidecar is a *poller* with no API |
| S2 | binary blob | **unbuilt** | TermLink has transport, it does not fit (below) |
| S3 | local store | **built** | `journal.sqlite`, 2,238 rows measured |
| S4 | flag | **built** | plus arrival memory surviving the ack |
| S5 | RECEIVED | **partial** | real + durable, **wrong mechanism** (hub, not API) |
| S6 | sender ledger | **unbuilt** | sender can only poll; a restart loses its waits |
| S7 | prompt-free check | **unbuilt** | classifier built (T-2402 Stage 3), wired to nothing |
| S8 | queue + priority | **unbuilt** | flat FIFO first, per operator |
| S9 | urgent flag | **unbuilt** | semantics undefined |
| S10 | inject + verify | **unbuilt** | **the key gap** |
| S11 | cron driver | **unbuilt** | crontab written, **not installed** |
| S12 | role swap | **unbuilt** | AEF agent cannot send (allowlist) |

**2 built · 1 partial · 9 unbuilt.**

---

## Open design questions for the operator

### Q1 — the sidecar API (S1, blocks S5/S6/S12)

Operator direction, verbatim in intent: the sidecar is a **separate process**
exposing an API, deliberately **independent of the hub because the hub goes down**.
Very simple. **Always respawns.** It also carries the startup chain — start agent,
start session, start project, start hub — and re-resolves when an **FQDN or IP
stops resolving**.

Open, and genuinely not mine to decide:

1. **Transport.** Unix socket (local only, simplest, no auth story) vs loopback
   TCP vs LAN TCP (needed for cross-host, needs auth).
2. **Does it replace the hub for this rail, or front it?** If the sidecar API is
   the delivery path, the hub becomes a fallback rather than the primary — that is
   a significant change to what "delivery" means here.
3. **Who respawns the respawner?** systemd is the obvious answer on .107; it is
   not the answer on a host without it.
4. **Does the startup chain belong in this process or beside it?** It is a
   different responsibility (bootstrap/supervision) that happens to need the same
   liveness knowledge.

### Q2 — blob transport (S2)

Investigated 2026-09-22; TermLink's blob transport does **not** drop in:

- `file send <TARGET> <PATH>` targets a **session**, not a `dm:` topic.
- `file receive` **waits for `file.init` events** — the receiver must be *actively
  listening*, which is precisely the continuous-attachment problem the flag exists
  to remove.
- **PL-095**: send spools chunks to the **sender's** hub inbox; receive pulls from
  the *source* hub. Cross-host needs symmetric peer config, not one-way.

So the question is how to bridge a **streaming, session-addressed** transport onto
a **store-and-forward, mailbox-addressed** rail. Options: spool the blob to the
receiver's disk and put a *reference* on the topic; or give the sidecar API a blob
endpoint and skip `file send` for this path. `--expected-sha256` (T-2472) exists
and should be mandatory on whatever lands.

### Q3 — priority (S8) and urgency (S9)

Flat FIFO for now, per operator. Open: where does priority live — envelope
metadata, or a queue-side field? And what exactly does "urgent" bypass: the *wait*
for a free prompt, or the *free-prompt check itself*? Those are very different
risk profiles, and the second can drop a message into a busy session where it is
silently discarded.

---

## Build order, and why

```
  S7 + S10  (T-3069)   the injector          <- START HERE
     |                 queue -> prompt-free -> inject -> verify -> L3
     |                 makes the rail DO something; unblocks honest L3
     v
  S11       (T-3068)   install the cron      <- makes it survive a reboot
     |
     v
  S1        (T-3070)   sidecar API           <- needs Q1 answered first
     |                 converts S5 partial -> built, unblocks S6/S12
     v
  S6 (T-3072) · S2 (T-3071) · S8 (T-3073) · S9 (T-3074)
```

**T-3069 first** because everything before it is plumbing that delivers to nobody,
and because the honesty of L3 depends on it: until an injector exists, the rung has
no truthful caller.

**S1 is deliberately not first**, even though it is spec step 1, because it needs
an operator decision (Q1) and because the injector can be built and proven against
the existing transport, then re-pointed at the API when it lands.

---

## How this arc may be closed

It may **not** be closed on a narrative. `arc-011.yaml` carries a `prover:` binding
that `scripts/check-arc-claim-drift.sh` executes, so the headline claim is
re-checked rather than asserted once. Closing requires:

1. every slice `built` (or explicitly descoped with a reason in the register),
2. the prover exiting 0, and
3. **a live run observed end to end** — a real message reaching a real prompt, with
   the agent verified working, read back from the hub rather than inferred.

Condition 3 is the one that was skipped. It is written down here so it cannot be
skipped quietly again.

---

## Guard rails learned the hard way (do not relearn these)

- **Never post content to a topic you are watching.** A note counts as unread mail,
  so it re-raises the flag you are watching. Two consumers on one topic amplified
  each other into ~100 envelopes in about two minutes on a live topic. Receipts are
  safe: the unread watermark excludes meta types, so a receipt cannot wake anyone,
  including itself.
- **A daemon is not deployed until it restarts.** A sidecar running pre-change code
  emitted no `last_mail_ts` for 120 seconds of confused debugging while the process
  was alive, the heartbeat fresh, and the flag well-formed — just an older *shape*
  (T-2405 class).
- **`pending` is a count, not an offset.** Passing it as a watermark acks nonsense.
- **Identity is declared, never derived from a flag-file label.** Deriving it minted
  a third fingerprint unrelated to the mailbox being served.
- **`pkill -f <pattern>` matches the shell whose command line contains the
  pattern** — including your own. Kill by recorded PID.
- **A guard that hardcodes a status will keep stating it after it stops being
  true.** The WAKE verdict claimed `L3=NOT-IMPLEMENTED` for two commits after L3
  shipped. It detects now.

---

## References

- `.context/arcs/arc-011.yaml` — slice register (source of truth for status)
- `docs/operations/deterministic-notify-sidecar.md` — L2/L3/C ladder
- `scripts/notify-rail-e2e.sh` — the prover; `--stages precond,deliver,receipt,ladder,wake`
- `scripts/notify-ack-read.sh` — L3 with the evidence gate
- `scripts/check-arc-claim-drift.sh` — runs this arc's prover
- T-2396 (inject returns before submission) · T-2402 Stage 3 (idle classifier) ·
  PL-095 (blob direction) · T-2405 (stale daemon code) · T-2472 (`--expected-sha256`)
