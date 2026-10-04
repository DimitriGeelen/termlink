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
