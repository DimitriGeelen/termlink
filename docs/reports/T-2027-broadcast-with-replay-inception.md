# T-2027 Inception Research — Substrate primitive #9: broadcast-with-replay (current-value-on-registration)

**Status:** GO with revised scope — ship the subscribe-side primitive only; defer the compaction-side to T-2028.
**Artifact created:** 2026-06-08
**Successor task on GO:** Build task — `channel.subscribe --from-latest` flag + matching CLI/MCP verbs.
**See also:** T-2018 ADR §6 #9; T-2028 (throughput/retention — the natural home for `retention: keep-latest`).

## 1. The §6 framing

ADR §6 primitive #9: *"For late-joiner room-state without replaying an entire log. Subscriber registers and receives the current value of a designated key, then live updates from cursor onward."*

The gap is real and concrete: when a new dashboard / late-joining agent / fresh consumer subscribes to a state-shaped topic (presence, focus pointer, room composition), they need the *current value* without paying O(topic_size) to replay every prior write.

## 2. Adjacent prior art in TermLink

**Existing `kv` primitive** (`crates/termlink-protocol/src/control.rs:77-81`):

| Verb       | Status                          | Fits T-2027? |
|------------|---------------------------------|--------------|
| `kv.set`   | Fire-and-forget per-session     | ✗ Per-session, not topic-shaped |
| `kv.get`   | Read-once per-session           | ✗ No watch / live updates |
| `kv.list`  | Enumerate per-session keys      | ✗ No watch |
| `kv.del`   | Delete per-session key          | n/a |

`kv` is **session-scoped metadata** — agent A's `kv` is invisible to agent B. There is no `kv.watch`. The kv store does not satisfy T-2027's use case.

**Existing `channel.subscribe`:**

```
termlink channel subscribe <topic> [--since-offset N | --limit L]
```

There is no `--from-latest --then-live` mode today. The closest workaround is two calls:
```
last = channel.subscribe <topic> --limit 1 --reverse
sub  = channel.subscribe <topic> --since-offset $last.offset
```

This has a race window: posts between the two calls can be missed or duplicated. Not a substrate-grade solution for state-shaped topics.

**Conclusion:** the substrate has the *data shape* (channel.post / .subscribe) but lacks the *operation* "give me the latest value atomically and subscribe to changes from that point".

## 3. Two halves of the primitive

Building T-2027 cleanly requires two pieces, and they belong to different inceptions:

| Half               | Concern                                                         | Owner |
|--------------------|-----------------------------------------------------------------|-------|
| Subscribe-side     | `--from-latest --then-live` semantics in `channel.subscribe`    | T-2027 (this one) |
| Compaction-side    | `retention: keep-latest` (or `keep-last-N`) topic-config option | T-2028 |

**Why the split:** the subscribe-side change is small and self-contained — one new flag, one read-latest-offset-then-stream code path, atomic by design (the hub holds the topic mutex during both halves). The compaction-side change touches storage, retention, and possibly the durability guarantees in ADR §3 — that's a bigger surface and belongs with T-2028's broader policy work on throughput/retention.

T-2027 can ship the subscribe-side independently; consumers writing to a "current-value-only" topic can simply post a new envelope each time (the latest one will be what `--from-latest` returns). Compaction is an optimization, not a correctness requirement. Without compaction, the topic grows; if that becomes a problem in practice, T-2028 addresses it.

## 4. Subscribe-side spec (the part this inception authorizes)

New flag on `channel.subscribe`:

```
termlink channel subscribe <topic> --from-latest [--then-live | --once]
```

- `--from-latest` — read the most recent envelope on the topic, return it as the first event.
- `--then-live` — continue streaming subsequent envelopes as they arrive (the default for `--from-latest`).
- `--once` — return just the latest envelope and exit. Equivalent to a "give me the current value" verb.

Semantics:
- Atomic: hub holds the topic's read mutex while resolving "latest" and seeking the cursor, so no posts can land between the two operations.
- Empty-topic case: `--once` returns `{ok: true, envelope: null}`; `--then-live` waits for the first post.
- Concurrent posts during fetch: behave as if the new post arrived just after the latest-snapshot was returned.

Bus library function:
```rust
pub async fn subscribe_from_latest(
    &self,
    topic: &str,
    mode: FromLatestMode,  // Once | ThenLive
) -> Result<(Option<Envelope>, Option<SubscribeStream>)>
```

MCP tool: `termlink_channel_subscribe_from_latest` — params `topic`, `mode` (`"once"` | `"then_live"`).

## 5. IW dispositions

- **IW-1 (current-value as topic-level config or per-subscriber on registration):** PER-SUBSCRIBER on registration. A subscribe flag is more flexible than a topic config — same topic can be read in either mode by different consumers (dashboards want `--from-latest`, audit tools want `--since-offset 0`). Confidence=4.
- **IW-2 (how is "current" defined — last-written, or separate snapshot):** LAST-WRITTEN. Adding a separate "publisher writes a snapshot" mechanism doubles the API surface and the publisher's coordination cost. Last-written is what the consumer means by "current". If a future use case needs structured snapshots, that's a kv-style overlay on top, not a substrate primitive. Confidence=4.
- **IW-3 (storage — keep current value in SQLite alongside log, or separate kv store):** NEITHER — no extra storage. The latest envelope IS what's already at the topic's max offset. `--from-latest` is a read pattern over existing data. Compaction (T-2028) can optionally evict older envelopes to bound storage growth, but that's an optimization, not a primitive. Confidence=4.
- **IW-4 (cursor interaction — new subscriber starts at current-value + live, or at cursor=0):** AT CURRENT-VALUE + LIVE. That's the explicit point of the primitive. Old behavior (`--since-offset 0`) remains available as the explicit replay-everything mode. Confidence=4.

## 6. Cost / risk

- **New code:** ~80 LOC. One new flag on the subscribe API, one new bus library function, one new CLI option, one new MCP tool.
- **Slices:** 4 vertical slices (bus library, hub handler / parser, CLI flag wiring, MCP tool + docs).
- **Schema migration:** none — uses existing channel storage and existing subscribe streaming.
- **Risk surface:** low. The atomicity proof is short (single read-mutex hold), and the empty-topic case is named.
- **Conflicts with prior primitives:** none. Pure additive surface.

## 7. Recommendation

**GO with revised scope.** Ship the subscribe-side primitive (`channel.subscribe --from-latest [--once | --then-live]`) and defer the compaction-side (`retention: keep-latest`) to T-2028.

**Build slice plan:**
- **Slice 1:** `Bus::subscribe_from_latest` library function + unit tests (happy path, empty topic, concurrent post race).
- **Slice 2:** Hub handler — extend `channel.subscribe` parameter parsing to accept `--from-latest` mode; route through the new bus function.
- **Slice 3:** CLI flag `--from-latest [--once|--then-live]` on `termlink channel subscribe`; session-client wrapper.
- **Slice 4:** MCP tool `termlink_channel_subscribe_from_latest` + help-registry entry + docs in `docs/operations/substrate-claim-primitive.md` showing the late-joiner-dashboard recipe.

## 8. GO criteria evaluation

- ✅ "Either consolidation with T-2025 is decided OR a small bounded spec is locked." — T-2025 turned NO-GO (presence DATA already durable). T-2027 stays as its own primitive, with bounded spec.
- ❌ "Use case is fully covered by T-2025 — close as duplicate." — Not the case. T-2025's NO-GO covered presence durability; T-2027 covers the late-joiner read pattern, which the existing `channel.subscribe` does not provide atomically.

## 9. ADR alignment check

| ADR section | Alignment |
|-------------|-----------|
| §2 "append-log is the primary surface" | ✓ Uses the same append log; adds a read pattern over existing data. |
| §3 "channel logs are durable" | ✓ Untouched. Compaction (the part that *could* affect durability) is deferred to T-2028. |
| §5 "one writer, serialized" | ✓ No new writers introduced. Subscribe-side change only. |
| §6 #9 framing | ✓ Resolved: subscribe verb gains the late-joiner mode the framing asks for. |

## 10. Open follow-up tasks to file on GO

- Build task: Slices 1-4 (`channel.subscribe --from-latest`).
- *(Pre-existing)* T-2028 to design `retention: keep-latest` topic-config — independent track; T-2027 ships without depending on it.
