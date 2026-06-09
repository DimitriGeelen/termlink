# Substrate Primitive #9 — Broadcast With Replay (cv_index)

> See T-2018 arc-parallel-substrate ADR §6 #9. Inception: T-2089
> (`docs/reports/T-2089-broadcast-with-replay-inception.md`, Design A:
> tagged-post current-value via `metadata.cv_key`).
> Build: T-2103 / T-2104 / T-2105 / T-2106 / T-2107.

## What it solves

Late-joiners need **current state**, not the full event log. A new agent
arriving on `agent-presence` after the fleet has been running 24 hours
should see _who's currently advertising_, not replay 14,400 heartbeats
to compute the latest one per agent.

Substrate primitive #9 gives the hub a small in-memory map per topic:
`(cv_key) -> latest_offset`. Producers tag posts with `metadata.cv_key`;
the hub records the last offset to carry each key. Late-joiners read
that map either via a snapshot in `channel.subscribe` (envelopes inline)
or via `channel.cv_keys` (keys + offsets only).

**Discovery cost:**
- Before: walking a topic to find latest-per-key is O(N_envelopes).
- After: O(K) where K = distinct cv_keys (usually = number of producers).

For a 5-agent fleet running 24h with 30s heartbeats: **14,400 → 5**
(~3000× cheaper).

## Five-slice build

| Slice | Task | What it shipped |
|---|---|---|
| 1 | T-2103 | Hub-side `cv_index` module + `channel.post` hook (records on every cv-tagged post; last-write-wins; per-topic cap) |
| 2 | T-2104 | `channel.subscribe` accepts `include_current_value: bool`; response gains `current_values: [{cv_key, offset, msg}, ...]` |
| 3 | T-2105 | CLI `--include-current-value` flag + MCP parity on `termlink_channel_subscribe` |
| 4 | T-2106 | `channel.cv_keys` JSON-RPC + CLI `channel cv-keys <TOPIC>` + MCP `termlink_channel_cv_keys` (read-only inspection) |
| 5 | T-2107 | Heartbeat producer wiring — `listener-heartbeat.sh` emits `metadata.cv_key=$agent_id` by default |

## Operator recipes

### Discover currently-advertising agents on a hub

```
termlink channel cv-keys agent-presence
```

Output (per agent):

```
topic=agent-presence count=2
  smoke-claude-alpha -> @1
  smoke-claude-beta -> @2
```

JSON form:

```
termlink channel cv-keys agent-presence --json
```

Empty cv_index is **not an error** — `count: 0, entries: []` is a valid
state for a topic with no cv-tagged posts. Human mode prints
`no cv_keys recorded on topic "<name>"` (loud, not silent zero).

Unknown topic returns `CHANNEL_TOPIC_UNKNOWN` (-32013).

### Late-joiner reads current state inline with replay

```
termlink channel subscribe agent-presence --include-current-value
```

The snapshot prefixes the regular envelope stream:

```
[cv:smoke-claude-alpha@1] <sender> heartbeat: listener
[cv:smoke-claude-beta@2]  <sender> heartbeat: listener
[0] <sender> (010-termlink) heartbeat: listener
[1] <sender> (010-termlink) heartbeat: listener
... (regular stream continues)
```

JSON mode emits one `{"current_values": [...]}` JSON-line header before
the regular envelope JSON-lines.

The snapshot is **one-shot** — sent on the first hub call only;
subsequent paginated fetches don't re-request.

### Producer-side: opt out of cv tagging

Default is opt-IN: every `listener-heartbeat.sh` heartbeat carries
`cv_key=$agent_id`. For tests or migration scenarios where you want
the legacy untagged behavior:

```
bash scripts/listener-heartbeat.sh --agent-id legacy-agent --no-cv-key
```

The envelope ships without `metadata.cv_key`; the hub does not record
that agent into the cv_index; `channel cv-keys agent-presence` does NOT
list it.

## Semantics

### Last-write-wins

When two posts share the same `(topic, cv_key)`, the cv_index records
the **higher** offset. Stale entries are overwritten in-place.

Three posts with `cv_key=alice` at offsets 0, 5, 12:
`cv_index["topic"]["alice"] = 12`.

### Per-topic cap + cap-overflow

To prevent unbounded growth, each topic's cv_index caps at
`DEFAULT_CV_INDEX_CAP_PER_TOPIC = 1000` distinct cv_keys. Override
per-hub at start:

```
export TERMLINK_CV_INDEX_CAP_PER_TOPIC=5000
termlink hub start
```

**Cap-overflow behavior:** if a post arrives with a NEW cv_key after
the cap is reached, the post itself **succeeds** (atomic) but the
cv_index annotation **drops loudly** — the `cv_index::overflow_total`
counter increments. Subsequent posts with EXISTING keys still update
normally (no overflow on existing-key writes). This is the substrate
"loud refuse" invariant: never silently apply partial state.

Cap-overflow telemetry is exposed via internal counters:
- `cv_index::entries_active()` — current count
- `cv_index::overflow_total()` — cumulative refused-new-key count

### In-memory only

cv_index is **process-local** to the hub. A hub restart clears it.
Producers re-populate within one heartbeat cycle (~30s). Late-joiners
who land in the cold-start window see partial state and converge to
correct as more heartbeats arrive. This is acceptable for the presence
use case (heartbeat producers self-heal), but consumers MUST tolerate
the cold-start window (don't assume "cv_index empty" = "no agents").

Persistence across restarts is **out of scope** for v1 — captured as
follow-up if pain emerges.

### Sender semantics + identity binding

The cv_key is application-defined free-form metadata. The hub neither
authenticates it nor binds it to the sender's identity. A malicious
producer COULD post with `cv_key=victim-id` and override another agent's
slot. Mitigation strategies are application-side (e.g., use the verified
sender_id fingerprint as the cv_key on identity-sensitive topics).

For `agent-presence`, this is low-risk: a hostile producer with
authenticated hub access could already spoof heartbeats directly.

## Producers wiring `metadata.cv_key`

Current producers tagged:

| Producer | Topic | cv_key value |
|---|---|---|
| `listener-heartbeat.sh` | `agent-presence` | `$agent_id` (T-2107) |

Future candidates (out of scope this arc):
- Chat-arc reactions / pin state — cv_key per emoji or pin-target
- Dialog-presence typing indicators — cv_key per sender
- Any "current value per slot" broadcast pattern

## MCP surface

| Tool | Purpose |
|---|---|
| `termlink_channel_subscribe` with `include_current_value: true` | Late-joiner reads cv-indexed envelopes inline with the message stream |
| `termlink_channel_cv_keys` | Read-only inspection: `{topic} → {ok, topic, count, entries}` |

Both are MCP-callable from agent contexts; the latter is the cheap
"who's currently advertising?" answer.

## Failure modes + observability

| Symptom | Diagnosis |
|---|---|
| `channel cv-keys` returns `count: 0` after hub restart | Expected — cv_index is in-memory; wait for one heartbeat cycle. |
| `channel cv-keys` returns 0 entries despite heartbeats | Heartbeats running with `--no-cv-key`? Old `listener-heartbeat.sh` pre-T-2107? Check envelope: `termlink channel subscribe agent-presence --limit 1 --json` → look for `metadata.cv_key` field. |
| `cv-keys` count grows unboundedly | Producer mis-emitting cv_key (e.g. timestamp instead of stable id). cap at 1000 protects the hub; check overflow counter. |
| Subscribe `--include-current-value` returns empty `current_values` array on a populated topic | Topic has posts but none carry `metadata.cv_key`. Tag producers per T-2107 pattern. |
| `CHANNEL_TOPIC_UNKNOWN` (-32013) | Topic doesn't exist on this hub. cv_index is per-hub-local (substrate primitive #9 has no inter-hub federation by design — see G-060). |

## Related primitives

- **#1 CLAIM** (`channel.claim`, T-2019 / T-2042) — exclusive ownership
  of `(topic, offset)`. Independent from cv_index.
- **#2 DISPATCH** (`agent.find_idle`, T-2020 / T-2045) — derived idle
  roster: `LIVE(agent-presence) ∖ DISTINCT(claimed_by)`. **Consumes**
  the same `agent-presence` topic that #9 indexes — future optimization
  candidate (find_idle could read cv_index directly).
- **#5 RESILIENCE** (offline queue, T-2018 / T-2051) — durable FIFO
  for blip absorption. cv-tagged posts get the same queue+replay path;
  T-2049 dedupe ensures cv_index doesn't double-record on replay.
- **#10 BACKPRESSURE** (`hub.governor_status`, T-2048) — surfaces
  cv-related counters could be added in a future expansion (currently
  exposes only dedupe/cap/rate-limit counters).

## References

- ADR §6 #9 — `docs/architecture/parallel-execution-substrate.md`
- Inception — `docs/reports/T-2089-broadcast-with-replay-inception.md`
- Test coverage — `crates/termlink-hub/src/cv_index.rs` (8 unit tests),
  `crates/termlink-hub/src/channel.rs` subscribe-with-cv tests (6) +
  cv_keys handler tests (4)
