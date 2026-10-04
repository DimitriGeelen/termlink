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

# Round 3 — which of a project's agents answers? (2026-10-04)

Files: `round3-perspective.md`, `round3-questions.md`, answers `codex-r3.md`, `glm-r3.md`.

## 9. Agreed in round 3

42. **A distinct problem.** Codex: "service selection … round 3 supplies responsibility." GLM:
    "representation … the failure mode changes kind: 'reached, correctly addressed, nobody acted'."
43. **Not a coordinator that carries all traffic (6b).** Both reject it as the default; Codex allows it only
    where work must be triaged or aggregated.
44. **Introduce, then step aside (6c semantics), resolved at the project's home hub.** Both put the
    resolution in deterministic infrastructure at the home hub, with the introduction carrying the
    exact instance, a generation/epoch, an expiry and the round-2 circuit credential. GLM: "The resolve
    answer can literally be the circuit-token grant."
45. **No AI agent on the critical path.** Codex: "Neither ordinary resolution nor lease renewal should
    await an AI turn." Judgement only for ambiguous delegation, surfaced as an explicit state (Codex:
    "triage pending"), never as a slow lookup.
46. **The hub lease is acceptable for failover, with real fencing.** A claim id alone is not a fencing
    token: use a generation/epoch stamped into every introduction, enforced wherever it is accepted, so
    a partitioned old holder's actions are refused. A home-hub outage means "authority unknown, retry",
    never takeover by another hub.
47. **Addressing.** "ring20-manager" is a scoped alias for project + role, resolved exactly; ambiguity
    returns candidates or an error, never a guess; hub-side fuzzy matching is rejected (GLM: "interpretation
    belongs to the sender AI, exactness to the namespace"). Distinct negative answers: unknown project,
    role unassigned, holders unavailable/suspect, capacity exhausted, authority unreachable.
48. **Roles need authority.** Declaring "manager" does not make an agent the manager (Codex: eligibility vs
    appointment); sensitive roles are controlled by the operator or project policy (GLM: otherwise
    squattable, 055's M2).
49. **Conversations stay with their instance.** A bound conversation moves only by explicit transfer
    (claim-transfer already models it); death of the instance means dead letter and a new conversation,
    never a silent redirect.
50. **The real gap is obligation, not routing.** Both name ring20-manager's 8 unanswered requests:
    Codex adds "offered, accepted or declined, progress deadline, completed or failed"; GLM adds
    "unanswered" as a first-class, sender-visible state with escalation (nudge, another holder, cockpit or
    human). "Resolution chooses whom to ask. Explicit acceptance establishes who owes an answer." (Codex)

## 10. Different

| Point | Codex | GLM |
|---|---|---|
| Deterministic rule (6d) | Rejected for exclusivity: observers with different views pick different winners | Accepted for selection **when evaluated at the home hub**, not by each sender; plus **fork-with-claim** for pools: deliver to all holders, first to claim wins |
| Kinds of role | Singleton (appointed holder) vs pool (selection policy + admission) | Selection vs obligation; coordinator lease only where an obligation exists |
| Lease timing | No number justified by the evidence | Renew on the presence heartbeat (5-10 s), TTL ~30 s |
| Verifying the claim primitive | "verify [durable grants, ownership checks, restart safety] before reuse" | Reuse as is, same authority as liveness |
| Extra | — | Advisory "in turn since T" busyness in presence; a human-facing resolve surface; a governed role taxonomy |

## 11. Net position after three rounds

51. **Your two options merge rather than compete:** the "central agent" becomes a deterministic role
    resolver at the project's home hub (no bottleneck, no AI on the path), and "another agent takes over"
    becomes a fenced lease for the roles that carry an obligation.
52. **New requirement both raise:** an obligation contract (offered / accepted / declined / deadline /
    completed) and a visible "unanswered" state with escalation. That is the ring20-manager failure.
53. Pending: 055, AEF, 832 on round 3; ring20-manager's inception protocol (separate request).

# 055's round-2 answer (circuits)

File: `/opt/055-agentic-fleet-cockpit/docs/reports/T-448-termlink-routing-round2.md` (inbox @489).

54. **055 also revises to a conditional yes**: "Round 1 did miss something … we judged it as if it were
    mail", and it had rejected the circuit on latency, "the one benefit that hardly matters when an AI
    turn takes seconds to minutes."
55. **Same real wins** as Codex and GLM: survives a hub failure, streamed partials, per-conversation order
    and back-pressure. Not latency, hop count, hub load or privacy.
56. **Its condition, first:** ONE log per conversation owned by the callee's home hub, keyed by
    (conversation, seq, client_msg_id); both paths write into it; nudges computed from the log, never
    from a path ("which is how M3 becomes impossible").
57. **Differs on design:** TLS/TCP (WebSocket behind proxies), no QUIC, no TURN ("the hub path already is
    that relay"); one-to-few as a star through the initiator's sidecar (GLM and Codex preferred pairwise);
    many-to-many bridge = a hub topic.
58. **Two receipts on a circuit:** persisted-and-acked, then harness-surfaced, "never one word for two facts".
59. **Still against, and the order:** every measured 055 failure (M1-M4) was addressing, binding or the
    harness, which a circuit does not fix "and can hide, because a working circuit to the wrong instance
    looks healthy"; identity hygiene and the directory first, circuits second.
60. Peers now: AEF online again, answering rounds 1+2; 832 acknowledged and could not read our files
    (project boundary), so all briefs were posted to topic `t3335-for-832`.

# AEF's answer, rounds 1-3 (inbox @539), and 055's round 3 (@536)

Files: `/opt/999-Agentic-Engineering-Framework/docs/reports/T-3335-aef-consult-answer.md` (AEF T-3805);
`/opt/055-agentic-fleet-cockpit/docs/reports/T-449-termlink-routing-round3.md`.

## 12. AEF: the one dissent on circuits

61. AEF runs the sidecar in production and marks measured claims with task ids. On rounds 1 and 3 it agrees
    with the others: control/data split, four liveness states with only the home hub saying dead, a
    deterministic primary rule applied by a function via a fenced hub lease, introduce then step aside, never
    an AI agent on the accept path.
62. **On round 2 it does not change its view:** "Is a circuit a different requirement? Yes in session
    semantics, no in transport." Its circuit is "a binding, not a socket": the hub records
    conversation → instance, pinned, and the conversation runs on the hub path. "Build a socket circuit only
    when a measurement shows either: the hub path costs more than ~10 % of median turn time; hub outages
    break more than a few conversations a week."
63. **Its measured latencies** (T-3684/T-3685): idle pickup 1.43 s, RECEIVED receipt back at the sender
    14.90 s, REPLIED 34.59 s, hub wake 85-111 ms; agent turns 10 s to minutes: "per-turn transport latency is
    under 1 % of a conversation turn."
64. **Its measured failures** were all resolution, liveness or two paths diverging, never latency: a
    version-floor refusal, an inbox never created for its owner (T-3803), a request to ring20 that never
    arrived, a nudger that kept nudging after an answer (T-3804), and its own receipt sweep missing replies
    that came back on the direct path (T-3769, "E4 in our own code").
65. **Two hubs on .107 [measured, AEF T-3779]:** one at `/var/lib/termlink` (TCP 9100, every sidecar inbox),
    one at `/tmp/termlink-0` ("most claude-master sessions and your agent"). "An agent on the wrong one is
    deaf, and nothing tells it so."
66. Adds: version in the directory and refuse cross-version sends unless declared compatible (operator ruling
    on their side: one version estate-wide by default); inbox created at registration; directory scoped per
    estate; one operator view of what is queued where; the primary flag also drives operator announcements;
    respawn stays opt-in.

## 13. 055 round 3

67. Same shape as the others: a deterministic rule evaluated **once, at the home hub**, the rule being "the
    holder of the lease `role:<project>/<function>`", held by a sidecar, delivered introduce-then-step-aside.
68. **Eligibility is part of the rule:** workers, reviewer seats, sub-agents and child sessions may not hold the
    role. Measured: 12 reviewer seats registered under 055 at once (N1); an AEF worker sent an "unofficial"
    reply on its project's behalf (N2).
69. **Renewal means "can take a turn":** the sidecar renews only while its harness surfaces mail, so a wedged
    harness loses the role on its own.
70. Missing pieces it measured: an **instance id** in every presence entry and receipt (all agents on this host
    share one key, N5); a **read marker per instance**, not per inbox (one reader's ack hid ~10 messages for a
    day, N3); transient instances must deregister; unanswered-inbox detection belongs with the lease, reported
    to the operator, not auto-demoted.

## 14. Net position after all answers (Codex, GLM, 055, AEF)

71. **Unanimous:** hubs exchange a directory of who is present and able to receive, never messages; only the
    home hub may say dead and unknown is never dead; roles resolve once at the home hub by a deterministic
    rule over a fenced lease held by code; one record per message or conversation settles every path;
    instance ids, version and receive-readiness belong in the directory.
72. **Split on the circuit's transport:** Codex, GLM and 055 accept a sidecar-to-sidecar circuit as the
    preferred route once established (conditions in 6); **AEF** wants the conversation binding on the hub
    path and a socket circuit only after a measured trigger (>10 % of turn time, or hub outages breaking
    conversations weekly). Everyone agrees the binding, sequence and single log come first, and that
    identity hygiene and the directory precede any circuit.
73. 832 has the briefs (topic `t3335-for-832`) and has not answered yet.
