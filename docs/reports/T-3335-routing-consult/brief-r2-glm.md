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

## Your round-1 answer



## 1. The split

1a. Verdict: yes. The control-plane/data-plane split is the right architecture, and it is not novel — it is how e-mail, SIP and XMPP work: servers exchange reachability and relay when needed; endpoints talk through their home server. It dissolves T-1793's costs because no message state is ever shared (§29 is correct), and it fits the scale (5–20 hubs) with trivial machinery.

1b. Biggest weakness: the "direct" leg. As written (§23, §27) it implies sidecars accepting connections from other hosts — ports, firewall, per-agent TLS identities, TOFU at N-agent scale — which multiplies the listening surface from ~M hubs to N agents, re-creating the credential sprawl the proposal claims to kill (§31a) and removing hubs as the single point of durability, dedupe and audit. In the first build, "direct" should mean direct-into-the-destination-hub (see 5a). I disagree with the operator here: the IP analogy fails at exactly this point, because in IP the "direct path" changes nothing about trust, whereas here it changes the trust topology.

1c. Second weakness worth naming: the architecture's value equals the directory's truth. A false "dead" destroys mail via dead letters; a stale "alive" wastes retries. Liveness correctness is a first-class requirement, not plumbing.

## 2. Protocol models

2a. Agree: IP routing is the wrong closest fit. RIP/OSPF compute multi-hop paths through a graph; at 5–20 full-mesh, one-hop hubs there is no path to compute. The need is resolution and membership. Only BGP contributes one idea (2d).

2b. SIP: borrow the architecture, not the syntax — REGISTER (binding an address-of-record to a contact = role→instance), location service (the directory), proxy resolution, and the ICE/STUN/TURN pattern: direct candidates first, relay (TURN = hub relay) as fallback. Skip SIP's dialog machinery.

2c. SWIM/Serf: borrow the state machine — probe, indirect probe, suspicion timeout, alive/suspect/dead — and anti-entropy exchange for directory convergence. But at 20 hubs the gossip transport is overkill: full-mesh TLS heartbeats carry it. SWIM's semantics, not its epidemics.

2d. BGP: advertise only what you are authoritative for; never re-advertise another hub's entries (no transit). This makes "only the home hub declares dead" structural rather than conventional.

2e. DNS: TTLs on every answer, including negative answers (cache "no such agent here" briefly, or lookups storm on typos), and answers carrying their age (§30c is right).

2f. E-mail MX/DSN: next-hop store-and-forward, and — most valuable — delivery status notifications with the 4xx/5xx split: temporary failure (retry, suspect) vs permanent (dead-letter, dead). The hop-rule receipts (§14) are DSNs; adopt their semantics.

2g. XMPP server-to-server: the closest whole-system precedent — federated servers, presence, clients talk only to their home server, addressed messages relayed between servers. Read it before choosing wire formats.

## 3. Liveness

3a. Four states: alive, suspect, dead, unknown. §31c is right, and unknown (home hub unreachable) must surface as "no information," never merge into dead.

3b. Authority: only the hub named in the address may declare dead, from its own agent heartbeats. Every other hub can only say "last seen alive at T, per hub X," attributed (§30d).

3c. Speed: dead immediately on clean deregistration; on silence, suspect after ~2–3 missed heartbeats, dead only after a window that tolerates a hub restart (minutes, not seconds). Never reuse instance ids — resurrection is a new instance; the start-time epoch is the fencing token.

3d. Sender use: alive → send. Suspect → retry the same message id with backoff; never dead-letter on suspect. Dead → stop; dead-letter mail to that exact instance, with evidence (who declared, when); role-addressed mail re-resolves instead. Unknown → retry with a cap, reported as "unreachable, no information," visibly distinct (OD-14). Every dead letter carries the authoritative state and its age.

## 4. Directory contents

4a. Each hub advertises: host id, hub id, hub epoch; per project served: project canonical id; per session: canonical id plus live session instance ids; per agent: canonical id plus live agent instance ids, each with status, start epoch, heartbeat age. Nothing else; no re-advertisement (2d).

4b. Scope and authority: authority is per (host, hub). The origin hub's directory is a cache of hints; the destination hub's answer at send time is authoritative. Role→instance resolution must be performed by the destination hub, not from origin cache alone.

4c. Caching: per-answer TTL, negative answers included; liveness removes entries (dead deletes, TTL does not); TTL governs only the role→instance binding. TTL expiry means re-ask; dead means stop. Answers carry their age.

4d. Role→instance: a canonical agent address resolves to the newest live instance by epoch; an exact-instance address must fail loudly if gone (§31d is right); no silent redirection. Start with single-active-instance per agent; defer multi-instance policy.

## 5. The direct path

5a. First build: direct = sender's sidecar → destination hub, over the existing TLS+HMAC surface. This is today's cross-post made pleasant by the directory and by hub-vouching (short-lived tokens minted hub-to-hub). Sidecar-to-sidecar should not be a pillar: the 85–111 ms hub path is already interactive; direct buys little and costs a listening surface per agent plus re-implemented durability and receipts. Direct is an optimization, not the default — my main disagreement with §23/§27.

5b. If/when sidecar-to-sidecar is built: hub-assisted circuit setup (ICE-like — hubs exchange endpoints and single-conversation tokens, TURN-allocation flavor), with every message still reconciled into the destination hub's durable store keyed by message id (§31b).

5c. When direct should not be preferred: work claims (hub arbitration is the point); anything needing an authoritative receipt ("stored by destination hub" can only be said by the destination hub); first message of a conversation; retention/audit-sensitive traffic; any send to a target not confirmed alive.

## 6. The fallback ladder

6a. Rungs: (1) resolve from cached directory; (2) post straight into the destination hub — expect "stored by destination hub"; (3) on refusal or unreachable, hand to my hub to relay (F): durable queue at origin, sender sees "accepted by my hub, queued, destination unreachable"; (4) suspect → retry same message id with backoff; (5) authoritative dead → stop, dead-letter. §31f's order is right; add one rule: never run rungs concurrently — escalate only on definitive failure, or you will inject duplicates.

6b. Sender feedback: the strongest receipt earned so far — accepted-by-origin < stored-by-destination < injected-into-instance. The operator's confirmation goal should take "stored by destination hub" as the default meaning of "delivered."

6c. Settlement: exactly one durable record per message, owned by the destination hub, whichever path carried it; the origin's relay queue holds only until the destination acknowledges storage, then drops. The receiver dedupes on `client_msg_id` with retention longer than the ladder's worst-case retry horizon — days (§31b). This is the answer to E4's two-paths objection.

## 7. First slice and test

7a. Slice: (i) hub-to-hub directory and heartbeat over existing TLS/HMAC (alive/suspect exchanged; dead declared only locally); (ii) one-hop relay with hop limit 1 and refusal — not creation — of unknown destinations; (iii) receipts accepted/stored/injected; (iv) the T-2569 tripwire rewritten to forbid only R (no topic sync, no re-advertisement). No sidecar-to-sidecar.

7b. Two real agents, A@host1/hub1 and B@host2/hub2. Positive: A messages B by role; B replies the same way; assert A sees "stored by hub2," then "injected"; record round-trip against the 85–111 ms baseline. Every step confirmed to the sender — the operator's actual goal.

7c. Negative control — dead agent: kill B's sidecar; hub2 goes suspect then dead; A's pending mail dead-letters with "dead, per hub2, at T"; A's next role-send re-resolves or fails loudly.

7d. Negative control — outage ≠ dead: stop hub2 entirely; hub1 must report B unknown/unreachable, never dead; A's mail sits "queued at hub1"; restart hub2, directory repopulates, queue drains, delivered. Assert the literal string "dead" never appears in that transcript. Add: an exact-instance send to the dead instance must fail loudly, not redirect.

## 8. What is missing

8a. Fleet admission and directory integrity — the biggest unstated gap. Everything assumes honest hubs. Nothing says who may join the mesh or how advertisements are authenticated; pairwise HMAC does not stop a compromised host from poisoning presence (declaring agents alive or dead) or squatting ids. Needed: a fleet hub allowlist or small PKI, and signed advertisements. §16 moves the trust burden to hubs but leaves the mesh itself open.

8b. Ordering and correlation across paths. §10 claims "ordering is per conversation," but two paths plus retries break per-conversation ordering unless sequences are explicit. Interactive two-way talk needs a conversation id and sequence numbers in the protocol — dedupe alone is not coherence.

8c. Churn and back-pressure. Agent fleets churn far faster than SIP handsets — sessions and instances appear and die constantly — so the directory needs hysteresis/flap damping (else suspect storms), and relay queues need per-peer limits: an unbounded origin-hub queue aimed at a slow destination is a new fleet-wide failure mode (§13 names the queue, not its limits).

## Round 2 — the operator's perspective, which round 1 did not address (2026-10-04)

The operator read both round-1 answers and said (voice, lightly edited):

"I'm not sure the reviewers have the perspective of why we have sidecar communication. The sidecar is especially for one-to-one agent communication. It could also be one-to-multiple agents, or multiple agents to each other, but it is for a circuit type of communication. We want efficient communication back and forth, and I don't think the hub helps with that. The hub could help with establishing the communication channel, but once it has been established, that should be the route. That's my feeling. I don't want to press it, but I really want to put this back in the review with that insight and that perspective."

Our framing of the distinction, for you to accept or correct:
1. Round 1 treated every exchange as **durable mail**: one message, stored, receipted, settled. The operator is describing a **conversation circuit**: a channel set up once between two (or more) agents' sidecars and then used for many quick turns in both directions, like a call rather than a series of letters.
2. In the operator's model the hubs are the **signalling** layer (resolve who and where, check alive, authorize, set the circuit up, and carry traffic only when no circuit can be made), and the circuit is the **media** layer.
3. The operator invites disagreement. Give your honest view: change your round-1 answer where this perspective changes it, keep it where it does not, and say which.

## Round 2 questions

Answer in English, under ~1300 words, numbered headings below, hierarchical labels (1, 1a, 1ab), never plain bullets. Do not modify any files.

1. **Did round 1 miss this?** Is a conversation circuit a genuinely different requirement from durable addressed mail? Did your round-1 answer change, and where exactly (quote yourself)?
2. **Steelman the circuit.** What does an established circuit give that the hub path cannot, for interactive agent-to-agent conversation: per-turn latency, hop count, hub load and hub failure independence, flow control and back-pressure, conversation-scoped state, streaming partial answers, multi-party, privacy? Be concrete; say which benefits are real at this scale (5-20 hubs, a few hundred agent instances, turns that take an AI agent seconds to minutes to produce) and which are not.
3. **Circuit design, if built.** Set-up through the hubs (signalling), transport between sidecars (TLS TCP, QUIC, WebSocket; NAT and firewall), circuit credentials (for example hub-minted, per-circuit, short-lived tokens so no sidecar holds fleet credentials), keepalive, teardown, re-establishment, and what "confirmed to the sender" means on a circuit.
4. **Multi-party.** One-to-many and many-to-many circuits: full mesh between sidecars, a star through one sidecar, or a bridge (like an SFU/MCU in video calls) — and whether that bridge should be a hub.
5. **Durability and the two-path problem.** What is recorded where while a circuit carries traffic (receiver persists before acknowledging? an asynchronous copy to the hub for telemetry and recovery?), and how a conversation continues cleanly when a circuit breaks mid-turn and traffic falls back to the hub path, without duplicates or reordering.
6. **Your verdict now.** Circuit as the preferred route once established, hub as signalling plus fallback — yes, no, or under which conditions; and what should decide it (a measurement, a failure mode, a scale threshold).
7. **First slice and test** that would prove or disprove the circuit's value with two real agents on two hosts, including the measurement and a negative control (circuit dropped mid-conversation).
8. **What is still a reason not to**, if any.
