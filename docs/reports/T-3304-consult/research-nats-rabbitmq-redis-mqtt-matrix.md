# T-3304 — How other buses handle topic discovery, metadata, and read-liveness

Legend: [cited] = confirmed from the linked page in this session; [unsure] = from memory or a conflicting source. Verify before relying on it.

## 1. NATS JetStream
- **Metadata location:** stored in the stream/consumer **config**, not as messages. `StreamConfig.Description` (free text) and `Metadata` (map[string]string) exist on both streams and consumers [cited: https://pkg.go.dev/github.com/nats-io/nats.go/jetstream]. The `metadata` map arrived in server **2.10** "to supplant or augment the description field" [cited via search: https://docs.nats.io/release-notes/whats_new/whats_new_210, https://github.com/nats-io/nats-architecture-and-design/issues/200].
- **Survives retention:** yes. MaxAge/MaxMsgs act on stored messages; config is a separate object [inferred from the stream docs, https://docs.nats.io/nats-concepts/jetstream/streams. The docs do not say this explicitly].
- **Discovery:** subject hierarchy with wildcards. `orders.>` captures every subject below `orders.` [cited, streams page]. There is also a STREAM.LIST/INFO and CONSUMER.LIST/INFO API (`nats stream ls`, `nats consumer report`).
- **Read-liveness:** ConsumerInfo carries `Delivered`, `AckFloor` and `NumPending` [cited, pkg.go.dev]. `nats consumer report` shows "last activity" [cited: https://www.synadia.com/insights/checks/nats-inactive-consumer]. As far as I know, `delivered.last_active` and `ack_floor.last_active` timestamps are present in 2.10+ [unsure].
- **Auto-cleanup:** `InactiveThreshold` makes the server remove a consumer that has been inactive for that duration [cited, pkg.go.dev]. Whether it applies to durable consumers is disputed: Synadia says ephemeral only, and "durables must be explicitly deleted"; another source says "prior to 2.9 this only applied to ephemeral consumers", which implies durables are covered from 2.9 [unsure]. The cleanup applies to the **consumer** (the reader), never to the stream.

## 2. RabbitMQ
- **Discovery:** topic exchanges with routing-key wildcards (`*` matches one word, `#` matches zero or more). Listing goes through the Management HTTP API/UI or `rabbitmqctl list_queues`, which report length, rates, consumer count and message states [cited: https://www.rabbitmq.com/docs/queues].
- **Metadata:** queue/exchange *arguments* and policies are broker-side definitions, separate from messages, so a purge never touches them. There is no first-class description field [unsure — some versions have exchange/queue descriptions only via definitions export].
- **"Unused" definition (x-expires):** the queue "has no consumers, … has not been recently redeclared (redeclaring renews the lease), and basic.get has not been invoked for a duration of at least the expiration period" [cited: https://www.rabbitmq.com/docs/ttl].
- **auto-delete:** the queue is deleted when its last consumer is cancelled or gone, but "if a queue never had any consumers … it won't be automatically deleted". Exclusive queues die with their connection [cited, queues page].

## 3. Redis Streams
- **Metadata:** none beyond the stream key. Any description is a convention (e.g. a sibling hash key) [unsure — no native field].
- **Read-liveness, all native:**
  - `XINFO GROUPS` returns `consumers`, `pending` (PEL), `last-delivered-id`, `entries-read` and `lag`. Lag is `entries_added − entries_read` and is NULL when it is unknowable (after an arbitrary SETID, or when entries were deleted or trimmed in the range) [cited: https://redis.io/docs/latest/commands/xinfo-groups/].
  - `XINFO CONSUMERS` returns `pending`, `idle` (ms since last *attempted* read) and `inactive` (ms since last *successful* read). The split arrived in **7.2.0** [cited: https://redis.io/docs/latest/commands/xinfo-consumers/].
- **Cleanup:** no automatic expiry of idle groups or consumers. `XGROUP DELCONSUMER` is manual. Trimming (MAXLEN/MINID) removes entries only [unsure on wording; behaviour well known].

## 4. MQTT (+ Sparkplug, Homie)
- **Discovery:** topic hierarchy with `+` (one level) and `#` (multi-level) wildcards. The protocol has no topic listing.
- **Retained messages:** the broker keeps the last retained message per topic and hands it to new subscribers. It is deleted by publishing a zero-byte retained message [cited: https://www.hivemq.com/blog/mqtt-essentials-part-8-retained-messages/]. This is effectively a "current state" slot that sits outside any stream retention. `$SYS/#` broker stats are a widespread but non-normative convention [unsure].
- **Homie v5:** devices publish retained `$`-attributes. `$state` holds the lifecycle and `$description` holds a JSON document of nodes and properties. Controllers "must by default perform auto-discovery on the wildcard topic `+/5/+/$state`" [cited: https://homieiot.github.io/specification/]. In short, the metadata is a retained message on a reserved sibling topic, discovered by wildcard.
- **Sparkplug B:** namespace `spBv1.0/{group}/{type}/{node}/{device}`. NBIRTH/DBIRTH declare every metric with its datatype. STATE is the host-application liveness signal [cited: https://sparkplug.eclipse.org/specification/version/3.0/documents/sparkplug-specification-3.0.0.pdf, via search summary]. As I recall, BIRTHs are *not* retained, and a consumer that missed one requests a "rebirth" via NCMD. In 3.0, STATE is retained [unsure].

## 5. Matrix
- **State vs timeline:** `m.room.topic` is a **state event**. `/sync` returns `state` and `timeline` as separate sections [cited: https://spec.matrix.org/latest/client-server-api/]. Current room state is a keyed map, `(type, state_key) → event`, so it survives pagination and gaps.
- **Retention (MSC1763 / Synapse):** "message retention policies don't apply to state events". Synapse also never deletes the last event in a room, but hides it [cited: https://element-hq.github.io/synapse/latest/message_retention_policies.html; MSC: https://github.com/matrix-org/matrix-spec-proposals/pull/1763]. The retention policy is itself a state event, `m.room.retention`.
- **Discovery:** the room directory, `GET/POST /_matrix/client/v3/publicRooms`. It returns per-room name, topic, alias and num_joined_members [endpoints cited; field list from memory, unsure].
- **Read signal:** read receipts / `m.fully_read` per user. There is no server-side "room is dead" signal [unsure].
- **The TermLink lesson:** TermLink copied `m.room.topic` but stored the description as a *timeline* message. Matrix's core design decision is that it is *state*, exempt from retention.

## 6. Google Cloud Pub/Sub (managed reference)
- **Metadata:** `labels` (key/value) on topics and subscriptions are resource metadata, separate from messages [unsure — not on the fetched page; standard GCP resource labels].
- **Retention:** subscription `message_retention_duration` defaults to 7 days (min 10 min, max 31 days). Topic-level retention stores each message once for all subscriptions [cited: https://docs.cloud.google.com/pubsub/docs/subscription-properties].
- **Inactive readers:** `expiration_policy` defaults to **31 days**. A subscription expires "without subscriber activity or property changes". Activity means "open connections, active pulls, or successful pushes" [cited, same page]. A topic with zero subscriptions remains, and there is no auto-delete of topics [unsure].

## Cross-system pattern
1. Metadata lives in a **separate config/state store** (NATS config, RabbitMQ definitions, Matrix room state, GCP labels, MQTT retained slot), never in the retention-governed log.
2. Read-liveness is tracked **per reader object** (consumer, group, subscription, queue). The bus keeps last-activity timestamps server-side. Clients do not own the only copy.
3. Dead-detection is **reader-centric with TTL leases**. Activity renews a lease (RabbitMQ x-expires, Pub/Sub expiration_policy, NATS InactiveThreshold), and expiry removes the *reader*. Topics are dead when they have zero live readers plus no recent writes.
4. Discovery = **hierarchical names + wildcard subscribe**, plus a server-side LIST/INFO API. Self-description uses a reserved, retained attribute topic (Homie `$description`).
