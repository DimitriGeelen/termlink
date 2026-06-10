# Substrate primitive #5 — offline-queue operator recipe

T-2051 documents the operator-side workflow for the offline-queue
substrate primitive (T-1439, T-2018 §6 #5). The queue itself is
load-bearing: a spoke that loses the hub mid-post does NOT silently
drop the envelope — the call enqueues to a local SQLite store and a
background flush task drains when the hub returns.

Read with:

- `docs/operations/substrate-post-idempotency.md` — T-2049 dedupe that
  makes replay safe.
- `docs/reports/T-2050-offline-queue-backoff-audit.md` — backoff
  parameter audit + the gap that T-2055 will close.

## What this primitive guarantees

| Scenario | What happens |
|---|---|
| Spoke posts, hub up, network healthy | Direct delivery. Queue untouched. |
| Spoke posts, hub down or unreachable | `channel.post` enqueues to SQLite, returns `PostOutcome::Queued{queue_id}`. CLI prints "Queued to T — queue_id=N (hub unreachable; will flush on next reconnect)". |
| Hub returns | Flush task wakes every 5s, drains in FIFO order, deletes rows after success. T-2049 dedupes any replay duplicate. |
| Queue is full (`TERMLINK_OUTBOUND_CAP`, default 1000) | LOUD fail per R3 — `channel.post` returns `QueueError::QueueFull{cap}`. The CLI exits non-zero with a clear message; no silent drop. |
| Hub rejects a queued row (e.g. unknown topic, malformed) | Row's attempts counter bumps. After `POISON_THRESHOLD=10` rejects, the row is dropped, `tracing::warn!` fires, and the flush continues. |

## Where things live on disk

```
~/.termlink/                     # default identity dir
├── outbound.sqlite              # the queue (single SQLite file)
├── outbound.sqlite-shm          # SQLite WAL companion (transient)
└── outbound.sqlite-wal          # SQLite WAL companion (transient)
```

Override per-identity via `TERMLINK_IDENTITY_DIR=/path/to/dir`. Per-test
isolation is the primary use case; production hosts default to `$HOME`.

The schema is intentionally narrow:

```sql
CREATE TABLE pending_posts (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    post_json     TEXT    NOT NULL,   -- full PendingPost serialized
    enqueued_ms   INTEGER NOT NULL,
    attempts      INTEGER NOT NULL DEFAULT 0
);
```

`post_json` carries the signed envelope, T-1287 metadata routing hints,
AND (post-T-2049) the `client_msg_id` — so a flush-replay reuses the
SAME id and the hub deduplicates the retry.

## End-to-end recipe

### Inspect pending rows

```sh
# Quick count
sqlite3 ~/.termlink/outbound.sqlite \
  'SELECT COUNT(*), MIN(enqueued_ms), MAX(attempts) FROM pending_posts;'

# Show oldest entry's topic + msg_type without dumping payload
sqlite3 ~/.termlink/outbound.sqlite \
  "SELECT id, enqueued_ms, attempts,
          json_extract(post_json, '\$.topic') AS topic,
          json_extract(post_json, '\$.msg_type') AS msg_type,
          json_extract(post_json, '\$.client_msg_id') AS client_msg_id
   FROM pending_posts ORDER BY id LIMIT 5;"
```

### Manually trigger drain

The CLI runs a one-shot drain on every `channel post` invocation
before sending its own. Idle hosts have no implicit drain — to force
one, post a no-op:

```sh
# This will drain pending rows first, then post; the post itself is
# safe to ignore (the dedup will absorb any operator-issued sentinel).
termlink channel post smoke:drain --payload "manual drain" \
  --client-msg-id "drain-$(date -u +%s)"
```

### Confirm dedupe absorbed any replays

```sh
# Counter rises by exactly the number of retried-but-already-applied rows.
termlink hub status --governor --json \
  | jq '.result | {dedupe_hits_total, dedupe_entries_active}'
```

### After a hub bounce, watch the flush

```sh
# Total queued
sqlite3 ~/.termlink/outbound.sqlite 'SELECT COUNT(*) FROM pending_posts;'

# 5 seconds later (one flush tick)
sleep 5
sqlite3 ~/.termlink/outbound.sqlite 'SELECT COUNT(*) FROM pending_posts;'
```

If the count is not falling, see the failure-modes section below.

## Observability arc (T-2083..T-2087)

The queue itself just absorbs blips. The observability arc is how an
operator (or an orchestrating agent) actually *notices* a blip, gets
paged about it, and answers "how often has this happened?" after the
fact — without keeping a shell window pinned to `sqlite3 ~/.termlink/outbound.sqlite`.

| Surface | Verb | Use case |
|---|---|---|
| Watch | `channel queue-status --watch <secs>` | Live monitor: clears screen + re-renders depth every N secs, prints one-line event on `drained↔pending` flips |
| Notify | `... --watch <secs> --notify <CMD>` | Page/Slack: operator-pluggable shell command fires fire-and-forget per state-flip event |
| Log | `... --watch <secs> --log <PATH>` | Forensic trail: append-only NDJSON audit. Schema `{ts, kind, old_pending, new_pending, oldest_age_ms, queue_path}` |
| History (CLI) | `channel queue-history [--since DAYS] [--kind …]` | Retrospective: walk the log offline, answer "has the queue been backing up?" |
| History (MCP) | `termlink_channel_queue_history` | Agent-callable parity — same envelope, same filters, no shell-out |

Queue state is binary (no `transition` kind — see T-2083 design note),
so events fire on the `0↔N+` flip only. Depth changes within
still-pending are not events.

The arc is symmetric with substrate primitive #2 DISPATCH (find-idle
T-2078..T-2082): same shape, same operator UX, just pointed at
queue.log instead of find-idle.log.

### Live monitor (watch)

```
termlink channel queue-status --watch 5
```

Clears the screen and re-renders every 5 seconds:

```
queue=/root/.termlink/outbound.sqlite  pending=0  oldest_age=-  (watch interval=5s)
  ▶ pending  pending=0→3  oldest_age=120ms
```

The `▶` line appears only on the tick where the queue flipped from
drained to pending (or vice versa). SIGINT (Ctrl-C) exits cleanly.

### Page-on-blip (watch + notify)

```sh
# /usr/local/bin/page-on-queue-pending.sh
#!/bin/sh
[ "$TERMLINK_QUEUE_CHANGE_KIND" = "pending" ] || exit 0
curl -sX POST "$SLACK_WEBHOOK" \
  -d "{\"text\":\":boom: queue ${TERMLINK_QUEUE_PATH} backed up to ${TERMLINK_QUEUE_NEW_PENDING} pending (age ${TERMLINK_QUEUE_OLDEST_AGE_MS})\"}"

# Then run the watch with --notify:
termlink channel queue-status --watch 5 \
  --notify /usr/local/bin/page-on-queue-pending.sh
```

Per-event env vars (always set, schema is exactly 6 vars):

| Var | Value |
|---|---|
| `TERMLINK_QUEUE_CHANGE_KIND` | `drained` or `pending` |
| `TERMLINK_QUEUE_TS` | RFC3339 detection time |
| `TERMLINK_QUEUE_OLD_PENDING` | numeric, prior depth |
| `TERMLINK_QUEUE_NEW_PENDING` | numeric, current depth |
| `TERMLINK_QUEUE_OLDEST_AGE_MS` | numeric or `n/a` when drained |
| `TERMLINK_QUEUE_PATH` | absolute path to the sqlite file |

Hanging scripts do NOT block the loop; command-not-found does NOT
kill the watch. Baseline tick fires no events.

### Forensic trail (watch + log)

```
termlink channel queue-status --watch 5 --log ~/.termlink/queue.log
```

Appends one NDJSON line per state-flip event to the log file. Parent
directory auto-created; disk-full / permission errors print one-line
stderr warning and the watch continues (never crashes on a log write).

NDJSON schema (exactly 6 fields — schema stability locked by unit
test):

```json
{"ts":"2026-06-09T13:00:00Z","kind":"pending","old_pending":0,"new_pending":3,"oldest_age_ms":500,"queue_path":"/root/.termlink/outbound.sqlite"}
{"ts":"2026-06-09T13:00:10Z","kind":"drained","old_pending":3,"new_pending":0,"oldest_age_ms":null,"queue_path":"/root/.termlink/outbound.sqlite"}
```

`oldest_age_ms` serializes as JSON `null` on `drained` events
(NOT the string `"n/a"` — that convention is `--notify` env-var only).

Ad-hoc grep for backpressure incidents:

```sh
jq -c 'select(.kind=="pending")' ~/.termlink/queue.log
```

`--notify` and `--log` are symmetric — set both flags and each event
lands in both surfaces from the same per-tick event source.

### Retrospective (queue-history)

After the watch has been running for a while (or you suspect prior
blips), answer "has this host been losing connectivity?" without
keeping the watch terminal attached:

```
termlink channel queue-history --since 7 --kind pending
```

Renders one human-format line per matching entry plus a per-kind
aggregate footer (`pending=N  drained=M`). Default 7-day window,
clamped 1..=365. Pure read; no auth; no network.

JSON envelope shape (for scripting):

```
termlink channel queue-history --since 7 --json
```

Returns `{ok, entries[], summary{total, pending_events, drained_events, since_days, kind_filter, malformed_lines_skipped, log_path}}`.

Agent-callable MCP parity:

```
termlink_channel_queue_history(since_days=7, kind="pending")
```

Same params, same envelope shape — exactly what
`channel queue-history --since 7 --kind pending --json` returns,
modulo the optional `hint` field when the log file doesn't exist yet.
Pure read; no auth; no network; no log mutation.

## Failure modes & how to spot them

### Hub down — queue accumulates

**Symptom:** SELECT COUNT(*) climbs on every CLI post; nothing drains.

**Cause:** Hub unreachable. Tracing emits `flush: transport error, will retry`.

**Action:** Bring the hub up. The next 5s tick drains. Verify with the
end-to-end recipe above.

### Queue full — loud fail

**Symptom:** `termlink channel post` exits non-zero with
`outbound queue full (1000 entries; refusing new posts — R3 loud-fail)`.

**Cause:** Hub has been down long enough that 1000 envelopes
accumulated. R3 says "refuse" — silent drop is forbidden.

**Action:** Either bring the hub up (queue drains, new posts accepted),
or pre-raise the cap before the next outage by setting
`TERMLINK_OUTBOUND_CAP=10000` in the spoke's environment.

### Poison-pill row — head-of-queue dropped

**Symptom:** Flush log shows
`flush: dropping poison post after 10 hub-reject attempts`. The
`dropped_poison` counter in `FlushReport` (returned by `flush()`)
increments.

**Cause:** A queued row is rejected by the hub on every attempt
(unknown topic, malformed payload, expired signature). The
implementation distinguishes hub-reject from transport-fail: a
transport error breaks the drain and retries; a hub-reject bumps
`attempts` per the audit in `docs/reports/T-2050-offline-queue-backoff-audit.md`.

**Action:** Find the offending row BEFORE it's dropped by looking for
the highest `attempts` value:

```sh
sqlite3 ~/.termlink/outbound.sqlite \
  "SELECT id, attempts,
          json_extract(post_json, '\$.topic') AS topic,
          json_extract(post_json, '\$.msg_type') AS msg_type
   FROM pending_posts ORDER BY attempts DESC LIMIT 5;"
```

If a row's topic is genuinely wrong (typo, retired primitive), accept
the drop. If the topic should exist but doesn't yet, `termlink channel
create <topic>` first, then trigger a manual drain.

### Interplay with T-2049 idempotency

A queued row carries its `client_msg_id` per T-2049. When the flush
sends it, the hub checks `(sender_id, client_msg_id)` against the
dedupe cache:

- If the hub never saw this id → process the post normally.
- If the hub already committed it (a retry after lost-ack) → return
  the cached `{offset, ts, deduped: true}` without re-appending. The
  spoke deletes the row from its queue; substrate stays exactly-once.

This means **a queue replay after a hub blip is safe by construction**
(within the 5-min dedupe TTL). Operators who manually inspect rows
and trigger drains do not need to worry about double-application.

## Tunables

| Env var | Default | Effect |
|---|---|---|
| `TERMLINK_OUTBOUND_CAP` | 1000 | Refuse new enqueues past this size. R3 loud-fail. |
| `TERMLINK_IDENTITY_DIR` | `$HOME/.termlink` | Override where the queue + identity files live. Test isolation. |
| `TERMLINK_DEDUPE_TTL_MS` (hub side) | 300_000 (5min) | Time window the hub keeps a dedupe entry. Bumping past spoke-side flush-loop intervals is wasted memory. |

Flush cadence is currently `DEFAULT_FLUSH_INTERVAL = 5s` in
`crates/termlink-session/src/bus_client.rs`. Not env-configurable
by default (tests use `connect_with_interval` directly). T-2055 will
add ±25% jitter; see the T-2050 audit for rationale.

## Telemetry

| Signal | Where | What it means |
|---|---|---|
| `flush: dropping poison post after 10 hub-reject attempts` | tracing::warn! | A row was abandoned. Search log aggregation for this string. |
| `dedupe_hits_total` | `hub.governor_status` JSON-RPC | How many spoke retries the hub absorbed. Rising = real outages. |
| `dedupe_entries_active` | same | Current cache occupancy. Capacity ceiling is `TERMLINK_DEDUPE_CAPACITY` (default 10_000). |
| `outbound.sqlite` row count | sqlite3 query | Live queue depth on the spoke. |
| `pending`/`drained` events | `channel queue-status --watch --log ~/.termlink/queue.log` | NDJSON audit trail of state-flips. `jq` retrospective answers "how often has this happened?" — see Observability arc above. |
| `pending_events` / `drained_events` totals | `channel queue-history --json` / `termlink_channel_queue_history` MCP | Aggregated counts over a time window; agent-callable. |

## What this does NOT do

- **Visibility across hosts.** The queue is local to each spoke. Two
  spokes on the same host running as the same identity share it (via
  `$HOME/.termlink/outbound.sqlite`); spokes on different hosts have
  independent queues.
- **Cross-restart flush guarantee.** The queue survives a spoke
  restart (SQLite is durable). The 5s flush tick resumes on the next
  `BusClient::connect`. If the spoke is killed mid-flush, the
  in-flight row remains in the queue and is retried next tick.
- **Synchronous replay confirmation.** `channel.post` returns
  `PostOutcome::Queued` and exits — there's no follow-up "the queue
  is now empty" notification. Operators check `outbound.sqlite` row
  count if they need that confirmation.

## Related

- **Master integration recipe (T-2124):** [`substrate-orchestrator-recipe.md`](substrate-orchestrator-recipe.md) — shows where the offline queue fits in the AEF-layer's end-to-end work-stealing pattern (worker `channel.post` during hub blip → queued → flushed → safe via dedupe on replay).
- T-2018 ADR §6 #5 — substrate primitive
- T-1439 — the original offline-queue implementation
- T-2049 — `client_msg_id` idempotency that makes replays safe
- T-2050 — backoff audit (this primitive's flush parameters)
- T-2055 — jitter wire-in (T-2050 audit follow-up)
- T-2083 — observability arc Slice 1: `queue-status --watch` live monitor
- T-2084 — observability arc Slice 2: `--notify` event hook (page-on-blip)
- T-2085 — observability arc Slice 3: `--log` NDJSON audit trail
- T-2086 — observability arc Slice 4: `queue-history` retrospective CLI verb
- T-2087 — observability arc Slice 5: `termlink_channel_queue_history` MCP parity (arc closure)
- `docs/operations/substrate-post-idempotency.md` — dedupe details
- `docs/reports/T-2023-client-reconnect-queue-inception.md` — inception
- Sibling arcs with the same shape: substrate primitive #2 DISPATCH (find-idle, T-2078..T-2082), substrate primitive #1 CLAIM (claims-summary, T-2042..T-2077), substrate primitive #10 BACKPRESSURE (governor, T-2048..T-2071)
