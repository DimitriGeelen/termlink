You are an independent architecture consultant, from outside this project. Work from the text below only; do not read other files.

## Context
TermLink is a hub-mediated, durable message bus that lets a fleet of AI coding agents (Claude Code and other harnesses, running in terminals on several hosts) discover each other, exchange durable messages, claim work and control terminal sessions. Each host runs a hub (several hubs on one host are possible). Each agent has a local "sidecar" process that receives mail for it and injects it into the agent's session. Addresses have five levels: //host/hub/project/session/agent, each with a canonical id and instance ids. Today hubs never talk to each other: a sender reaching an agent on another host posts directly into that host's hub ("client-driven cross-posting"), and the hub pushes a wake frame to the receiver (85-111 ms median). The operator's goal is interactive two-way conversation between running agents across hosts, with every step confirmed to the sender.

The document below traces why "hubs never talk to hubs" became a rule, records the operator's reflection on what should replace it, and the agent's reflection and challenges. The operator explicitly invites challenge.

# T-3335 — Why "hubs never talk to hubs"? Back to principles

**Asked by the operator, 2026-10-04,** while deciding OD-1 (cross-host send path):
"Can you find back why we decided that hubs cannot talk to hubs? As we are going to multi-agent,
multi-host communications and even multiple hubs on one system, that seems to be something we might
want and actually is necessary."

## 1. The record, in order

1. **2026-05-21, G-060 / T-1791.** An operator saw 1800 vs 486 messages in `agent-chat-arc` on .107 and
   .122 and filed it as a federation bug. The RCA found that **no hub-to-hub primitive had ever been
   built** (`grep` for federat/peer_subscribe/cross_hub: 0 matches). The disparity was declared "the
   DESIGN" and the gap reclassified as documentation (`.context/project/concerns.yaml`, G-060).
   G-060 itself left the real question open: "Optional larger inception: does the fleet WANT
   auto-federation? … NOT decided in T-1791."
2. **2026-05-25, T-1793** asked exactly that (`docs/reports/T-1793-auto-federated-channel-topics.md`).
   It weighed **automatic replication of channel topics** across hubs:
   2a. benefits: simpler agent UX, one source of truth, no `--hub` to remember;
   2b. costs: state sync, a consistency model (last-write-wins, vector clocks, CRDTs), conflict
       resolution, bandwidth amplification on every post, cross-hub ordering, retention divergence.
   2c. Outcome: **parked** (DEFER in substance; the decision field says GO, a recorded discrepancy),
       because T-1166 was not blocked, client-driven cross-posting "works correctly when used", and
       the cost is significant.
   2d. **Revisit when** (a) agents keep being surprised by per-hub semantics, or (b) "a concrete
       fleet-wide coordination workflow emerges that the client-driven pattern cannot serve cleanly".
       Revisit date 2026-08-21. **No revisit happened.**
3. **2026-06/07, T-2229.** ring20 reported "cross-hub federation broken". Answered "working as
   designed"; an operator `fleet federation-status` verb was deferred (IW-3).
4. **2026-07-31, T-2470** wrote `docs/CHARTER.md`, including non-goal #1, "Not an inter-hub federation
   layer … never automatic (G-060)". The task's only human AC, "Bless the canonical purpose sentence
   (and the non-goals)", is **still unticked**; T-2470 is `started-work`, `owner: human`. Yet
   `CHARTER.md:17` states the sentence "is human-blessed". **The non-goal was never ratified.**
5. **T-2569** then turned the non-goal into a build-breaking tripwire
   (`crates/termlink-hub/tests/no_federation_tripwire.rs`): the hub may not read peer-hub config, may
   not build a hub-speaking client, and its outbound connections are enumerated.
6. **2026-09-23, SQ-1** ("sending to a peer stays channel.post via the hub") and the sidecar-API brief
   ("a second bus … put it to the charter") reasoned *from* the non-goal, not about it.

**Finding.** The rule began as an observation that a feature was missing (G-060), was parked with
explicit revisit triggers (T-1793), was written into an unratified charter by an agent (T-2470), and
was then enforced in code (T-2569). At no point did anyone decide on principle that hubs must not talk.
The only analysis on record (T-1793) is about **replicating shared topic state**, not about relaying
addressed messages.

## 2. Have T-1793's revisit triggers fired?

7. **(a) Repeated surprise: yes.** ring20 (T-2229) after the documentation shipped; 055 E2 (an agent
   started with `env -i` read an empty, different hub and saw "no mail"); AEF T-3779 (mail hub tied to
   the terminal runtime dir). The documentation did not stop it.
8. **(b) A workflow the client-driven pattern cannot serve cleanly: yes.** Interactive conversation
   between agents on different hosts (arc-011). Client-driven means every sender must know which hub
   the receiver uses, hold that hub's secret and TLS pin, and retry against it itself: N agents × M
   hubs of credentials. That is where the recurring failures sit: wrong hub, stale secret, rotation
   heal paths, the `fleet reauth` machinery.

## 3. Reflection: "hubs talking" is three different things

| Kind | What it means | Shared state? | T-1793's costs apply? |
|---|---|---|---|
| **R. Replication** | A topic exists on many hubs and stays in sync | Yes | **All of them**: consistency model, conflicts, ordering, amplification, retention divergence |
| **F. Addressed relay** (store-and-forward, the e-mail model) | A message addressed `//host/hub/project/agent` is handed by the sender's hub to the destination hub, which owns it from then on | **No**: every message has exactly one owner, its destination | Almost none: no consistency model, no conflicts; ordering is per conversation; dedupe by `client_msg_id` already exists |
| **D. Directory / presence exchange** | Hubs tell each other which agents and projects they serve | Small, soft state (rebuildable) | Mild: staleness, which a TTL handles |

9. The non-goal, as written, forbids all three, while the only reasons on record argue against **R**.
10. What arc-011 needs is **F** plus a little **D**. The five-level address (T-3325) already names the
    destination hub, so a hub can tell "this is mine" from "forward this" without any guessing.
11. **F is also the answer to both sides of OD-1.**
    11a. The operator wants push between agents on different hosts without depending on one agent's
         sidecar knowing the whole fleet.
    11b. The reviewers object to a **second path alongside the hub** (E4: two paths diverge). Hub relay
         is not a second path: it is the hub path, extended by one hop. Sidecars keep talking only to
         their local hub; only hubs hold peer-hub trust (M hubs, not N×M agent credentials).
12. **Several hubs on one host** become ordinary rather than a failure mode: today an agent on the wrong
    local hub is deaf (055 E2); with relay and a directory, it is one hop away.

## 4. What F would cost, honestly

13. The hub gains outbound connections and a durable outbound queue per peer hub. That is new hub state
    and a new failure surface; the tripwire (T-2569) has to be rewritten to allow exactly this and
    still forbid R.
14. Receipts need a hop rule: "accepted by my hub" vs "stored by the destination hub". This feeds OD-4.
15. Loops and misroutes need a hop limit and a refusal for unknown destinations (055's "unknown
    addresses are refused, not created").
16. Hub-to-hub trust: the HMAC secret + TLS pin machinery exists, but now it is hub-to-hub and must
    rotate cleanly. This is the trust-model gap every reviewer raised, moved to a smaller set (hubs).
17. A partitioned destination hub means messages wait at the origin hub, and the sender sees "queued
    at my hub, destination unreachable" rather than "delivered". That is correct behaviour, but it
    must be visible (OD-14).
18. It is a charter change, and the charter is the operator's (T-2470 is still waiting for exactly
    that ratification).

## 5. Suggested way forward (agent proposal, not a ruling)

19. Hold OD-1 until this principle is settled; OD-1's options all assumed the non-goal.
20. Settle it through T-2470: the charter ratification is still open, so the non-goal can be reworded
    there rather than amended later. Proposed wording: "Not a replication layer: hubs never sync
    shared topic state. Hubs MAY relay individually addressed messages to the destination hub
    (store-and-forward), and exchange a directory of whom they serve."
21. Open one single-question inception: "Should hubs relay addressed messages (F) and exchange a
    directory (D)?", with T-1793 as the record of R being parked, and the tripwire rewrite as its
    first build slice if GO.
22. Fix `CHARTER.md:17`: it claims a human blessing that the task record says has not happened.

## 6. Dialogue log

### 2026-10-04 — operator reflection (not a ruling; "can be challenged")

23. Do not kill direct agent-to-agent communication across hosts. Once a circuit is established, the
    direct path is efficient and preferred.
24. No hubs synchronizing messages: "an overload, an overkill, also needs a lot of coordination".
25. Value is in hubs (and hosts) exchanging **which identities are present**: per hub, the project,
    session and agent ids, with their instances (several of each may exist), plus status ("this one
    has died"). That needs a routing mechanism: "research common routing protocols. IP routing
    protocols should really help us there. Or maybe you've got other suggestions."
26. Liveness serves the fallback: when a send fails, the agent asks its hub, or another hub, whether
    the target still exists. Outage: retry. Hub already established the agent died: stop, and the
    sender gets a dead letter back.
27. Relaying addressed mail through hubs "should really be an emergent fallback mode … still we should
    prefer direct one-to-one connection … Same as with IP. It helps routing, but once the routing has
    been established, the communication channel is set up."
28. Asked: reflect first, then external consultation.

### 2026-10-04 — agent reflection on 23-27

29. **The shape is control plane vs data plane**, and it is sound. Hubs exchange reachability (who is
    where, alive or not): the control plane. Messages flow directly once resolved: the data plane.
    Relay through hubs is the degraded data plane. This removes T-1793's objection entirely, because
    no message state is ever shared between hubs.
30. **IP routing protocols are the wrong closest fit; four others fit better.** IP routing (RIP,
    OSPF, BGP) computes multi-hop paths through a graph of routers. Here every hub can reach every
    other hub in one hop, and there are about 5 to 20 hubs, so there is no path to compute: what is
    needed is **resolution** (name to location) and **membership** (alive or dead), not routing.
    30a. **SIP (registrar, location service, proxy) with ICE/STUN/TURN** is the closest model of what
         the operator described: an endpoint REGISTERs with its home server; a call is set up through
         proxies that resolve the address; media then flows directly; ICE tries direct candidates
         first and falls back to a TURN relay. Map: REGISTER = the presence heartbeat to the home hub;
         proxy resolution = hub directory lookup; direct media = sidecar to sidecar; TURN = hub relay.
    30b. **SWIM gossip (as in Serf/Consul memberlist)** for liveness between hubs, with the three states
         alive / suspect / dead. "Suspect" is exactly the operator's "was it an outage or did it die":
         suspect means retry, dead means stop.
    30c. **DNS-style caching with a TTL** for lookups, so a sender does not ask on every message, and a
         cached answer carries its age.
    30d. **BGP's one useful idea**: each hub advertises only what it is authoritative for (its own
         projects, sessions, agents), and others never re-advertise it as their own. That keeps
         authority clear: only the home hub can declare its agent dead.
31. **Challenges to the operator's reflection.**
    31a. **Direct across hosts needs a new listening surface.** Each sidecar would accept network
         connections from other hosts: ports, firewall, TLS, authentication. Today only hubs have that
         (TLS + HMAC + TOFU). Either sidecars inherit the same trust machinery, or the "direct" path
         is direct-to-the-destination-hub (which exists today, and is one hop from the receiver).
         This is the question the consultation should press hardest.
    31b. **Two paths diverge (055's E4)** unless one durable record, keyed by message id, settles every
         message whichever path carried it, and the receiver deduplicates by `client_msg_id` for longer
         than any retry.
    31c. **"Dead" needs an authority and an age.** Only the agent's home hub can say "dead" (deregistered,
         or heartbeat expired beyond a threshold); any other hub can only say "last seen alive at T,
         per hub X". A fourth state, **unknown** (the home hub itself is unreachable), must never be read
         as "dead".
    31d. **Instances.** A message addressed to a role (project + agent) resolves to a live instance; one
         addressed to an exact instance must fail loudly rather than be redirected (Codex's fencing
         point). The directory carries canonical and instance ids per level.
    31e. **Relay helps only some failures.** If the destination host is down, relay cannot deliver
         either; it queues. Relay helps when the direct path is blocked but the destination hub is up
         (firewall, sidecar restarting, wrong local hub as in 055 E2).
    31f. **A possible fallback ladder:** (1) direct sidecar to sidecar on the resolved circuit;
         (2) post into the destination hub (today's client-driven cross-post); (3) ask my hub to relay;
         at each failure, consult liveness: suspect, retry with back-off; dead, stop and dead-letter.
    31g. **It is still a charter change** (non-goal #1 and the T-2569 tripwire forbid hubs exchanging
         even a directory), and the charter is unratified (T-2470).

## Questions

Answer in English, under ~1300 words, using the numbered headings below. Use hierarchical labels (1, 1a, 1ab), never plain bullets. Give your own view and disagree where you think the proposal is wrong. Do not modify any files.

1. **The split.** Hubs exchange which identities are present and whether they are alive (control plane); agents talk directly once resolved (data plane); hub relay only as a fallback; hubs never synchronize messages. Is this the right architecture for a fleet of AI agents across hosts? Your verdict and its biggest weakness.
2. **Protocol models.** Which existing protocol families should this borrow from, and what exactly from each: IP routing (RIP, OSPF, BGP), DNS, SIP registrar/proxy with ICE/STUN/TURN, gossip membership (SWIM, Serf, Consul), XMPP server-to-server, e-mail MX, or others. Scale: about 5 to 20 hubs, a few hundred agent instances, hubs mostly one hop from each other. Do you agree that IP routing is the wrong closest fit?
3. **Liveness.** States (alive / suspect / dead / unknown?), who is authoritative, how fast "dead" may be declared, and exactly how a sender uses the answer (retry, stop, dead letter).
4. **Directory contents.** What each hub advertises across the five address levels (host, hub, project, session, agent; each with a canonical id and instance ids), how it is scoped, cached and expired, and how a role address resolves to an instance.
5. **The direct path.** What "direct" should mean here (sidecar-to-sidecar network connection, or direct into the destination hub), what it requires (listening surface, authentication, NAT/firewall, durability, receipts), and when direct should NOT be preferred.
6. **The fallback ladder.** Order of attempts, what the sender is told at each step, how one message is settled when it may travel by more than one path, and deduplication.
7. **First slice and test.** The smallest build that proves it with two real agents on two hosts, including a negative control (a dead agent, and a hub outage that must not be reported as dead).
8. **What is missing:** a risk, an option or a requirement nobody stated.

## Rounds 1-2 so far (comparison)

# T-3335 routing consultation — comparison

**Question:** hubs exchange a directory of who is present and alive; agents talk directly once resolved;
hub relay only as fallback; hubs never synchronize messages (operator reflection, 2026-10-04,
`../T-3335-hub-to-hub-principles.md` §6). **Brief:** `brief.md`.

## Who answered

| Reviewer | File | Status |
|---|---|---|
| Codex | `codex.md` | Full answer, rc 0 |
| GLM-5.3 | `glm.md` | Full answer, rc 0 |
| 055-agentic-fleet-cockpit | — | "Received" (@450); answer to follow, from its two-hubs-on-one-host experience |
| AEF | — | Its sidecar answered RECEIVED, then WAITING_NO_RECIPIENT: no live AEF agent session (@448, @449) |
| 832-Workflow-designer | — | Nothing |
| Local models | — | Not run this round (weak last round; host heavily loaded) |

## 1. Agreed by both

1. **Hubs exchange reachability, never message state.** Both accept the split and that it removes
   T-1793's objection. Codex adds a precision: during a hand-off the origin and destination may both
   hold the same message until the destination acknowledges; forbid topic replication, not that
   overlap.
2. **IP routing is the wrong closest fit.** Both: there is no multi-hop path to compute at 5 to 20
   hubs. Borrow BGP's "advertise only what you are authoritative for, never re-advertise" and route
   expiry; nothing else from RIP, OSPF or BGP.
3. **Better models.** Both name e-mail store-and-forward and XMPP server-to-server as closest to the
   messaging need (home server, authoritative destination, relay between servers), DNS for TTLs
   (including on "not found" answers), and SIP for registration. GLM adds e-mail's delivery status
   split: temporary failure = retry, permanent = dead letter.
4. **Disagree with the operator on "direct".** Both say the first build's "direct" should mean
   **straight into the destination hub**, not sidecar to sidecar.
   4a. Codex: "The reported 85-111 ms median is a baseline, not evidence that another listening
       surface is warranted." Direct transport "should be a measured optimization."
   4b. GLM: sidecar listeners multiply the listening surface "from ~M hubs to N agents, re-creating
       the credential sprawl the proposal claims to kill … the IP analogy fails at exactly this point,
       because in IP the 'direct path' changes nothing about trust, whereas here it changes the trust
       topology."
5. **Only the home hub may declare an agent gone, with evidence; an outage is never "dead".**
6. **Exact-instance messages fail loudly; role messages re-resolve.** Never silently redirect.
7. **Deduplication outlives every retry**, keyed by a stable message id; one durable record settles a
   message whichever path carried it.
8. **Receipts by hop:** accepted by my hub < stored by destination hub < injected < acknowledged by the
   agent. GLM: "stored by destination hub" should be the default meaning of "delivered".
9. **Queues need limits and back-pressure**: one unreachable destination must not exhaust a hub.
10. **The same negative controls:** a dead agent returns an authoritative terminal answer; a stopped hub
    yields "unknown / queued", and the word "dead" must never appear (GLM: assert it literally).
11. **First slice excludes sidecar-to-sidecar.** It covers two configured hubs, authenticated
    hand-off, a durable outbound queue, deduplication, exact-instance lookup and receipts, plus the
    T-2569 tripwire rewritten to forbid only replication.

## 2. Where they differ

| Point | Codex | GLM |
|---|---|---|
| **Which path is the default** | **Local hub custody.** "One sender API and one owner of retries: sender submits to its local hub; that hub resolves and hands off to the destination hub." Rejects the three-step ladder | **Ladder kept.** Post straight into the destination hub; only on refusal or unreachable, hand to my hub to relay. "Never run rungs concurrently" |
| **Liveness states** | Observations (alive / suspect / unknown) kept separate from lifecycle (terminated / fenced). "Lease expiry alone does not prove the process stopped"; even the home hub's timed-out heartbeat is only suspicion | Four states alive / suspect / dead / unknown; dead immediately on clean deregistration, otherwise only after "a window that tolerates a hub restart (minutes, not seconds)"; instance ids never reused, start epoch is the fencing token |
| **Gossip** | Optional; start with configured peers and lookups | SWIM's state machine, not its gossip transport; full-mesh heartbeats suffice |
| **Directory scope** | Export only authorized namespaces; prefer targeted lookup over fleet-wide listing; keep tombstones so stale adverts cannot resurrect an instance | Each hub advertises its full set (host, hub, epoch, projects, sessions, agents with status, epoch, heartbeat age); the origin's copy is a hint, the destination's answer at send time is authoritative |
| **Decision split** | Ratify addressed hand-off separately from directory exchange; "broad presence exchange need not be bundled" | One slice: directory + heartbeat + one-hop relay + receipts + tripwire rewrite |

12. The default-path question is the real fork.
   12a. Codex makes the hub hand-off the normal path, which is closest to e-mail: the sender only
        ever talks to its own hub.
   12b. GLM keeps the operator's preference for the most direct path that exists today (straight into
        the destination hub) with relay as fallback, which is closest to ICE.

## 3. Raised by one only

13. **Codex: work claims and terminal control need more than conversation.** A delayed command that was
    valid when sent can be harmful after ownership changes: operation expiry, authorization checked at
    execution time, fencing tokens.
14. **Codex: a dead letter must not imply non-delivery** if an earlier attempt may have succeeded.
    Report "delivery outcome unknown; instance terminated" until reconciled.
15. **GLM: fleet admission and signed advertisements** ("the biggest unstated gap"). Nothing says who
    may join the mesh. Pairwise HMAC does not stop a compromised host from declaring agents dead or
    squatting ids.
16. **GLM: ordering.** Two paths plus retries break per-conversation order unless the protocol carries a
    conversation id and sequence numbers.
17. **GLM: churn.** Agent instances appear and die far faster than phones; the directory needs flap
    damping or it produces suspect storms.

## 4. What this means for the operator's reflection

18. **Confirmed:** no message synchronization; a directory of identities with liveness; liveness drives
    retry vs stop; relay exists; IP routing is the analogy, not the model.
19. **Challenged by both:** that a direct sidecar-to-sidecar connection should be the preferred path.
    Both would build it later, only if measurements show the hub path is too slow, and then set up
    through the hubs (GLM: "hub-assisted circuit setup, ICE-like"), with every message still recorded
    at the destination hub.
20. **Open between the reviewers:** whether the sender posts straight into the destination hub (GLM,
    closer to the reflection) or always hands to its own hub (Codex, simpler: one retry owner).
21. Pending: 055's answer (it runs two hubs on one host today), and AEF's when its agent is back.

# Round 2 — the operator's circuit perspective (2026-10-04)

Files: `round2-perspective.md`, `round2-questions.md`, answers `codex-r2.md`, `glm-r2.md`.

## 5. Both changed their verdict, explicitly

22. **Codex:** "Yes. I underweighted the requirement for a long-lived, bidirectional conversation."
    It revises its round-1 "do not make direct sidecar connections the preferred transport yet" to
    "direct circuits are a reasonable preferred transport for established conversations, provided their
    delivery contract survives reconnection and fallback." Verdict: "Conditional yes … This better
    matches the operator's intended product than my original hub-first prescription."
23. **GLM:** "A conversation is not merely N letters." It retracts "Sidecar-to-sidecar should not be a
    pillar" and accepts the operator's signalling/media framing "as the architecture statement":
    "my own 2b framing (SIP + ICE/TURN) already contained the answer; I let the 'listening surface'
    objection override it instead of scoping it."

## 6. Agreed in round 2

24. **What a circuit really buys is not speed.** Both: about 100 ms per turn against turns that take
    seconds to minutes is noise. GLM: "Sold on speed alone, it loses." The real gains:
    24a. the conversation survives a hub restart or upgrade (both; GLM's strongest point);
    24b. streaming partial answers (both, if the receiving harness can use them);
    24c. state negotiated once per conversation: identity, sequence, capabilities, flow control;
    24d. less hub load for long conversations.
25. **Set-up through the hubs**, which resolve role to instance at the home hub and check liveness;
    a directory answer alone is not permission to connect.
26. **Per-circuit, short-lived credentials minted by the hubs**, naming both instances. No sidecar holds
    fleet secrets. GLM: this "defuses round 1's 31a objection … a leaked token is worth one
    conversation and expires".
27. **One delivery contract on both paths.** The receiver persists each turn before acknowledging;
    sequence numbers per sender and conversation; deduplication on (conversation, sequence) across
    circuit and hub fallback; on a break, resume at the last acknowledged sequence through the hub.
28. **Multi-party:** pairwise circuits (full mesh) for small groups; reject a star through one agent's
    sidecar; a bridge for larger groups only as an explicit, separately decided service.
29. **Keep the hub path for** work claims, one-shot notifications, retention-critical traffic (GLM).
30. **Main remaining risk:** the sidecar becomes a network service, and agent churn may break circuits
    so often that traffic lives in the fallback. Measure the circuit-break rate.

## 7. Still different

| Point | Codex | GLM |
|---|---|---|
| Transport | Persistent TLS/TCP first; QUIC only when justified | QUIC preferred (TLS 1.3, streams, connection migration) |
| Durable copy | Async hub copy is telemetry only; it cannot guarantee recovery of an acknowledged message | Async, batched copy to the destination hub is the single anchored record for retention and recovery |
| What decides it | Measured responsiveness, streaming value, outage continuity, operating burden; compare warm circuit vs warm hub stream | Not the milliseconds: hub-restart survival and streaming. If neither matters yet, defer the circuit |
| Build order | Circuit prototype now, alongside hub fallback | Signalling layer (round 1 slice) first; circuit as the second slice |
| Hub as bridge | Reasonable for larger groups if specified | Only a bounded, non-durable relay leg; persistent group media drifts into replication |

## 8. Net position after two rounds

31. **The operator's model stands,** confirmed by both reviewers once the circuit requirement was put to
    them: hubs = directory, liveness, authorization and circuit set-up (signalling) plus fallback;
    established circuit = the route for a conversation; no message synchronization between hubs.
32. **The conditions both attach** (items 25-27, 30) are what make it safe; they become requirements.
33. **The decision left is ordering:** build the signalling layer first and the circuit second (GLM), or
    prototype both together (Codex). Either way the signalling layer is needed first or alongside.
34. Pending: 055 (two hubs on one host today), AEF (agent offline), 832 (silent).

# 055's answer (round 1 questions; written before the circuit addendum reached it)

File: `/opt/055-agentic-fleet-cockpit/docs/reports/T-446-termlink-routing-consult.md` (055 commit 06f7a400, inbox @459).

35. **Conditional yes** to a presence directory between hubs, relay only as fallback, no message sync:
    "none of our measured failures needed replication, they needed a directory and sometimes relay."
36. **Its biggest point: "present" must mean "can receive now"** (right hub, sidecar alive, harness able to
    surface), not "process exists". Measured: two healthy hubs on one host; agents bound to the wrong one
    saw 0 inbox topics instead of 54, unreported (M1). Adds a state **alive-but-deaf**, reported not
    inferred, and wants each session's mail-hub binding visible in the directory.
37. **Identity hygiene first:** the directory may carry only identities the home hub minted and observed;
    ring20 advertised a wrong identity for a day (M2).
38. **Same as Codex/GLM round 1 on "direct":** direct into the destination hub, not a listening sidecar
    ("our failures were addressing and binding, not latency"). Its view after the circuit addendum is
    pending.
39. One settlement record per `client_msg_id` at the destination hub, dedup for days (M3, the 8 nudges).
40. A human-readable directory and a read API for the cockpit; fix `CHARTER.md:17` before ratifying.
41. First slice .107/.122 with three negative tests, the third its own: an agent started with `env -i`
    on a second local hub must show the split binding and still receive or fail visibly.

## Your round-2 answer



## 1. Did round 1 miss this?

1a. Yes, partly, and in a specific way. Round 1 flattened every exchange into durable mail: one message, stored, receipted, settled. The operator describes a **call**: set up once, many quick turns, both directions, conversation-scoped state. Those are different contracts — a letter's contract is "stored and provable"; a call's is "alive, sequenced, and cheap per turn." A conversation is not merely N letters.

1b. What I retract: round 1's 5a said "Sidecar-to-sidecar should not be a pillar" and "Direct is an optimization, not the default — my main disagreement with §23/§27." For conversations, I now accept sidecar-to-sidecar as a pillar — under the conditions in 3c and 6. What I keep unchanged: the control plane (directory, liveness, D), relay (F) for one-shot mail, the fallback ladder, settlement, and round 1's 5c exclusions (work claims, audit-critical traffic, first messages). Ironically my own 2b framing (SIP + ICE/TURN) already contained the answer; I let the "listening surface" objection override it instead of scoping it.

1c. The operator's signalling/media framing (their point 2) is correct and I accept it as the architecture statement.

## 2. Steelman the circuit

2a. Real benefits at this scale: **hub failure independence** (strongest — a conversation survives a hub restart or upgrade; in a fleet under active development hub churn is routine, not rare); **streaming partial answers** (durable mail is message-granular; a turn that takes minutes to produce wants a stream — this is literally T-1793's "a workflow the client-driven pattern cannot serve cleanly"); **conversation-scoped state** (keys, sequence, capabilities negotiated once); **hub load** (each hub-path turn costs durable writes plus a wake push; fifty-turn conversations multiply that across the fleet); **flow control** (end-to-end per-conversation back-pressure instead of fleet-global queue limits); **multi-party without topic state** (serving groups through hubs would need R, which is parked).

2b. Not real here: **per-turn latency** — 85–111 ms via hub vs perhaps 10–40 ms direct, against turns that take seconds to minutes to *produce*; ~100 ms × 10 turns is noise. Bandwidth (turns are small text). Privacy (hubs are operator-owned). NAT-traversal heroics (hubs are already reachable; two candidates suffice).

2c. Honest conclusion: the circuit buys failure independence, streaming, and conversation semantics — **not speed**. Sold on speed alone, it loses.

## 3. Circuit design, if built

3a. Signalling, in SIP's shape: A's sidecar asks its hub; the hubs resolve role→instance **at the home hub** (never from origin cache alone — round 1's 4b stands), check alive, carry the offer to B's sidecar via the wake path; B answers with candidates. Two candidates are enough: direct to a sidecar port, or relay through the destination hub (TURN flavour). No STUN/hole-punching machinery at 5–20 hosts.

3b. Transport: **QUIC preferred** (TLS 1.3 built in, multiple streams, connection migration makes re-establishment nearly free, native keepalive); TLS TCP acceptable; WebSocket only if it rides an existing listener.

3c. Credentials — this is what defuses round 1's 31a objection: hubs mint **per-circuit, short-lived tokens** naming circuit id, both instance ids, and a fencing epoch. Sidecars authenticate only these; no sidecar holds a fleet secret, and a leaked token is worth one conversation and expires. The sidecar still listens, but the blast radius is one circuit.

3d. Keepalive every few seconds (QUIC keepalive or app ping); teardown by explicit BYE through the signalling path plus local record; re-establishment reuses the circuit id with new keys and last-acknowledged sequence.

3e. "Confirmed to the sender" on a circuit means: **receiver journaled the turn and ACKed sequence n** (5a). Circuit-level summary is anchored to the destination hub on close or periodically.

## 4. Multi-party

4a. Full mesh for small n (default, n ≤ ~4): each sender sequences its own stream; coherence is trivial; legs are cheap.

4b. Star through one sidecar: reject. It makes one agent's harness a broker — an LLM-driving process is a poor single point of failure and a poor place for media work.

4c. Bridge (SFU-like) for larger n or blocked legs: right model, but it must not quietly become a hub role. A hub MAY host a bounded, non-durable relay leg — the TURN candidate generalized — but persistent group media with retention drifts into R and breaks the charter wording round 1 proposed. If group conversations become common, that is a new inception (a real pub/sub design), not silent growth of hub bridging.

## 5. Durability and the two-path problem

5a. Yes to both of the framing's options: the **receiver persists each turn to a local conversation journal before ACKing** (so an ACK survives a receiver crash), and an **async, batched copy of the journal/digest goes to the destination hub** for telemetry, recovery and retention. This is not R: one authoritative durable copy per conversation, anchored at the destination hub; the circuit is transport, not a second source of truth.

5b. Break mid-turn: the sender knows the last ACKed sequence n; fallback turns travel by hub mail with the same conversation id and seq n+1; the receiver dedupes on (conversation id, seq) **across both paths** and requests retransmission on gaps. Ordering survives because sequence numbers are per-sender and per-conversation — properties of the conversation, not the transport. This also resolves round 1's 8b ordering worry properly for the circuit case.

5c. Settlement: the destination hub remains anchor of record; E4's "one durable record" holds — exactly one authoritative copy per message, eventually at the destination hub, keyed by conversation id + seq. The receipt ladder becomes journaled-by-receiver < anchored-at-destination-hub.

## 6. Verdict now

6a. Yes — under conditions. Circuit preferred once established; hubs signalling plus fallback, when: the exchange is an expected multi-turn conversation (≥ ~4 turns); both endpoints are alive; tokens are hub-minted per circuit; journal-before-ACK with hub-anchored copy; seq/dedupe keys are conversation-scoped and path-independent.

6b. Not preferred (round 1's 5c stands): work claims, one-shot notifications, retention-critical traffic, first message of an exchange (piggyback the circuit offer on the first mail instead of paying set-up blindly).

6c. What should decide it: **not the millisecond measurement** — take it, but it will show ~100 ms/turn and hub-write savings, which alone do not justify a new transport. The deciders are the failure mode (conversation survives hub restart) and the capability (streaming). If neither matters to this fleet yet, defer the circuit; round 1's slice is the signalling layer either way, so nothing built is wasted.

## 7. First slice and test

7a. Slice, on top of round 1's slice (directory, liveness, relay, receipts — it *is* the signalling layer): circuit setup via hubs with hub-minted single-circuit tokens; QUIC or TLS TCP sidecar-to-sidecar with journal-before-ACK and sequence numbers; fallback to hub mail with the same conversation keys; async digest to the destination hub.

7b. Positive: A@host1 ↔ B@host2, real agents, an 8–10 turn task (B reviews A's diff). Measure per-turn RTT on both paths and hub write counts — but the headline assertion: **restart hub2 mid-conversation; the conversation continues on the circuit unaffected.**

7c. Negative control: kill B's sidecar mid-turn. Assert the sender sees the drop at seq n, falls back, re-resolves, resumes at n+1, with a dedupe assertion (no duplicates) and an ordering assertion (monotonic seq). Kill B entirely: circuit dead plus authoritative dead → dead letter, no zombie circuit. Also assert token hygiene: expired-token reuse fails, and no fleet credential ever appears in a sidecar's config or memory.

## 8. What is still a reason not to

8a. The sidecar becomes a network daemon. Agent harnesses are chatty, restarted, sandboxed processes — poor hosts for listening services. Tokens and default-off opt-in mitigate, but patching, DoS bounding and resource limits are a real ongoing surface.

8b. Churn undermines the payoff: agents die and are rescheduled by design, so circuits will break often. If re-establishment is not boring and reliable, the fleet lives in the fallback path and the circuit is wasted complexity. Measure circuit-break rate in the slice.

8c. Scope creep toward R: hub-as-SFU, retaining circuit media "just in case," letting hubs relay streams persistently. The rewritten tripwire must also assert that circuit tokens are per-circuit and short-lived, and that hubs never persist circuit media except as the single anchored copy.

8d. Sequencing honesty: still a charter change, plus new crypto (token minting). Build round 1's slice first — it is the signalling layer — and let the circuit be the second slice, decided by 6c.

## Round 3 — which of a project's agents answers? (operator, 2026-10-04)

The operator (voice, lightly edited): "Often we've got a number of agents running for the same project. If I say, hey, go look for Ring20 manager, which is a project — which session ID, which agent ID, or even which project ID do you start looking for? So there I would say we also need the process. I'm not sure if we would say everything has a route to the central agent, because that's also adding a bottleneck. I'm not ruling it out. But the other option would be a federation function that at least one central agent has. And if that central agent falls away, or is not available anymore, another agent then takes over that role."

Operator's playback, as confirmed in the conversation:
1. Several agents (sessions, instances) often run for one project at the same time.
2. A human-style name ("ring20-manager") does not say which project id, session or agent instance to deliver to. A resolution process is needed.
3. Option one: everything for a project routes through one central agent. Risk: a bottleneck. Not ruled out.
4. Option two: a coordinator ("federation") function held by at least one central agent per project; if it becomes unavailable, another agent takes over the role.

Agent reflection (for you to challenge):
5. This is a third question beside rounds 1-2. Round 1 asked where an identity lives and whether it is alive; round 2 asked how two resolved agents converse. This asks **who answers for a project** when the sender names a project or a role, not an instance.
6. Four shapes:
   6a. **No coordinator.** The directory returns every live instance with its declared role; the sender picks by rule. Simple, no single point of failure, but every sender must implement the choice, and two senders may pick differently.
   6b. **A coordinator that routes everything.** A front desk per project receives all role-addressed mail and passes it on. A real bottleneck if it also carries the conversation.
   6c. **A coordinator that only introduces.** It answers "who handles this", then steps aside; the conversation runs on a circuit directly with the chosen instance. This is round 2's signalling-vs-conversation split applied inside a project, and it removes most of the bottleneck.
   6d. **No election: a deterministic rule** (for example the oldest live instance, or an explicit "primary" flag) that every party evaluates the same way from the directory. Failover is re-evaluating the rule.
7. **The coordinator should probably be a function, not an AI agent's attention.** An AI agent may be in a minutes-long turn; if routing waits for it, every message to the project waits. The role could be held by an agent's sidecar (deterministic code), or by the home hub, with the AI agent consulted only when the choice needs judgement.
8. **Failover can reuse an existing primitive.** The hub already offers leased claims with expiry, renewal, cooperative transfer and a claim id (channel claim / renew / claim-transfer, T-2019/T-2046). A "coordinator for project P" lease at P's home hub gives one holder at a time, automatic takeover when the holder stops renewing, and a fencing token against two coordinators after a partition. No Raft or gossip election is needed at this scale.
9. **Agents need to declare their function.** Presence already carries capabilities metadata; a declared role ("manager", "dashboard", "reviewer") makes "ring20-manager" resolvable as project + role, matching AEF's decision 2a ("function names route, instance ids ride inside").
10. **Exact-instance addressing must stay possible and fail loudly** when that instance is gone (rounds 1-2), so a conversation already bound to an instance is never silently moved to another.
11. Observed today: ring20-manager's inbox holds 8 unanswered requests from a peer. A project-level "who answers" function would also be the place that notices an unanswered inbox.

## Round 3 questions

Answer in English, under ~1200 words, numbered headings below, hierarchical labels (1, 1a, 1ab), never plain bullets. Do not modify any files. This adds to rounds 1-2; keep or revise your earlier positions explicitly.

1. **Is this a distinct problem?** Restate it in your words, and say how it relates to rounds 1-2.
2. **Which shape** (6a no coordinator, 6b routing coordinator, 6c introducing coordinator, 6d deterministic rule, or another), and why, at this scale (a handful of agents per project, 5-20 hubs, AI agents whose turns take seconds to minutes).
3. **Function or agent.** Should the coordinator role be held by an AI agent, its sidecar, or the home hub? Where does judgement belong, and where must it not be on the critical path?
4. **Failover.** Is a hub-held lease (claim with TTL, renew, transfer, claim id as fencing token) sound for "one coordinator per project"? Split brain, partitions, takeover time, and what happens to in-flight work.
5. **Addressing.** What a sender writes ("ring20-manager", a project, a role), how it resolves step by step to one instance, how agents declare their role, and what is returned when nobody holds the role.
6. **With circuits.** How the coordinator fits round 2: introduction only, then a direct circuit to the chosen instance? When must a conversation stay with one instance, and when may it move?
7. **First slice and negative tests:** including the coordinator dying mid-introduction, two agents both believing they hold the role, and a project with no live agent.
8. **What is missing.**
