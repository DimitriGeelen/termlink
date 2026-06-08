# Substrate primitive #10 (Track B) — connection cap + per-sender rate limit

T-2048 ships the hub-side LOUD-refuse governors that resolve T-2028 §6
primitive #10 Track B (the IW-3 gap — "No connection cap, rate limiter,
or backpressure governor exists. T-1991 (agent-presence bloat) was found
in PRODUCTION, not predicted").

Read this with the inception report at
[`docs/reports/T-2028-throughput-retention-inception.md`](../reports/T-2028-throughput-retention-inception.md)
for the design rationale (PARTIAL-GO, three sub-tracks, why per-sender).

## Two governors, one shape

| Governor | Scope | Default | Refusal code | Refuses ... |
|---|---|---|---|---|
| `ConnGovernor` | per-process | `TERMLINK_MAX_CONNECTIONS=256` | `HUB_AT_CAPACITY` (-32019) | new connections beyond `max` |
| `RateGovernor` | per-sender | `TERMLINK_RATE_LIMIT_PER_SEC=1000` | `RATE_LIMITED` (-32008) | RPCs that exceed the sender's bucket |

Both emit one structured envelope on refuse — never a silent drop
(IW-3 — LOUD). Envelope carries `data.retry_after_ms` so the caller can
back off precisely.

## What gets refused

### Connection cap (`HUB_AT_CAPACITY`)

Triggered in `run_accept_loop` BEFORE the connection is handed to a
spawned `handle_connection` task. Two paths:

1. **Unix socket:** `write_capacity_refusal` writes one
   `{"jsonrpc":"2.0","id":null,"error":{"code":-32019,"message":"Hub at
   capacity (retry in 1000ms)","data":{"retry_after_ms":1000}}}` line
   then `shutdown()`. The next time the caller tries to read a frame
   they get the envelope.

2. **TCP:** the raw socket is closed BEFORE TLS handshake (faster path
   off the wire; would require a fake TLS cert otherwise). The CLI
   surfaces this as a handshake-fail; server-side it surfaces via
   `capacity_hits_total` in `hub.governor_status`.

`retry_after_ms` is always 1000 — the accept loop cannot predict when a
slot frees, so it gives a fixed conservative hint.

### Per-sender rate limit (`RATE_LIMITED`)

Triggered in `handle_connection` AFTER request parse but BEFORE
`router::route` dispatch. Sender-key priority:

1. `params.from` (operator/agent self-identifier — same key the audit
   log uses)
2. `peer_addr` (network identity for TCP callers)
3. `peer_pid` (Unix-local PID as string)
4. `"anonymous"` (degenerate fallback)

Each sender gets a token bucket with capacity = refill rate =
`rate_limit_per_sec`. On overflow:

```json
{
  "error": {
    "code": -32008,
    "message": "Rate limit exceeded for sender 'X' (retry in 1ms)",
    "data": {
      "retry_after_ms": 1,
      "sender": "X"
    }
  }
}
```

`retry_after_ms` is computed exactly from the deficit and refill rate
(`ceil(deficit / (rate_per_sec / 1000))`).

## Observability — `hub.governor_status`

JSON-RPC method, scope=Observe (no auth required for Unix-socket
callers). Returns:

```json
{
  "connections_active": 12,
  "connections_max": 256,
  "capacity_hits_total": 0,
  "rate_buckets_active": 5,
  "rate_hits_total": 0,
  "max_rate_per_sec": 1000
}
```

Field semantics:

- `connections_active`: current count of in-flight connections (read
  from the global `ConnGovernor`).
- `connections_max`: cap value at hub-start; matches
  `TERMLINK_MAX_CONNECTIONS` env var or `DEFAULT_MAX_CONNECTIONS`
  (256). Bumping requires a hub restart.
- `capacity_hits_total`: monotonic counter; every time
  `ConnGovernor::try_acquire` refused, this increments. A non-zero
  value is the "T-1991-style 'found in production not predicted'
  failure mode" signal — operators alert on it.
- `rate_buckets_active`: number of distinct senders currently tracked
  in the rate map. Bounded by sender diversity; idle buckets evict via
  `RateGovernor::evict_idle` (wire-up reserved for a follow-up).
- `rate_hits_total`: monotonic counter; every time
  `RateGovernor::try_acquire` refused, this increments. Pair with
  `capacity_hits_total` for "is the substrate under stress?".
- `max_rate_per_sec`: refill rate at hub-start; matches
  `TERMLINK_RATE_LIMIT_PER_SEC` or `DEFAULT_RATE_LIMIT_PER_SEC`
  (1000).

T-2049 added three post-idempotency fields (present on any hub running
≥ T-2049; older hubs omit them — CLI/MCP renderers fall back to `n/a`):

- `dedupe_entries_active`: number of `(sender_id, client_msg_id)` keys
  currently live in the post-dedupe LRU. Bounded by
  `TERMLINK_DEDUPE_CAPACITY` (default 10_000). Steady-state under
  normal traffic; a steady climb here means the LRU window is too
  small for the post-rate.
- `dedupe_hits_total`: monotonic counter; every time a `channel.post`
  matched a prior `(sender_id, client_msg_id)` pair and was returned
  the cached envelope (exactly-once retry absorbed), this increments.
  Non-zero is the "spoke-side retry budget actually fired" signal —
  expected during hub blips, suspicious during steady state.
- `dedupe_ttl_ms`: LRU eviction TTL at hub-start; matches
  `TERMLINK_DEDUPE_TTL_MS` (default 300_000 = 5 min). A spoke retry
  beyond this window will double-apply — operators tuning for higher
  blip tolerance bump it here.

See `docs/operations/substrate-post-idempotency.md` for the full
exactly-once delivery model these fields surface.

### Recipe — operator probe

The hub.governor_status envelope is reachable through four parallel
routes — pick the one that fits the caller:

```sh
# 1. Raw RPC (local socket; no auth setup)
termlink remote call local hub.governor_status

# 2. Single-hub CLI inline with lifecycle (T-2060, Track C)
termlink hub status --governor

# 3. Fleet-wide aggregation across every hub in ~/.termlink/hubs.toml
#    (T-2062, Track D) — answers "which hub is wedged / hitting cap?"
termlink fleet governor-status            # human-readable
termlink fleet governor-status --json     # for scripts / dashboards
termlink fleet governor-status --timeout 5  # tighter per-hub bound

# 4. MCP parity for agents — single-hub + fleet
termlink_hub_governor_status              # one local hub (T-2048)
termlink_fleet_governor_status            # walks every profile (T-2063)

# 5. Continuous-monitor surveillance — leave it running in a terminal (T-2064)
termlink fleet governor-status --watch 30          # baseline + change-only emission
termlink fleet governor-status --watch 30 --timeout 5
# Pair with --notify for operator-pluggable response on change events (T-2065):
termlink fleet governor-status --watch 30 --notify /usr/local/bin/page-on-cap.sh

# 6. Bare-bones inspection one-liner (Unix socket only, no termlink CLI needed)
echo '{"jsonrpc":"2.0","id":1,"method":"hub.governor_status","params":{}}' \
  | socat - UNIX-CONNECT:$(termlink hub status --json | jq -r .socket) \
  | jq .result
```

### Recipe — `--notify` script template (Track F)

The `--notify <CMD>` flag fires `sh -c <CMD>` fire-and-forget on every per-hub
change event (skipped on the baseline cycle). The script gates on env vars
and responds however the operator wants:

```sh
#!/bin/sh
# /usr/local/bin/page-on-cap.sh — page on-call when a hub starts refusing connections.

# Gate: only fire when capacity_hits actually moved.
# (NEW/REMOVED kinds use empty strings — `[ -n "$x" ]` filters them out.)
[ -n "$TERMLINK_GOV_CAP_HITS_DELTA" ] && [ "$TERMLINK_GOV_CAP_HITS_DELTA" -gt 0 ] || exit 0

# Body: hand off to your paging tool. The substrate-governor doc has the full
# env-var dict; the common ones are HUB, TS, OLD/NEW_CAP_HITS, CAP_HITS_DELTA.
pd-send-event --routing-key="$PD_KEY" \
  --summary="Hub $TERMLINK_GOV_HUB refused +$TERMLINK_GOV_CAP_HITS_DELTA connection(s)" \
  --severity=warning \
  --custom-detail "ts=$TERMLINK_GOV_TS" \
  --custom-detail "cap_hits=$TERMLINK_GOV_OLD_CAP_HITS → $TERMLINK_GOV_NEW_CAP_HITS"
```

Same pattern works for `RATE_HITS_DELTA` (runaway poller fired the rate-limit)
or `DEDUPE_HITS_DELTA` (spoke retries are landing more than expected). For
reach transitions, gate on `$TERMLINK_GOV_NEW_REACH = "fail"` instead.

The script's environment is documented inline in the `--help` text and on the
CLAUDE.md BACKPRESSURE row.

The fleet view returns `{ok, total, reachable, hubs[], summary}` —
the `summary` rollup includes `total_*` sums plus `hubs_at_capacity`
and `hubs_rate_limited` counts. Per-hub failures (timeout / RPC error
on older hubs that lack the verb) are surfaced as `{ok:false, error}`
without short-circuiting the rest of the fleet.

### Recipe — set tighter limits for canary testing

```sh
# Stop hub if running.
termlink hub stop

# Restart with a small cap to make refusals easy to reproduce.
TERMLINK_MAX_CONNECTIONS=4 \
TERMLINK_RATE_LIMIT_PER_SEC=10 \
  termlink hub start --tcp 0.0.0.0:9100

# Hammer with 20 concurrent listeners — 16 should get
# HUB_AT_CAPACITY (-32019).
for i in $(seq 1 20); do
  termlink listen &
done
wait

# Check the counter.
termlink remote call local hub.governor_status | jq .result.capacity_hits_total
```

## Why per-sender, not per-topic or per-RPC

T-2028 IW-4 disposition (Confidence=3): per-sender aligns with the
trust model (HMAC identifies the sender; the same key gates auth and
gates throughput), while per-topic adds policy complexity for limited
gain and per-RPC is too granular. The trade-off is documented in
the inception §5; this implementation reifies it.

If the future demands per-topic or per-RPC bucketing (e.g. a runaway
`channel.subscribe` loop on a single topic from many senders), that's
a Track-D extension — not a swap-in replacement.

## Defaults and how they were chosen

| Knob | Default | Rationale |
|---|---|---|
| `TERMLINK_MAX_CONNECTIONS` | 256 | ~10× the largest known production fleet (~30 agents × 8 concurrent listeners). Operators on small hosts (Raspberry Pi class) drop to 64 via env. |
| `TERMLINK_RATE_LIMIT_PER_SEC` | 1000 | Comfortable upper bound for AEF-style burst patterns (channel.subscribe + channel.unread + topic_stats in sub-second windows). A stuck poller hammering 10k/sec gets contained without affecting legitimate traffic. |

Both defaults are documented as `pub const` in
`crates/termlink-hub/src/governor.rs`
(`DEFAULT_MAX_CONNECTIONS`, `DEFAULT_RATE_LIMIT_PER_SEC`) — change them
there if the fleet shape shifts.

Setting `TERMLINK_RATE_LIMIT_PER_SEC=0` disables per-sender
rate-limiting entirely. There is no equivalent disable for the
connection cap — it's load-bearing (a hub with `max=u32::MAX` would
still exhaust file descriptors before refusing anything).

## Failure modes this catches

| Symptom | Cause | Surface |
|---|---|---|
| Hub OOMs under burst | Unbounded connection accumulation | `capacity_hits_total > 0` (operator alerts before OOM) |
| One stuck poller (`while true; do termlink_inbox_status; done`) wedges every other agent | Per-RPC unfairness | `rate_hits_total > 0` for that sender; other senders unaffected |
| Cross-fleet identity confusion ("which agent is hammering us?") | No per-sender visibility | `rate_buckets_active` + log lines tagged by sender |

T-1991 was the original "found in production not predicted" — `hub.governor_status` is what would have surfaced it pre-wedge.

## What this does NOT do

- **Backpressure on in-flight RPCs.** A single slow `channel.subscribe`
  with a 30s timeout still ties up a tokio task — only NEW work is
  refused. Slow-handler backpressure is a separate concern (T-2017
  worker-starvation arc).
- **Cross-hub aggregate limits.** Each hub enforces its own cap; a
  fleet of 10 hubs serves up to 2560 simultaneous connections by
  default.
- **Burst smoothing.** The token bucket is leaky-after-empty, not
  smoothing — a burst of 1000 from `alice` succeeds, the 1001st
  fails. If you need smoothing, set a lower rate (the bucket is also
  the burst capacity).
- **Authentication gating.** Refusals happen BEFORE auth (the
  attacker doesn't need a valid token to hit the cap), but cap
  exhaustion does NOT downgrade auth on existing connections. The
  governor is orthogonal to the auth path.

## Related

- T-2018 ADR §6 #10 — the framing this resolves
- T-2028 inception — PARTIAL-GO recommendation that fanned this out
- T-1991 — agent-presence bloat in production (precedent for "found
  not predicted")
- T-1166 — Track A retention audit follows separately (different
  task; this one is Track B exclusively)
- `governor::evict_idle` — wire-up reserved for the bucket-eviction
  housekeeping follow-up (not in T-2048's scope; bounded by sender
  diversity in practice).
