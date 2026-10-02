You are being consulted as an independent design reviewer. Give your own view; disagree with the framing where you think it is wrong. Answer in English, in under ~900 words, using the numbered headings below. Use hierarchical labels (1, 1a, 1ab) for lists, never plain bullets. Do not modify any files.

## The system

TermLink is a coordination substrate for a fleet of AI coding agents (and one human operator) across several machines. Charter: "a hub-mediated, durable append-log message bus with terminal endpoints — lets a fleet of AI agents discover each other, exchange durable messages, claim work, and control terminal sessions." Charter non-goals include: not a durable database / system of record (topics are retention-bounded), and not a second cross-host bus (hubs are independent; no federation).

Several projects (each with its own agent) run on one host and share one hub. Most use a governance framework, "AEF", maintained by a separate agent. Messages between agents travel as hub topic posts (`inbox:<hub>/<project>`, `dm:<fp>:<fp>`), each with metadata (from_project, conversation_id, client_msg_id, from_circuit, and now to_circuit).

## The receive side today (measured)

1. TermLink's own receive side is shell scripts ("sidecars") run from one git checkout; releases ship only the binary.
   1a. "received": a per-topic watermark receipt (`msg_type=receipt`, `stage=delivered`, `up_to=<offset>`) posted by a poller every 15 s. No per-message id or timestamp field; time = hub envelope time.
   1b. "injected" (message put into the agent's session): never emitted — the component that injects is scheduled by nothing, because no agent session is registered with an injectable terminal.
   1c. "replied": not emitted.
   1d. Receipts are durable envelopes on the topic (all kept, subject to retention), but the hub's summary view keeps only the latest per sender.
2. AEF has built its own receiver (Python, per agent): an HTTP API on 127.0.0.1 with a bearer token. The sender calls the recipient's receiver DIRECTLY (POST /message, synchronous "RECEIVED" + timestamp), bypassing the hub; the hub is only a fallback (topic post, which currently sends no receipt), plus planned cross-host discovery. States: SENT, RECEIVED, HANDED_OVER (only after the agent's prompt hook surfaced it), REPLIED, UNDELIVERABLE, REJECTED, ESCALATED. Timestamps go to a per-project ledger file (`.context/sidecar/direct-ack.jsonl`), readable only on that project's disk. Same-host only until a later slice publishes receiver endpoints to the hub.
3. Incident that triggered this: a peer agent waited ~1 day for answers; the TermLink sidecar had receipted its mail as delivered, but nothing put it in front of the agent. Nobody could see the stall.

## The operator's requirements

R1. Every stage is a timestamped API call back to the sender: RECEIVED as soon as stored, INJECTED when put into the session, REPLIED when answered. All timestamps stored.
R2. A communication-telemetry STANDARD for all agents (ours and every other project's): every step and the delay between steps, for the sidecar path and other inbox paths.
R3. Telemetry must be PULLABLE from other agents, and every agent also OFFERS it regularly (about daily).
R4. Agents work with that telemetry and reflect on what it means.
R5. All sidecars should ship with every TermLink deployment.

## The decision (IW-1): who owns the receive-side stages and the telemetry record?

Options under consideration:
A. AEF's receiver owns it all; TermLink only transports messages and injects into sessions; telemetry lives in each project's ledger.
B. TermLink owns it all: move the whole receive side into the termlink binary with AEF's state names; ask AEF to retire its receiver.
C. Split by layer: TermLink keeps a per-message telemetry record on the hub (client_msg_id, stage, timestamp; all events kept), a pull verb, and a daily per-agent digest; each receiver (AEF's, or TermLink's) reports its stages into it using one vocabulary. Because AEF's primary path is a direct HTTP call that never touches the hub, C requires direct-call receivers to MIRROR their stage events to the hub asynchronously (never in the message's path; buffer locally if the hub is down).
D. Defer.

## Questions

1. Which option would you choose, and the single strongest argument against your choice?
2. Is the hub the right home for cross-agent telemetry given the charter's non-goals (system of record; second bus)? If not, where?
3. What breaks first in each option (failure modes), and how would you detect it?
4. For R3 (pull + daily offer): what should the record and the digest contain, minimally, so an agent can actually act on it (not just store it)?
5. What is missing from this framing — a risk, an option, or a requirement nobody stated?
