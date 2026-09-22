# arc-011 Q1 — The sidecar API: architectural analysis

**Task:** T-3075 (inception) · **Arc:** arc-011 · **2026-09-22**
**Status:** analysis for operator decision. No build is authorised by this document.

---

## The proposal

> The sidecar must expose an **API**, as a **separate process**, deliberately
> **independent of the hub because the hub goes down**. Very simple. **Always
> respawns.** It also carries the startup chain — start agent, start session, start
> project, start hub — and re-resolves when an FQDN or IP stops resolving.

---

## 1. The charter collision, stated plainly

TermLink's canonical purpose sentence is human-blessed and names the mechanism:

> **TermLink is a hub-mediated, durable append-log message bus with terminal
> endpoints…**

And the charter's own gloss on that noun:

> **Hub-mediated** — a strict star: spokes never talk peer-to-peer, **the hub
> mediates all coordination**.

This is not a preference. It is enforced in code by
`crates/termlink-hub/tests/no_federation_tripwire.rs` (T-2569): the hub crate may
not read peer-hub config and may not build a hub-speaking client.

**So "an API independent of the hub" collides with the single load-bearing noun of
the project's purpose — if, and only if, that API carries coordination between
agents.** That conditional is the whole analysis.

---

## 2. Strawman

*"A sidecar API is a second message path that bypasses the hub. It makes spokes
talk to each other directly, which is exactly what the charter forbids and a
tripwire test guards. It re-implements delivery, auth, retry and dedupe — all of
which the hub already does — and it does so in bash, in a process whose stated
virtue is being simple. Two delivery paths mean two truths about what was
delivered, and the rail's entire recent history is about the cost of not knowing
which claim to believe. Reject it; fix the hub's availability instead."*

**Why the strawman is not stupid:** every clause is true *of the peer-to-peer
reading*. Two delivery paths really would be a disaster here, and "the hub goes
down" really is an availability problem that has an availability answer.

**Why it fails:** it assumes the API carries *delivery*. The operator's spec does
not say that. Steps 7–10 — check the prompt is free, read the queue, inject, verify
the agent is working — are **entirely local to the receiving host**. None of them
is a spoke talking to another spoke.

---

## 3. Steelman

*"The hub is a **coordination** substrate and should stay one. But the last 200
metres — from 'a message is durably in my local journal' to 'my agent is working on
it' — is not coordination. It is local control of one host's own prompt, and it
must keep working when the hub is unreachable, because that is precisely when an
operator most needs to reach an agent.*

*Today that last leg is a poller with no interface: nothing can ask it anything,
nothing can tell it anything, and its liveness is inferred from a heartbeat file. A
small local API turns an opaque process into an addressable one: 'what is in my
queue?', 'inject this now', 'are you alive?', 'is my agent free?'. None of those
questions crosses a host boundary, so the strict star is untouched.*

*The always-respawning property is not gold-plating either. Everything else in this
rail is supervised by something; the supervisor of last resort is currently cron,
which is a 5-minute hole. A process whose only job is to be alive can be supervised
harder than one that does real work.*

*And the startup chain belongs with whatever holds that liveness knowledge. Knowing
'the hub is down, the FQDN no longer resolves, the agent session is gone' is the
same knowledge needed to answer 'can I inject right now?'."*

---

## 4. Against the Four Constitutional Directives

Evaluated on the **local-control** reading (the steelman), since the peer-to-peer
reading is refused by the charter regardless of merit.

| Directive | Weight | Verdict |
|---|---|---|
| **D1 Antifragility** | 9 | **Strongly for.** Today a hub outage silently stops the whole rail: mail stops arriving, so no flag rises, so nothing wakes. A local API that keeps serving the *local* journal degrades instead of stopping — the system gets weaker gracefully rather than going dark. The rail's own history (82 days dark, and my own feedback loop) is the argument that stress currently produces silence, not signal. |
| **D2 Reliability** | 7 | **For, with one sharp caveat.** "No silent failures" improves: an addressable process can be *asked* whether it is working, instead of having liveness inferred from a file's mtime. **The caveat is the danger**: if the API ever becomes a second *delivery* path, we get two truths about what was delivered, and this project has already paid for ambiguous delivery claims twice this week. Reliability argues **for** a local control API and **hard against** a parallel delivery path. |
| **D3 Usability** | 5 | **For.** "Actionable errors" is currently impossible: when nothing wakes, an operator cannot ask *why*. The diagnostic ladder is `cat` a flag file and guess. An API can answer "your agent is BUSY", "your queue has 3 items", "the hub has been unreachable for 40 minutes". |
| **D4 Portability** | 3 | **Mildly against.** A Unix-socket API is portable; a systemd-supervised always-respawning process is not. macOS has no systemd. This is the lowest-weighted directive, so a systemd-first answer is *defensible* — but per the charter it should be an explicit decision with a stated degradation, not an accident. |

**Value drivers** (protected weights): D1=9, D2=7, F-RECALL=6, D3=5, D4=3. The
proposal scores strongly on the two heaviest and weakly negative on the lightest.
On weighted drivers alone it is a clear go — *for the local-control reading*.

---

## 5. TermLink's objective vs AEF's objective — the real question

This matters more than Q1's transport details, and it is **IW-2**.

**Charter non-goal 4:**

> **Not a workflow or orchestration engine.** TermLink provides the primitives.

Now look at what steps 7–10 actually are:

- read a **queue**
- apply a **priority policy**
- decide **when** a worker is ready
- **dispatch** work into it
- **verify** the worker accepted it

That is an orchestration engine. Written into TermLink, it is a charter violation
by the plain text of non-goal 4 — and non-goals in this project are enforced
(`check-charter-drift.sh` deletes off-charter surface; P4 pruned 52 tools for less).

| | TermLink's job | AEF's job |
|---|---|---|
| Durable topics, offsets, acks, replay | ✅ has it | — |
| Blob transport (`artifact.put`) | ✅ has it | — |
| PTY inject / exec / doorbell | ✅ has it | — |
| Presence, READY/BUSY classification | ✅ has it | — |
| **Queue with a priority policy** | ❌ non-goal 4 | ✅ |
| **Deciding when to inject** | ❌ non-goal 4 | ✅ |
| **Verifying the agent took the work** | ❌ non-goal 4 | ✅ |
| **The task/work model it all serves** | — | ✅ |

**Reading:** TermLink already supplies *every primitive this rail needs*. What is
missing is the orchestration on top — and that is AEF's stated purpose, not
TermLink's. The sidecar is the component that sits on the boundary and consumes
TermLink primitives to serve AEF's model.

If that reading is right, then:

- the injector (T-3069) is **AEF work**, not TermLink work;
- the sidecar API's correct scope is **local control**, which does not collide with
  the strict star at all;
- and the reason the hub-independence requirement *felt* like a charter fight is
  that we were implicitly proposing to build an orchestrator inside a bus.

**This is a genuine finding, and it is the operator's to confirm or reject.** I am
not confident enough to act on it: it would move a task between repositories, and
the charter is human-blessed territory.

---

## 6. What the API should and should not carry

If the local-control reading is accepted:

**In scope (local, host-bounded — no charter tension)**
- `status` — am I alive, when did I last cycle, is the hub reachable from here
- `queue` — what is pending for this agent, in order
- `inject <id>` — put this in front of my agent now (urgent path, §8)
- `agent-state` — READY / BUSY / UNKNOWN / NOT-RUNNING
- `ack <offset> --evidence <kind>` — post L2/L3 through the one existing path

**Explicitly out of scope (would break the strict star)**
- sending a message to a *peer* — that is `channel.post`, and it goes to the hub
- any store-and-forward for *other* hosts
- anything that makes this host reachable *as a bus* by another host

**The bright line:** the API may read and act on **this host's own state**. The
moment it moves a message *between* hosts, it has become a second bus and the
charter refuses it. A tripwire test should encode that, mirroring
`no_federation_tripwire.rs`.

---

## 7. Answer to Q2 — how blobs work in TermLink today (**corrects my earlier answer**)

I previously reported that TermLink's blob transport "does not drop in". **That was
wrong** — I read the CLI surface and the legacy path and stopped.

Measured in the source:

- **The modern path (T-1249)** is `artifact.put` + `channel.post`, with
  `channel.post { msg_type: "artifact", artifact_ref: <sha256> }` (T-1164a). The
  envelope carries an `artifact_ref` and the blob is **content-addressed** by
  sha256. *This is blob-on-a-topic, and it already exists.*
- **The legacy path** is the 3-phase `file.init` → chunks → `file.complete` event
  emit, session-targeted. `file.rs` uses it **only as a fallback** when the peer hub
  does not advertise `artifact.put`.
- `--expected-sha256` (T-2472) gives *independent* receiver-side verification, and
  without it the output deliberately labels the digest by provenance rather than
  claiming "verified".

**So the rail does not need a new blob mechanism.** It needs to (a) use the
artifact path rather than `file send`, (b) decide when the receiver fetches the
blob — eagerly on arrival, or lazily at injection — and (c) make
`--expected-sha256` mandatory on that fetch. That is T-3076, and it is much smaller
than I made it sound.

PL-095 (send spools to the sender's hub inbox) applies to the **legacy** path. It
should be re-measured against the artifact path rather than assumed.

---

## 8. Answer to Q3 — priority and urgency

Operator: *"Urgent bypasses the wait. Reliability is important at all times but can
also be out of band."*

Taking that precisely, it resolves a confusion in my earlier framing. I had asked
whether urgent bypasses *the wait* or *the check*. The answer is neither-exactly:

- **Urgent bypasses the WAIT** — it does not sit in the queue behind FIFO.
- **Urgent may go OUT OF BAND** — a different delivery route (the PTY doorbell,
  which already exists and is un-armed) rather than the queue path.
- **Reliability is not traded away.** Out-of-band still means *confirmed*: it must
  still produce an L3 with real evidence. Out-of-band changes the **route**, not
  the **proof**.

That gives two lanes with one confirmation contract:

| | normal | urgent |
|---|---|---|
| route | queue → FIFO → inject when free | doorbell / direct inject |
| waits for a free prompt? | yes | no |
| confirmation | L3, evidence=idle-gated-inject | L3, **still required**, evidence names the route |
| if the prompt is busy | wait | inject anyway — **and this is the risk** |

**The risk to decide (not mine):** injecting into a busy session is exactly the
T-2396 failure — text lands unsubmitted and is discarded. So an urgent injection
into a BUSY prompt can be *silently lost*, which is the worst outcome for the class
of message most likely to be marked urgent. Options: refuse-and-escalate, retry on
next idle, or accept the loss and say so loudly. **This needs a decision before
T-3072 is built**, and flat FIFO (T-3071) is unaffected by it.

---

## 9. Recommendation

**GO on the analysis; NO BUILD until IW-1/IW-2 are answered by the operator.**

1. **Confirm or reject the AEF/TermLink split (IW-2).** Highest leverage question
   here. If the injector is AEF work, several tasks move repository, and the
   charter tension mostly dissolves.
2. **If the API is local-control-only**, it is consistent with the charter and
   scores strongly on D1/D2/D3. Build it after T-3069, not before — the injector
   can be proven against existing plumbing and re-pointed later.
3. **Encode the bright line as a tripwire test** the way federation is, so "local
   control only" is enforced rather than remembered.
4. **T-3076 shrinks**: adopt `artifact.put`, do not invent blob transport.
5. **Decide the urgent-into-busy behaviour** before T-3072.

**What would change my mind:** if the real requirement is cross-host delivery while
the hub is down, then this is not a local-control API at all, it is a second bus,
and the honest move is to say so and put it to the charter — not to build it under
a name that sounds local.

---

## Open questions carried on T-3075

- **IW-1** — does the API violate the charter? *Depends on what it carries; the
  local-control reading does not.*
- **IW-2** — TermLink or AEF? *Evidence points to AEF for the orchestration half.*
- **IW-3** — what failure does hub-independence actually buy, given local state is
  already local?
- **IW-4** — who respawns the respawner, portably?
