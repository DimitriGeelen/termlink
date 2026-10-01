# T-3304 research: topic discovery, metadata, and "is anyone reading" in Kafka / Pulsar / catalogs

Legend: [cited] = confirmed from a source this session; [known] = standard documented behaviour, not re-fetched; [unsure] = flagged.

## 1. Apache Kafka

**Discovery.** `AdminClient.listTopics()` returns the names (internal topics are hidden unless `listInternal`), `describeTopics()` returns partitions, leaders and the topic UUID (KIP-516), and `describeConfigs(ConfigResource.TOPIC)` returns per-topic config such as `retention.ms` and `cleanup.policy`. All of it lives in cluster metadata (ZooKeeper, or the KRaft `__cluster_metadata` log). It is separate from the data log, so retention never deletes it. [known] https://kafka.apache.org/documentation/#adminapi · https://cwiki.apache.org/confluence/display/KAFKA/KIP-516:+Topic+Identifiers

**Native description, owner or tags: none.** A topic has a name, an ID, partitions and a config map. There is no free-form description, owner or label field, and a KIP search found nothing that adds one. [cited, but absence is hard to prove: unsure] Teams fall back on three things: naming conventions, out-of-band catalogs (below), and occasionally fake config keys, which brokers reject. [known]

**Pattern subscription.** `consumer.subscribe(Pattern)` matches topics by regex. Matching is re-evaluated on each metadata refresh (`metadata.max.age.ms`), so new topics that match are picked up automatically. [known] https://kafka.apache.org/documentation/#consumerapi

**Naming conventions.** The common recommendation is hierarchical dotted names, e.g. `<domain>.<dataset>.<event>.<version>`, sometimes with an environment prefix. The aim is to make names greppable and regex-subscribable, and to support prefix ACLs. [known; Confluent blog guidance, not normative]

**The "is anyone reading" signal.** Consumer groups commit offsets to the `__consumer_offsets` compacted topic. `listConsumerGroups`, `listConsumerGroupOffsets` and `describeConsumerGroups` (members) give a committed offset per group per partition. Lag = log-end offset − committed offset. A topic with no group holding committed offsets, or whose groups have no members and stale commits, is the "dead topic" heuristic that tools such as Burrow and Cruise Control use. [known] Committed offsets themselves expire: `offsets.retention.minutes` defaults to 10080 (7 days), and since KIP-211 the clock starts only once the group is empty. So the signal dies a week after the last reader leaves. That is useful, because absence of a group means dead, but it is lossy. [known] https://cwiki.apache.org/confluence/display/KAFKA/KIP-211%3A+Revise+Expiration+Semantics+of+Consumer+Group+Offsets
- Readers that keep **their own cursor** (`assign()` plus external offset storage) are **invisible** to this signal. That case maps directly onto TermLink's client-side cursors. [known]

**Schema Registry** (Confluent, not Apache). A separate service stores schemas under subjects. By default the subject is `<topic>-value` (TopicNameStrategy). It is backed by its own compacted `_schemas` topic, independent of data retention. Schema IDs are embedded in each message's header bytes. [known] https://docs.confluent.io/platform/current/schema-registry/index.html

**Stream Catalog / Data Portal** (Confluent Cloud). This is a separate metadata store. Free tags (e.g. `PII`) and typed **business metadata** (e.g. `owner=david`, `owner_email`) attach to topics, schemas, fields and connectors through `/catalog/v1/entity`, with search over REST and GraphQL. Data Portal asks that each topic carry a description, tags, business metadata and an owner, so it can be discovered and access requests can be routed to the owner. The catalog survives data retention. [cited] https://docs.confluent.io/cloud/current/stream-governance/stream-catalog.html · https://docs.confluent.io/cloud/current/stream-governance/data-portal.html

**`auto.create.topics.enable`** defaults to `true` on Apache Kafka. A produce, or a metadata fetch, to an unknown name creates the topic with broker defaults. The pitfalls: a typo silently creates a new topic, defaults for partitions, replication and retention apply, nothing records who created it or why, and topics sprawl. Production guidance is to disable it and create topics explicitly, often through IaC or GitOps. Confluent Cloud does not allow it. [known] https://kafka.apache.org/documentation/#brokerconfigs_auto.create.topics.enable

## 2. Apache Pulsar

**Hierarchy.** Topics are named `persistent://tenant/namespace/topic`. The namespace is the policy unit: retention, TTL, backlog quota, inactive-topic policy and permissions all inherit from it. Topic-level policies override the namespace. Discovery is `pulsar-admin topics list <tenant/ns>`. All of this metadata lives in the metadata store (ZooKeeper or etcd), not in the data ledgers. [known] https://pulsar.apache.org/docs/next/concepts-multi-tenancy/

**Topic properties (PIP-110 "Support topic metadata").** These are free key/value pairs: `createNonPartitionedTopic(topic, Map properties)`, `getProperties` and `updateProperties`, which merges and keeps old keys. This is the closest native equivalent to description and owner. [cited] https://pulsar.apache.org/api/admin/3.1.x/org/apache/pulsar/client/admin/Topics.html · https://github.com/apache/pulsar/pull/17238 [unsure: whether properties are stored in the managed-ledger properties or in the metadata store; they survive message retention either way, but are **deleted with the topic**]

**Regex subscription.** `subscribe(topicsPattern)` works within a namespace and periodically auto-discovers new matching topics (`patternAutoDiscoveryPeriod`). [known]

**Built-in schema registry.** Schemas are stored per topic in BookKeeper and versioned, with a compatibility strategy set per namespace. Schema lifetime is tied to the topic, and topic/schema deletion consistency has been a bug class. [cited issue] https://github.com/apache/pulsar/issues/12795

**The "is anyone reading" signal.** Subscriptions are **server-side, durable cursors**. `topics stats` gives, per subscription, `msgBacklog`, `consumers[]`, `lastConsumedTimestamp`, `lastAckedTimestamp` and `lastConsumedFlowTimestamp`; topic-level stats give `publishers` and `msgRateIn`. Backlog plus last-ack time is a direct staleness signal per reader. [known] https://pulsar.apache.org/docs/next/administration-stats/

**Inactive-topic auto-deletion.** These broker settings are overridable per namespace and per topic (PR #7598): `brokerDeleteInactiveTopicsEnabled`, `brokerDeleteInactiveTopicsFrequencySeconds`, `brokerDeleteInactiveTopicsMaxInactiveDurationSeconds`, and `brokerDeleteInactiveTopicsMode`. The mode is either:
- `delete_when_no_subscriptions`: no subscriptions and no active producers;
- `delete_when_subscriptions_caught_up`: every subscription has zero backlog, there are no active producers or consumers, and nothing was published for longer than the max inactive duration.

[cited] https://github.com/apache/pulsar/pull/6077 · https://github.com/apache/pulsar/pull/7598 · https://streamnative.io/blog/apache-pulsar-2-6-0
Caveat: auto-deletion **also deletes the topic's permissions** (and its properties and schema), so in Pulsar metadata does *not* outlive the topic. [cited] https://support.streamnative.io/hc/en-us/articles/28502267550235

## 3. Catalog approaches

**AsyncAPI.** A spec document (YAML/JSON, version-controlled) describes `channels` (address, `description`, `messages` with payload schemas, `tags`, bindings for Kafka and others) and `operations` (send and receive, i.e. who produces and who consumes), plus `info.contact` for ownership. The metadata lives entirely outside the broker and is unaffected by retention or deletion. Drift between the document and reality is the cost. [known] https://www.asyncapi.com/docs/reference/specification/v3.0.0

**DataHub / OpenMetadata.** These crawl the broker (and the schema registry) on a schedule to create topic entities, then layer on catalog-owned aspects: `ownership` (owners with a type), `globalTags`, `glossaryTerms`, `editableProperties.description`, and domains. Descriptions and owners entered in the catalog persist even if the topic vanishes; stale entities are soft-deleted by "stateful ingestion". [known] https://datahubproject.io/docs/generated/ingestion/sources/kafka · https://docs.open-metadata.org/connectors/messaging/kafka

## Transferable takeaways for TermLink
- Topic metadata belongs in a **store separate from the message log**, as in every system above. Pulsar ties it to topic lifetime; Kafka plus a catalog lets it outlive the topic.
- "Is anyone reading" requires **server-visible cursors** (Kafka committed offsets, Pulsar subscriptions). Client-private cursors are invisible: the Kafka `assign()` blind spot.
- An inactive definition worth copying is Pulsar's `caught_up` mode: no producers, no consumers, zero backlog, and idle longer than N.
- Make topic creation explicit and record the creator (the lesson from Kafka's `auto.create` pitfalls).
