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
