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
