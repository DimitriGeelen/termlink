# T-3335 routing consultation — GLM-5.3

Source: glm.raw (opencode, zai-coding-plan/glm-5.3; rc=0). ANSI and banners stripped.



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
