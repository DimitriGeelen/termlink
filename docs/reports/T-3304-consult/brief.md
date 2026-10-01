You are being consulted as an independent design reviewer. Give your own view; disagree with the current design where you think it is wrong. Answer in English, in under ~900 words, using the numbered headings below. Do not modify any files.

## The system

TermLink is a coordination substrate for a fleet of AI agents (and humans) across several machines. Its charter sentence: "a hub-mediated, durable append-log message bus with terminal endpoints — lets a fleet of AI agents discover each other, exchange durable messages, claim work, and control terminal sessions."

Facts about how messages are stored today:
- Each machine runs a hub. Hubs are independent: a topic named X on hub A and on hub B are unrelated (no federation, by design).
- A topic is an append-only log. Every message gets the next offset (0,1,2,...). Offsets never rewind, even after old records are deleted.
- Retention is set per topic: forever, keep last N messages, keep N days, keep only the latest, or keep the latest per key (metadata.cv_key). Pruning ("sweep") is explicit: nothing runs in the background; an operator cron must run it.
- On one hub today: 111 topics, 6,698 records; 75 topics are "forever", 36 are bounded. The largest is a health-probe topic at 2,777 records, kept forever, that nobody reads.
- Agents read with "subscribe(topic, from offset N, optional filters: conversation_id, in_reply_to, limit)". The client remembers its own cursor. The hub jumps straight to offset N via an index, so a plain read is instant; with a filter the hub must open each record to test it, which is the only slow path (capped by a 20 s server-side deadline).
- Topic discovery: list all topics (optionally by name prefix), topic info/describe, a substring/regex search inside one topic, a "current values per key" view, and snapshots. There is no cross-topic query, no time-range query, and no query language.
- Big binary payloads do not go through topics; files are chunked over a separate artifact transfer.
- The charter currently has a non-goal: "Not a durable database or system of record. Topics are retention-bounded append logs sized for coordination, not archival. Durability means 'survives a hub blip and replays', not 'stored forever'."

## Questions

1. Should the hub be a queryable source of truth, or should it stay a transport where agents keep whatever history they need? Give your recommendation and the strongest argument against it.
2. If the hub keeps history: what retention and pruning model would you use (defaults, who decides, automatic vs explicit sweep), and why?
3. Subscription and query model: is "cursor + optional filter" enough? Should there be time-range queries, cross-topic queries, server-side indexes on metadata, pagination/chunking rules? What would you add first, what never?
4. Topics: how should an agent discover and select topics (naming conventions, tags, registry, per-topic descriptions)? How should it ask for the topics relevant to it?
5. Cost: what does each choice cost (storage, CPU on the hub, complexity, failure modes) and where would you draw the line for a fleet of tens of agents on a few machines?
6. One thing in this design you think is a mistake that nobody asked about.
