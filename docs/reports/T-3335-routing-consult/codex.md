# T-3335 routing consultation — Codex

Source: codex.raw (codex exec -s read-only; rc=0), final answer only; prompt echo and tool trace removed.

## 1. The split

**1a. Verdict:** Separate discovery and presence from message transport, but do not make direct sidecar connections the preferred transport yet. My baseline would be: local hub accepts durable responsibility, destination hub accepts delivery responsibility, destination sidecar injects the message. Add a direct transport only when measurements justify it.

**1b. Biggest weakness:** The proposal treats connection establishment as the hard part; your requirement is **durable conversation with meaningful confirmations**. Resolving an address does not establish who retains a message, retries it, or proves that the agent received it. Those responsibilities must remain identical across transports.

**1c.** “Hubs never synchronize messages” needs precision. For reliable handoff, origin and destination may temporarily retain the same message until an acknowledgment arrives. Prohibit shared-topic replication, not overlapping durable copies during transfer. The assertion that relay involves “no consistency model” is too strong: custody, deduplication and receipts still require a distributed protocol.

## 2. Protocol models

**2a.** I agree that IP routing is the wrong closest fit under the stated topology. You need identity resolution and durable handoff, not shortest-path computation. Borrow scoped advertisements, route expiration and loop prevention; do not import RIP, OSPF or BGP machinery without a multi-hop requirement.

**2b.** E-mail’s store-and-forward model and XMPP’s home-server model are closer to the messaging requirement: authoritative destinations, authenticated server handoff, offline retention and explicit failures. Borrow those concepts without assuming their delivery semantics meet your needs unchanged.

**2c.** DNS provides useful caching ideas: authoritative answers, bounded lifetimes and negative caching. Expired or missing presence must mean “refresh or unknown,” never “dead.”

**2d.** SIP provides a useful registration and session-establishment analogy. ICE/STUN/TURN become relevant if you actually introduce endpoint connections across restrictive networks. Media transport is a poor model for durable message settlement; establishing a circuit does not solve that problem.

**2e.** Gossip membership is optional. At 5–20 hubs, start with configured peers and authoritative lookup, perhaps with subscriptions for changes. Gossip can distribute observations, but its “dead” classification must not become proof that a remote agent has terminated.

## 3. Liveness

**3a.** Separate **reachability observations** from **instance lifecycle**. Use alive, suspect and unknown for current observations; use terminated or fenced for authoritative terminal states. A timed-out heartbeat is suspicion, even when observed by the home hub.

**3b.** The home hub is authoritative for registrations within its namespace. It may declare an instance terminated after observing its exit, receiving authenticated deregistration, or revoking its lease and enforcing that revocation. Lease expiry alone does not prove the process stopped.

**3c.** No universal timeout follows from the supplied information. Detection thresholds should reflect heartbeat cadence and tolerated pauses. Faster timeouts increase false suspicions; terminal declarations require evidence or enforceable fencing.

**3d.** Senders retry suspect or unknown destinations within a message deadline. A terminal declaration ends further attempts to that exact instance. If an earlier attempt may have succeeded, report “delivery outcome unknown; instance terminated” until reconciled. A dead letter must not falsely imply non-delivery.

## 4. Directory contents

**4a.** Advertise serving relationships across all five levels: host, hub, project, session and agent, including canonical identifiers and the relevant instance identifiers. Include authoritative hub, endpoint, registration revision, lifecycle state, lease validity, supported protocol version and access scope.

**4b.** Do not imply that one hub owns every occurrence of a canonical project or agent identity. Several hubs may serve different instances. Authority belongs to an explicitly scoped registration; canonical identity alone does not establish exclusive ownership.

**4c.** Export only authorized namespaces. Prefer targeted lookup over fleet-wide enumeration where practical. Cache answers for bounded durations; preserve issuer and revision, reject older updates, and retain terminal tombstones long enough to prevent stale advertisements resurrecting an instance.

**4d.** Role resolution needs a declared selection policy: designated primary, capability match, load distribution or explicit user choice. Reject ambiguity where no policy exists. Bind the selected instance to the message before transmission. Retargeting after an uncertain attempt risks execution by two agents.

## 5. The direct path

**5a.** Initially, “direct” should mean a connection to the destination hub. That preserves the existing network boundary and durable inbox. The reported 85–111 ms median is a baseline, not evidence that another listening surface is warranted.

**5b.** Sidecar-to-sidecar transport requires endpoint discovery, authenticated identity, authorization, encrypted connections, credential rotation, firewall traversal, durable acceptance and recovery. Hub-issued, narrowly scoped credentials could reduce credential distribution, but introduce their own issuance and revocation rules.

**5c.** A sidecar must commit acceptance durably before acknowledging it, preferably through the same destination inbox used by relay. Otherwise the “fast” path has weaker guarantees.

**5d.** Do not prefer sidecar transport when it bypasses access controls, cannot durably accept, exposes unnecessary ports, or produces negligible end-to-end benefit. Direct transport should be a measured optimization.

## 6. The fallback ladder

**6a.** I would reject the proposed three-stage ladder initially. Use one sender API and one owner of retries: sender submits to its local hub; that hub resolves and hands off to the destination hub. An unavailable destination leaves the message queued locally.

**6b.** If client-to-destination or sidecar transport is later added, retain one durable sending coordinator. A transport failure triggers another attempt under the same message identity, not a new send with independent retry ownership.

**6c.** Expose distinct receipts: accepted locally, stored at destination, accepted by sidecar, injected into session, and acknowledged by agent. Injection does not prove comprehension or execution. Each receipt needs a precise issuing authority.

**6d.** Use a stable identifier scoped to the authenticated sender, bound destination instance and payload hash. Destination insertion and deduplication must be atomic. Repeated submissions return the existing result; identifier reuse with different content is rejected.

**6e.** Define maximum retry age and retain deduplication records beyond it. Lost acknowledgments cause reconciliation or duplicate submissions, never a second inbox entry. This does not guarantee exactly-once external actions by the agent.

## 7. First slice and test

**7a.** Ratify addressed handoff separately from directory distribution and correct the unsupported charter blessing. Permit only the required peer behavior in the tripwire. Broad presence exchange need not be bundled into this decision.

**7b.** Build two configured hubs, authenticated handoff, a durable outbound queue, destination deduplication, exact-instance lookup and durable receipts. Avoid gossip, automatic role selection and sidecar listeners in this slice.

**7c.** Have two real agents exchange a question and response across hosts. Verify each receipt through actual agent acknowledgment. Drop the destination acknowledgment after storage, then restart the origin: retry must produce one destination inbox entry.

**7d.** Terminate an agent while its hub remains available: sending to that instance must return an authoritative terminal result. Separately stop or isolate its hub: sending must remain queued with unknown reachability, never “agent dead,” and resume after recovery. Measure latency against the current path.

## 8. What is missing

**8a.** Work claims and terminal control require stronger semantics than conversation. A delayed, previously valid command may be harmful after ownership changes. Include operation expiry, authorization at execution time and fencing tokens for exclusive ownership. Durable delivery alone is insufficient.

**8b.** Specify queue limits, backpressure and retention. A partition must not allow one unreachable destination to exhaust a hub and disrupt otherwise healthy conversations.
