# T-3335 routing consultation round 2 — GLM-5.3

Source: glm-r2.raw (ANSI and banners stripped).



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
