# Substrate primitive #5 Gap A — client_msg_id idempotency

T-2049 closes T-2023 Gap A (IW-4 — idempotency): the double-apply
scenario where a spoke posts, the hub commits at offset N, the TCP ack
is lost, the spoke retries from its offline queue, and the hub commits
AGAIN at offset N+1. Subscribers see the same payload twice. With this
primitive, the hub recognises the retry and silently returns the
cached `{offset, ts}` of the original commit — exactly-once across hub
blips.

Read with `docs/reports/T-2023-client-reconnect-queue-inception.md`
§4.A for the inception framing.

## The wire shape

`channel.post` accepts a new optional field, `client_msg_id`:

| Field | Type | Required | Meaning |
|---|---|---|---|
| `client_msg_id` | string, 1..=128 chars | NO | Opaque idempotency token. The hub uses `(sender_id, client_msg_id)` as a dedupe key. |

Old clients omit the field and behave exactly as before — dedupe is
opt-in. New clients (CLI default, post-T-2049) mint a fresh random
128-bit hex id per call and persist it with the offline-queue row, so a
flush-replay reuses the same id.

A duplicate post (same `sender_id` + same `client_msg_id` within the
TTL) returns the original success envelope with one extra field:

```json
{"offset": 42, "ts": 1780934741917, "deduped": true}
```

Clients can ignore `deduped`; it's purely diagnostic. The `offset` is
the ORIGINAL post's offset — re-using it for `--reply-to` or cursor
ops is safe.

## TTL and capacity

| Knob | Default | Override |
|---|---|---|
| `TERMLINK_DEDUPE_TTL_MS` | 300_000 (5 min) | Set at hub start |
| `TERMLINK_DEDUPE_CAPACITY` | 10_000 entries | Set at hub start |

Five minutes is comfortably longer than realistic spoke reconnect
windows (TCP backoff, hub bounce, network blip) — a retry that takes
longer than this is no longer a "lost ack on the same call", it's a
fresh post and double-apply isn't possible anyway.

Ten thousand entries is the floor under pathological burst-of-distinct
load. TTL keeps the cache small under normal traffic; LRU eviction
fires only when both the TTL hasn't caught up AND distinct-id volume
exceeds the cap. Memory bound is ~1 MB.

Setting `TERMLINK_DEDUPE_TTL_MS=0` clamps to 1 ms (dedupe effectively
disabled but the code path still runs). Setting `TERMLINK_DEDUPE_CAPACITY=0`
clamps to 1.

## Operator probe — `hub.governor_status`

The existing T-2048 `hub.governor_status` RPC (Tier-A, scope=Observe,
no auth) gains three sibling fields:

```json
{
  ...
  "dedupe_entries_active": 12,
  "dedupe_hits_total": 3,
  "dedupe_ttl_ms": 300000
}
```

- `dedupe_entries_active`: current cache occupancy. Watch this trend
  against `dedupe_capacity` (hard-coded — bump via env if growing).
- `dedupe_hits_total`: monotonic counter. **A non-zero value is the
  smoking gun for "hub blip caused a spoke retry — and we caught it
  before subscribers saw the double-apply"**. Pair with the
  observed-blip from `fleet doctor` or rotation logs to confirm
  causality.
- `dedupe_ttl_ms`: configured TTL. Static; matches the env var at
  hub start.

### Probe recipes

```sh
# CLI (Unix-socket, no auth setup)
termlink remote call local hub.governor_status

# MCP parity
termlink_hub_governor_status

# Python one-liner (works against any local hub.sock)
python3 -c "
import socket, json
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect('/var/lib/termlink/hub.sock')
req = {'jsonrpc':'2.0','id':1,'method':'hub.governor_status','params':{}}
s.sendall((json.dumps(req)+'\n').encode())
print(s.recv(4096).decode())"
```

## Spoke-side opt-in

The CLI's `termlink channel post` mints a fresh 128-bit hex id per
call by default. To pass an explicit one (e.g. content-hash for
script-level dedupe across retries):

```sh
termlink channel post smoke:topic \
  --payload "hello" \
  --client-msg-id "$(echo -n "hello" | sha256sum | cut -c1-32)"
```

Or to opt out entirely (recover pre-T-2049 wire shape):

```sh
# No explicit way to opt out — minting is unconditional in the CLI.
# Direct RPC callers omit the field; the hub then short-circuits the
# dedupe check.
```

The offline queue (`crates/termlink-session/src/offline_queue.rs`)
persists `client_msg_id` with each row, so the flush loop replays the
SAME id on every retry — that's the load-bearing piece. Without it,
each replay would mint a fresh id and the hub couldn't dedupe.

## The queue-replay scenario, end-to-end

1. Spoke calls `termlink channel post topic X --payload P`. CLI mints
   `client_msg_id=AAA` and tries direct delivery.
2. Hub receives, signature verifies, dedupe miss, `bus.post` commits
   at offset N, response `{offset: N, ts}` is written to the socket,
   dedupe records `(sender, AAA) -> {offset: N, ts}`.
3. TCP RST mid-flight — spoke never sees the response. From its point
   of view, the call failed.
4. Spoke retries (immediately on TCP error, or after backoff from
   the offline queue's flush loop). The PendingPost row carries the
   SAME `AAA`.
5. Hub receives the retry, signature verifies, dedupe HIT, returns
   `{offset: N, ts, deduped: true}` without calling `bus.post`.
6. Spoke sees success, removes the row from its queue. Substrate is
   exactly-once.

If step 2-3 happened before the dedupe could record (e.g. the hub
crashed between `bus.post` and the dedupe insert), step 5 would
re-append at offset N+1. This is the residual gap — a follow-up
could pre-reserve the dedupe entry before `bus.post`, but the race
window is microseconds wide and only fires under hub crash, which is
itself an outage event. Not in T-2049's scope.

## What this does NOT do

- **Cross-hub dedupe.** Each hub maintains its own LRU. A federated
  post that traverses two hubs would dedupe on each independently —
  if the federation layer retries through a different hub, both could
  apply. This is a federation-layer concern, not a primitive concern.
- **Cross-restart dedupe.** The LRU is in-memory. A hub restart
  loses the cache, and a retry that arrives after restart will
  re-apply. The TTL (5 min) bounds the exposure window — a spoke
  whose retry takes longer than 5 min has already given up by spec.
- **Content-hash dedupe.** The id is opaque; if a caller passes a
  content hash, content-hash idempotency falls out for free. If
  callers pass random ids (the CLI default), legitimate intentional
  re-posts of identical payloads each get distinct ids and proceed
  normally — which is the safe default.
- **Authentication.** Dedupe runs AFTER identity verification (T-1427
  invariant). An attacker can't poison another sender's dedupe
  namespace.

## Related

- T-2023 inception report — the framing this resolves
- T-2018 ADR §6 #5 — the substrate primitive
- T-2048 — the governor LRU that this sits alongside (shares the
  `hub.governor_status` RPC)
- T-1439 — the original offline-queue implementation (the durable
  FIFO that calls this with the same id on every replay)
- T-1427 — the signed-sender identity invariant that namespaces the
  dedupe key safely
- `docs/operations/substrate-offline-queue-recipe.md` — operator-side
  workflow for inspecting the queue, manual drain, poison-pill triage
  (T-2051)
