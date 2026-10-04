# T-3335 routing consultation round 2 — Codex

Source: codex-r2.raw (final answer only).

## 1. Did round 1 miss this?

**1a.** Yes. I underweighted the requirement for a long-lived, bidirectional conversation. A circuit provides session continuity, streaming and conversation-scoped flow control. Durable mail provides recoverable delivery. These are different requirements that can coexist: a circuit can carry durable messages and transient updates.

**1b.** I revise “do not make direct sidecar connections the preferred transport yet.” My position is now: **direct circuits are a reasonable preferred transport for established conversations, provided their delivery contract survives reconnection and fallback.** They deserve a prototype, not rejection until hub relay is built first.

**1c.** I also revise “Use one sender API and one owner of retries: sender submits to its local hub.” Keep one API and retry coordinator, but that coordinator can be the sidecar, with durable local state. I retain: “Injection does not prove comprehension or execution,” and the requirement for atomic destination deduplication.

## 2. Steelman the circuit

**2a. Latency and hops.** Direct sidecar transport removes hubs from the remote data path and avoids their application queues. An established connection also avoids repeated connection setup, although hubs can maintain persistent connections too. The supplied 85–111 ms measures wake delivery, not comparable end-to-end conversation latency. For turns taking seconds or minutes, modest transport savings probably matter less than reliable injection; for rapid coordination and streaming, they may matter substantially.

**2b. Load and independence.** Direct transport moves payload processing and buffering off hubs. The benefit depends on traffic volume, not merely agent count. Existing circuits can survive hub outages if credentials remain valid and sidecars own acceptance and recovery. If every acknowledgment requires a hub write, that independence disappears.

**2c. Flow control and state.** A circuit naturally holds negotiated capabilities, conversation identity, sequence progress, cancellation and receiver capacity. However, transport backpressure only measures buffer capacity. Explicit application credits must represent what an agent can actually consume.

**2d. Streaming, privacy and parties.** Streaming partial answers is a real benefit if the receiving harness can use them incrementally. Direct transport can keep content out of hubs, but encrypted payloads through a relay can also do that. Multi-party communication is possible, not automatically simpler. None of these features is impossible through hubs; the strongest circuit argument is simpler endpoint ownership and less dependence on intermediaries.

## 3. Circuit design, if built

**3a. Setup.** The initiating sidecar asks its hub to resolve an authorized target. Signalling establishes exact endpoint instances, conversation identity, capabilities and candidate addresses. Both endpoints authorize the peer; a directory answer alone is not permission to connect.

**3b. Transport.** Start with one persistent TLS/TCP connection per communicating pair where network access permits. WebSocket is useful for HTTP-compatible infrastructure; it does not itself solve NAT traversal. Consider QUIC for independent streams or network migration only when those requirements justify it. Negotiate connection direction so either reachable endpoint can accept; use a relay when neither can.

**3c. Credentials.** Use short-lived, per-circuit grants binding both endpoint identities and instances, permitted operations, expiry and endpoint public keys. Require proof of key possession. Sidecars need issuer trust, not fleet-wide shared secrets. Define whether established sessions survive token expiry and how long revocation may take effect during hub outages.

**3d. Lifecycle and confirmation.** Keepalive detects an unusable connection, not a dead agent. Re-establishment authenticates endpoints again and exchanges committed progress. Graceful teardown drains acknowledged work; abrupt loss leaves outstanding records pending. Distinguish transport receipt, durable acceptance, session injection and explicit agent acknowledgment. Confirm meaningful application stages, not every network frame.

## 4. Multi-party

**4a.** For small groups, use pairwise circuits and sender fan-out. Full mesh has quadratic connection growth, but may be perfectly adequate for a handful of participants. Track delivery independently for each recipient.

**4b.** A star through one sidecar reduces connections but makes that agent’s lifecycle a group dependency. I would avoid an arbitrary participant becoming the durable group coordinator.

**4c.** Larger groups may warrant a bridge. A hub is a reasonable bridge if it already offers availability, authorization and queue management. This does not imply shared-topic replication. Treat bridging as an explicit service, and specify membership changes and ordering. A shared total order requires coordination somewhere; pairwise streams do not provide it.

## 5. Durability and the two-path problem

**5a.** Classify conversation traffic. Final messages, commands and commitments need durable acceptance. Typing indicators and replaceable partial output may be transient. Reliable streaming requires numbered chunks and an explicit completion record; otherwise promise recovery of the final message only.

**5b.** For durable traffic, the sender sidecar records an outbox entry before sending; the receiver atomically records payload and deduplication identity before acknowledging. Direct and relayed traffic must reach this **same logical acceptance ledger**. Hub storage during fallback means “queued at hub,” not “accepted by receiver.”

**5c.** An asynchronous hub copy can provide telemetry, but cannot guarantee recovery of a message acknowledged before that copy exists. If surviving loss of the receiver’s disk is required, acknowledgment needs stronger storage guarantees. Process restart durability is a different promise.

**5d.** On circuit failure, resume with the same message identifiers, bound instances and per-sender sequence numbers. Fallback and direct attempts may race; the receiver accepts once and returns the existing receipt. Reconcile missing records before advancing ordered delivery, with explicit gap handling. Do not silently redirect an uncertain operation to a replacement instance. Local deduplication still cannot guarantee exactly-once external actions.

## 6. Your verdict now

**6a.** **Conditional yes:** hubs as signalling and fallback, direct circuits preferred once established. This better matches the operator’s intended product than my original hub-first prescription.

**6b.** Conditions are endpoint-owned durable recovery, equivalent authorization across paths, explicit receipt semantics and manageable network exposure. Preference should depend on measured conversation responsiveness, streaming utility, hub-outage continuity and operational burden. There is no useful hub-count threshold by itself.

**6c.** “The hub cannot help once established” remains too strong. A persistent hub-mediated stream can provide the same conversation abstraction. Compare implementations with equivalent guarantees, not a warm direct connection against repeated cold mail submissions.

## 7. First slice and test

**7a.** Build two sidecars on two hosts, hub-mediated authorization and exact-instance resolution, one persistent TLS circuit, durable endpoint outboxes/inboxes, and fallback through destination-hub storage. Limit scope to one-to-one conversation; omit gossip and automatic role reassignment.

**7b.** Have real agents conduct repeated question-and-answer exchanges. Compare warm direct and warm hub-mediated paths. Measure median and tail durable-acceptance latency, injection latency, complete-turn time, hub resource use and recovery time. Separately test streaming with harness-consumable partial output.

**7c.** Drop the circuit after receiver persistence but before acknowledgment. Fallback must deliver one inbox entry, preserve promised ordering and reconcile the receipt. Restart a sidecar to verify recovery.

**7d.** Stop a hub while a circuit remains active: conversation should continue within the credential policy. Then break that circuit: report unavailable signalling or queued traffic, never “agent dead.” If direct transport offers no meaningful benefit, that disproves its priority, not its feasibility.

## 8. What is still a reason not to?

**8a.** Sidecars become network services and durable protocol endpoints, increasing upgrade, storage and security responsibilities. Direct circuits also trade immediate centralized revocation for outage independence.

**8b.** The strongest reason to decline is practical: restrictive networks force frequent relay, harnesses cannot exploit streaming, and measured interaction barely improves. The strongest reason to proceed is equally practical: conversations continue independently of hubs with demonstrably better interaction and understandable recovery.
