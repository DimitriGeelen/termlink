# T-1155: Channel-based communication bus for TermLink agents

**Status:** inception / research in progress
**Task:** T-1155
**Started:** 2026-04-20
**Related:** T-163, T-690, T-908, T-1074, T-1101, T-945, T-1017, T-1018, T-1051..T-1058

## One-line framing

> **Can a channel-based bus subsume `event.broadcast` + `inbox` + `pickup envelopes` + `send-file` into one persistent, offline-tolerant abstraction, without adding a new liveness domain?**

This is the single go/no-go question. Deployment model (separate service vs inside hub), transport choice, and auth story are **outputs** of the inception, not inputs.

## Why now

Three operational pains, surfaced by the user, that a shared messaging primitive might unify:

1. **Liveness** — agents or hubs frequently offline. Current mitigations (cron jackset, startup units) are partial. Every abstraction that *requires* liveness compounds the pain.
2. **Auth/secret propagation** — rotations break trust between agents (T-1051 lineage). Every new trust domain makes it worse.
3. **Discoverability** — agents often don't know termlink is available, or what's going on across the fleet. No "fleet chat" equivalent exists.

The user's proposal: a shared comm bus — channels, 1:1, groups, messages + artifacts, like Signal/WhatsApp for agents. Deployed as a separate service.

## What we already have (fragmented partial bus)

| Primitive | Shape | Known gaps |
|---|---|---|
| `event.broadcast` + `event.poll` | Live pub/sub via hub | No persistence, no replay, no topic registry |
| `inbox` | Per-session mailbox | T-1017 silent drop, T-1018 stale chunks, per-session only |
| `kv` | Shared state | Not messaging, no change events except via poll |
| `pickup envelopes` | Async one-shot | No reply channel, one-way, no ordering |
| `send-file` | Artifact transport | T-953/PL-011 ok-on-accept ≠ ok-on-delivery |
| `termlink inject` | Direct prompt delivery | Requires receiver-up-at-send-time |

A bus done right **subsumes** these. A bus done parallel **duplicates** them.

## Assumptions to test (register via `fw assumption add`)

- **A-001** Agents want channels, not just 1:1 sessions. (Evidence we'd need: look at pickup/inbox traffic patterns for clustering.)
- **A-002** Persistent history + cursor is worth the storage+complexity cost. (Alternative: live-only pub/sub with no replay.)
- **A-003** A single trust anchor for the bus is achievable without making hub rotation harder. (Alternative: per-hub secrets inherited; then why a bus?)
- **A-004** Offline-tolerant posting is feasible — agents can queue when bus is down and replay on reconnect. (Alternative: bus becomes a new SPOF.)
- **A-005** Migration from `event.broadcast` + `inbox` + `pickup` is tractable. Need to count live call sites and estimate churn.

## Decision criteria (must all land for GO)

1. **Subsumption clear** — at least `event.broadcast` and `inbox` replaced by the channel abstraction; `pickup envelopes` and `send-file` either replaced or cleanly reduced to special cases of channel post.
2. **No new liveness domain** — the bus either runs inside the hub (already required) or is strictly optional (clients degrade to local queue when bus unreachable).
3. **Auth story plausible** — either (a) reuse hub secret per-hub with a federation layer, or (b) introduce a single fleet-wide identity that *replaces* per-hub rotation rather than adding to it.
4. **Migration path exists** — concrete plan for moving the ~N known call sites off the legacy primitives without a flag day.
5. **Storage model chosen** — log-append (Kafka-style, replay from cursor) vs rolling buffer (IRC-style) vs TTL queue (SQS-style). Each has different ops costs.

Any one of these unresolved = NO-GO / defer.

## Exploration plan (time-boxed spikes)

**S-1. Existing-primitives call-site census (30 min).**
- Count callers of `event.broadcast`, `inbox`, `send-file`, `pickup` across termlink + framework
- Classify by pattern: broadcast, 1:1, persistent-state-change, artifact-transfer
- Output: table of patterns → proposed channel shapes

**S-2. Persistence model decision spike (45 min).**
- Sketch three storage models (log-append, ring, TTL) with space/retrieval characteristics
- Map each to the 3 user pains and to the 4 subsumption candidates
- Output: ranked recommendation + disqualifiers

**S-3. Liveness/offline-tolerance spike (30 min).**
- Can clients queue posts locally when bus is down? What would a "local tail + sync on reconnect" look like?
- Does this mean every client has a mini-bus? Or an opaque queue?
- Output: viability verdict on A-004

**S-4. Auth integration sketch (30 min).**
- Three candidates: (a) bus trusts hub secrets transitively; (b) bus has its own secret per-channel; (c) fleet-wide identity issued by bus, hubs become relays
- Evaluate against T-1051 lineage + rotation pain
- Output: ranked recommendation + disqualifiers

**S-5. Migration scope estimate (15 min).**
- From S-1 output, estimate work to move each call site
- Output: go/no-go on criterion 4

Total exploration budget: **≤ 3h**. Hard stop — if not done, we're descoping, not extending.

## Scope fence

**IN:**
- Evaluating subsumption of existing primitives
- Exploring persistence, auth, liveness models at sketch level
- Migration-scope sizing

**OUT (separate tasks if GO):**
- Implementing the bus
- Specific wire protocol design
- UI/watchtower integration
- Channel ACL model beyond sketch
- Cross-cluster federation
- Non-agent uses (e.g., human-to-agent pager)

## Related prior art

- **Signal/WhatsApp** — the user's reference point. Session state, channels, artifacts, offline queue. But: centralized server, strong identity, E2E crypto. We'd reuse the UX vocabulary, not the architecture.
- **IRC** — simplest channel model. No persistence, strict liveness. Good lower bound for "how little could we do and still be useful."
- **Slack/Discord** — channels + persistence + reactions + threads. Too heavy for our use case as a baseline but useful as a vocabulary map.
- **Kafka** — log-append with per-consumer cursor. Best fit for "replay what you missed" but operationally heavy.
- **NATS/Redis pub/sub** — live-only pub/sub. Closest to current `event.broadcast`. Lower bound for bus-that-is-just-broadcast.
- **Matrix** — federated messaging protocol. Relevant if we want cross-hub without a central coordinator.

## Dialogue Log

### 2026-04-20 — initial user prompt

> *User:* "We keep having difficulties with establishing reliable communication protocol between termlink agents. One cause is that often the agent or the hub is not online. Another issue is authentication, certificates, secrets not being being recreated, not being able to share between agents. A third one is the agent not knowing that it can use termlink. Can we create a shared communication bus, like WhatsApp or Signal for termlink agents? Where they can create channels they can subscribe to, post messages and artefacts in. Also one-to-one communication. Set this up as a separate service."

> *Agent reflection (summarized):* Three pains are distinct. Bus genuinely addresses discoverability; moves liveness pain unless it subsumes the hub; compounds auth pain unless it replaces per-hub secrets. Recommend inception scoped to subsumption question rather than the general "should we build it" framing. User: "yes".

### Next dialogue checkpoint

To be filled when spike S-1 output is ready for discussion.

## Recommendation

*[to be written after spikes S-1..S-5 complete]*

## Decision

<!-- inception-decision -->

*[to be filled via `fw inception decide T-1155 go|no-go`]*
