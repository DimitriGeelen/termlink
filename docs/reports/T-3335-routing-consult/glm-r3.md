# T-3335 routing consultation round 3 — GLM-5.3

Source: glm-r3.raw (ANSI and banners stripped).



# Round 3 — who answers for a project

## 1. Is this a distinct problem?

1a. Yes, and the distinction matters: rounds 1-2 solved *reachability* — where an identity lives, and how two resolved endpoints converse. Round 3 is *representation*: several live instances back one logical service, so someone must choose among them, and after the choice someone must be **obliged to answer**. The failure mode changes kind: rounds 1-2 failures were "cannot reach"; the observed round-3 failure (8 unanswered requests in ring20-manager's inbox) is "reached, correctly addressed, nobody acted".

1b. In the mapping I used in rounds 1-2: round 1 was the SIP registrar/location service, round 2 the media path. Round 3 is SIP's forking/redirect question — what the registrar does when one address of record maps to many contacts. DNS SRV and e-mail aliases are the same question in older clothes.

## 2. Which shape

2. My answer splits 6a-6d, because they conflate two jobs: **selection** (which instance takes this — stateless, computable) and **obligation** (who is on the hook to answer — needs exactly one holder).

2a. **Selection: 6d, evaluated at the project's home hub, not by every sender.** The hub's resolve verb returns live candidates with declared roles. Exactly one holder → that instance. Several → a documented deterministic default rule, or **fork-with-claim**: deliver to all holders, first to claim wins, the rest see "claimed by X" — SIP parallel forking built from the hub's existing claim primitive. Fork is the honest fix for 6d's dumbest failure (the "oldest live instance" is mid-10-minute-turn): the free agent self-selects, no coordinator attention needed. This keeps 6a's no-single-point-of-failure while avoiding its policy drift.

2b. **Obligation: 6c, and only where an obligation exists** (answering for the project, watching for unanswered mail). An introducing coordinator that is a function — never 6b, which I reject for the same reason round 2 rejected a star through one sidecar: it puts a chatty, restarted process on every critical path. The operator's own bottleneck hesitation is correct.

2c. **Degenerate default:** if no coordinator holds the lease, resolution still works via 2a. The coordinator is an enhancement, never a dependency — the right posture at a handful of agents per project.

## 3. Function or agent

3a. Never an AI agent's attention on the critical path. Turns take seconds to minutes; judgement is needed rarely, latency always.

3b. **Default holder: the home hub**, running a dumb resolve function. This is coherent with round 1 — the home hub is already the liveness authority; making it the role authority adds no new failure surface. The charter concern (hubs stay dumb transport) is met by keeping policy as *data*: the hub mechanically applies declared attributes and versioned default rules; it never decides what "manager" means.

3c. The lease may instead sit on an agent's **sidecar** where the function needs state or judgement (e.g., "who holds context"). The AI behind it is consulted off the critical path only.

3d. Where judgement must **not** be: the transport. Fuzzy matching ("mgr" ≈ "manager") must never live in the hub; interpretation belongs to the sender AI, exactness to the namespace.

## 4. Failover

4a. Yes — reusing the hub lease (T-2019/T-2046) is sound at this scale: no election protocol, one authority, and it is the *same* authority round 1 made solely competent to declare an agent dead. One authority for existence and coordinatorship is a genuine simplification.

4b. **Split brain:** a lease prevents double-holders at the hub, but a partitioned holder can keep *believing*. Fencing must make its actions refusable: every coordinator output is hub-stamped — put the claim id/epoch inside the minted introduction token, reusing round 2's token minting. Stronger, and the design rule I'd insist on: exercising coordinator powers requires reaching the home hub to mint the artifact, so a partitioned holder is *structurally* unable to act, not merely forbidden to.

4c. **Takeover:** renew the lease on the presence heartbeat (5-10 s), TTL ~30 s. Holders are sidecars or hub code, so renewal is cheap. Tens of seconds of introduction unavailability is fine; conversations on circuits are unaffected — round 2's failure independence now bought inside projects too.

4d. **In-flight work:** already-minted introductions stay valid (they are answers, not promises). The coordinator should hold no state the hub cannot reconstruct; then failover loses only seconds. The one stateful duty — the unanswered-mail watch — rebuilds from the hub's own records.

4e. Home hub outage: no election possible, and that is correct — round 1's rule: unreachable authority is *unknown*, retry; never "dead", never a coup by a foreign hub.

## 5. Addressing

5a. A sender writes one of three: an exact instance (unchanged, fails loudly — rounds 1-2); `//project/role/name`; or role + fork. "ring20-manager" is sugar for the second.

5b. Steps: (i) project → home hub via the directory, TTL-cached, negative answers cacheable with a TTL (round 1's DNS rule); (ii) resolve **at the home hub**, never from origin cache alone — round 2's permission rule extended to role resolution; (iii) return instance ids with epochs, status, heartbeat age, and mail-hub binding so 055's alive-but-deaf cannot hide; (iv) one → deliver; several → default rule or fork; none → explicit negative.

5c. Declaration: the sidecar asserts role labels at registration; several allowed. My round-1 admission point (15) applies doubly: only hub-minted identities may declare, and sensitive roles should be ACL'd by the operator — otherwise "manager" is squattable (055's M2).

5d. Nobody holds the role: return "no holder of role X in P, as of T, per hub H", distinguished from "declared, all holders suspect" (retryable) and "no such role" (permanent-ish). Never a silent drop into a project inbox — inbox delivery is an explicit, visible sender choice (OD-14).

## 6. With circuits

6a. **Introduction only.** The resolve answer can literally be the circuit-token grant (round 2's 3c collapses into it): coordinator mints, steps aside, media runs instance-to-instance. The coordinator is structurally off the conversation path — that is 6c's whole value.

6b. **Sticky vs any**, declared by the sender at set-up. Sticky (default): once bound, the conversation stays with that instance; on authoritative death → dead letter; the sender re-resolves and opens a *new* conversation (context transfer is the agents' job, never a silent redirect — rounds 1-2 unchanged). Any: per-turn re-resolution for stateless turns (status pings).

6c. Cooperative moves exist and only there: explicit transfer (claim-transfer already models it), hub re-points the role, epochs fence the outgoing holder.

## 7. First slice and negative tests

7a. One project, three agents (two declaring "manager"), one or two hubs. Build: role field in registration; hub resolve verb with negative answers; lease on one sidecar renewed with heartbeat; hub-stamped introductions. Defer fork-with-claim if the slice must stay minimal.

7b. Positive: cross-host resolve plus one exchanged conversation.

7c. Negatives: (i) kill the coordinator mid-introduction → takeover within TTL, introduction completes, and **zero** post-expiry artifacts accepted from the old holder (assert on epoch); (ii) induced split brain — pause the holder past TTL, grant B, resume A — at every timestamp exactly one valid claim id exists, and A's actions fail with "stale claim"; (iii) no live agent for the project → explicit negative answer, no hang, no void delivery, and "dead" never appears for a merely unreachable case (round 1's literal assertion); (iv) role squat by a non-admitted identity → refused; (v) resolve during home-hub outage → "unknown, retry".

## 8. What is missing

8a. **The obligation layer is the real gap, and none of 6a-6d contains it.** Perfect routing still produced 8 unanswered requests in a live, correctly-addressed inbox. Round 1's receipt ladder ends at "injected/acknowledged"; nothing owns *answered*. Needed: role mail wakes, an unanswered timeout, escalation (nudge → another holder → cockpit/human), and "unanswered" as a first-class sender-visible state. That, not selection, is the operator's observed failure.

8b. Humans are senders too: "if I say, go look for Ring20 manager" is a human addressing an agent; the resolve verb needs a read-only human surface (CLI/cockpit — 055's point 40).

8c. **Role taxonomy governance:** without a project-scoped schema, labels drift and someone will propose hub-side fuzzy matching. Fuzzy belongs to sender AIs; the namespace stays exact.

8d. Advisory busyness in presence ("in-turn since T") would let even single-pick rules avoid the busy instance; sidecars know it, and the directory can carry it cheaply.
