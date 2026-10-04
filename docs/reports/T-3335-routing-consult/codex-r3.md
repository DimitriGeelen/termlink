# T-3335 routing consultation round 3 — Codex

Source: codex-r3.raw (final answer only).

## 1. Is this a distinct problem?

**1a.** Yes: this is **service selection**, not merely locating an identity. “Who currently represents project P as manager?” must be answered before locating that instance and establishing transport. Round 1 supplies reachability; round 2 supplies conversation transport; round 3 supplies responsibility.

**1b.** I retain my conditional support for direct circuits, endpoint durability and exact-instance binding. I would refine the directory proposal: presence, eligibility for a role, appointment to that role and ability to accept work are separate facts. A live agent declaring “manager” does not automatically become the project’s authorized manager.

## 2. Which shape?

**2a.** Prefer **6c’s introduction semantics, implemented initially as a home-hub resolver**, without requiring an elected coordinator process. For a handful of agents, a configured role policy and authoritative role binding should usually answer the question. Introduce a leased coordinator only when assignment requires active scheduling or project-specific decisions.

**2b.** Distinguish singleton roles, such as an authorized project manager, from pools, such as reviewers. A singleton resolves to its current appointed holder; a pool uses an explicit selection policy and recipient admission. Put that policy behind one resolution API rather than making every sender invent it.

**2c.** Reject 6d as an exclusivity mechanism. “Oldest live instance” produces different winners when observers have different membership views. An explicit primary flag also needs an authority governing who may set it. Deterministic selection is useful for distributing requests, but does not establish exclusive ownership.

**2d.** A routing coordinator remains reasonable when it must triage work, aggregate responses or enforce workflow. It should not carry every conversation merely because it introduced the participants.

## 3. Function or agent?

**3a.** The home hub should own authoritative bindings and enforce selection policy. Sidecars register capabilities, report readiness, accept assignments and establish circuits. Neither ordinary resolution nor lease renewal should await an AI turn.

**3b.** AI judgement belongs in ambiguous delegation: interpreting an underspecified request, deciding which expertise matters, or negotiating responsibility. Return an explicit “triage pending” state when such judgement is needed. Do not conceal it as a slow directory lookup.

**3c.** Separate **who answers as manager** from **who operates the resolver**. The manager may be an AI agent; resolving its address should remain deterministic infrastructure. Likewise, an unanswered-inbox monitor is a separate function, not evidence that a coordinator must receive all project mail.

## 4. Failover

**4a.** A hub-held lease is sound **provided one authoritative hub serializes grants durably**, checks renewals and transfers against current ownership, and recovers safely after restart. The supplied description of existing claims does not establish those guarantees; verify them before reuse.

**4b.** A claim id is not automatically a fencing token. Use a monotonically increasing generation, or validate an opaque grant against current authoritative state. Every component accepting coordinator-authorized assignments must reject obsolete grants. Recording a token without enforcing it provides no fencing.

**4c.** During a partition, an old coordinator may still believe it owns the role. Safety means its stale actions are rejected, not that its belief disappears. Cached introductions need bounded validity and validation at admission. If that validation is unavailable, new exclusive assignments may need to pause.

**4d.** Takeover follows expiry plus detection and acquisition time; no numerical TTL is justified by the supplied evidence. Renew independently of AI turns. Hub unavailability means authority unavailable, not permission for another hub to appoint a competing coordinator. Automatic authority failover would require additional coordination.

**4e.** Coordinator takeover does not transfer accepted work. Persist introduction decisions before returning them, keyed by request id. A successor reconciles pending assignments; already accepted work remains with its recipient unless explicitly transferred.

## 5. Addressing

**5a.** Let humans write “ring20-manager,” but resolve it through a scoped alias to a canonical project identity and role selector. Ambiguous aliases return candidates or an ambiguity error. Do not guess a project, session or agent from string similarity.

**5b.** Resolution proceeds: identify project authority; authorize the caller; read role policy and binding; check candidate readiness; select an exact instance; obtain admission where capacity matters; return an introduction. It contains the complete five-level canonical/instance address, binding generation, expiry and scoped circuit authorization.

**5c.** Agents register supported roles and capabilities through authenticated sidecars. Project policy determines eligibility and who may appoint singleton holders. Include actual mail-hub binding and receive readiness: process existence alone is insufficient.

**5d.** Distinguish “unknown project,” “role unassigned,” “eligible agents unavailable,” “capacity exhausted” and “authority unreachable.” An authorized caller may request bounded waiting for a role, with an expiry. Never create an identity implicitly or report an unreachable authority as a dead project.

## 6. With circuits

**6a.** Yes: introduce, authorize, then establish the round-2 circuit to the selected instance. Coordinator failure should not break an established conversation. Retain equivalent authorization, durable acceptance and deduplication across direct and fallback paths.

**6b.** An introduction binds an interaction to an instance; it does not permanently certify that instance as manager. Existing discussion may continue after role reassignment, but actions requiring current managerial authority must validate current authority. A circuit credential must not confer indefinite role privileges.

**6c.** Accepted tasks, exact-instance requests and conversations containing private working context stay bound. Movement requires an explicit handover or a newly authorized interaction with the replacement. Transfer relevant context and reconcile outstanding message ids; never silently resend uncertain commitments to a new holder.

## 7. First slice and negative tests

**7a.** Use two hosts, two configured hubs and two candidate agents for one project. Implement one authoritative singleton binding, authenticated role registration, idempotent introduction records, generation enforcement and circuit setup. Demonstrate normal selection, conversation and controlled transfer. Omit gossip and automatic home-hub failover.

**7b.** Kill the coordinator after persisting an introduction but before replying. Retry with the same request id: return the recorded assignment or its explicit expired/cancelled outcome, never an unrelated second assignment. If it died before persistence, retry may select normally.

**7c.** Partition the old holder, expire its lease and appoint the second. Have both attempt introductions and privileged actions. Obsolete generations must fail at enforcement points, including over an already open circuit.

**7d.** With no live eligible agent, return unavailable/unassigned without inventing a recipient. Stop the authoritative hub separately: return authority unknown/unreachable, never “all agents dead.” Also test a live but incorrectly bound sidecar and require visible receive-readiness failure.

## 8. What is missing?

**8a.** The missing requirement is **responsibility after delivery**. Eight unanswered requests may reflect successful routing followed by overload, unsuitable assignments or absent ownership.

**8b.** Add an application contract: offered, accepted or declined, progress deadline, completed or failed. An overdue request triggers escalation or reconciliation; it does not prove death or authorize duplicate execution. Resolution chooses whom to ask. Explicit acceptance establishes who owes an answer.
