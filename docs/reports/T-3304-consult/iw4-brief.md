You are being consulted as an independent design reviewer on ONE decision. Give your own view and disagree where you think the framing is wrong. Answer in English, under ~800 words, with the numbered headings below. Do not modify any files.

## The system (short)
TermLink is a small message bus for a fleet of tens of AI agents on a few machines. Each machine runs a hub; topics are append-only logs with numbered offsets. Readers keep their OWN cursor (the hub does not track reader positions). Already decided by the operator: the hub is authoritative for coordination within a declared retention window (not an archive); new topics default to 14-day retention, "forever" needs an owner and a reason; a background sweeper enforces retention on every hub.

## The decision: topic metadata, discovery, and "is anyone reading?"
Measured facts:
- The hub stores per topic only: name, retention, created_at. `list` shows count, latest offset, name, retention.
- A topic description exists only as an ordinary message inside the topic (copied from Matrix m.room.topic). Retention sweeps can therefore delete it. 12 of 113 topics have a description.
- The hub has no record of who reads a topic or when (a per-reader server table exists but holds 2 rows; readers track cursors client-side). A health-probe topic grew to 2,777 records with nobody able to tell whether anyone read it.
- Topics are auto-created by known name patterns (dm:*, inbox:*, agent-presence, state:*) and by a generic --ensure-topic flag.
- Discovery today: list all topics, filter by name prefix; no tags, no owner, no wildcard subscriptions.

Research on other systems (summarised, sources available):
- Metadata lives OUTSIDE the message log everywhere: Kafka cluster config; Pulsar topic properties; NATS stream config (Description + Metadata map, 2.10+); Matrix keeps m.room.topic as room STATE, which Synapse retention does not apply to.
- Kafka has no native description/owner; teams use naming conventions + an external catalog (Confluent Stream Catalog asks every topic for description + owner). AsyncAPI describes channels in a spec file (drifts from reality).
- "Is anyone reading" is server-side per reader everywhere: Kafka consumer groups (committed offset, lag; invisible if clients keep their own offsets), Pulsar subscriptions (lastConsumedTimestamp, lastAckedTimestamp, backlog), Redis XINFO (idle), NATS consumer info.
- Dead-topic rules: Pulsar deletes inactive topics (no producers/consumers, all subscriptions caught up, nothing published for N s) — and that deletion also removes the topic's properties/schema. RabbitMQ x-expires and Google Pub/Sub expiration_policy (31 days default) expire unused subscriptions.
- Kafka auto.create.topics (default on) is the classic sprawl mistake: typos create ownerless topics.
- Discovery elsewhere: hierarchical names + wildcard subscriptions (NATS `orders.>`, MQTT `+`/`#`); MQTT Homie devices publish a self-description on reserved retained topics.

Options under consideration:
A. Nothing (descriptions stay optional in-log messages).
B. Hub-side catalog, optional: description, owner, tags, last-post time stored in the hub's own topic table (outside the log, survives retention); `list` shows them and filters by tag/owner.
C. B with owner + one-line purpose REQUIRED on explicit creation; auto-filled by the hub for known auto-created name patterns; existing topics backfilled by pattern. Refined by the research: (1) the hub records per READER (by signed identity) when it last fetched from each topic, without making cursors server-side; (2) a "dead topic" = no read and no write for N days → FLAGGED for operator review, never auto-deleted.
D. C plus wildcard/hierarchical subscriptions and a "topics relevant to me" query by project/capability.

The agent recommends C (with the refinements); wildcard subscriptions deferred until an agent needs them.

## Questions
1. Which option would you choose, and the strongest argument against your choice?
2. Is "required owner + purpose" the right enforcement, or will it be filled with junk? What would you do instead?
3. Per-reader last-fetch tracking without server-side cursors: sound, or should the hub own cursors (consumer-group style)? Trade-offs.
4. The dead-topic rule: what N, flag vs auto-delete, and what should count as "activity"?
5. Wildcard subscriptions / hierarchical names: now, later, or never for a fleet of this size?
6. One thing this framing misses.
