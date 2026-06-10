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
  "max_rate_per_sec": 1000,
  "dedupe_entries_active": 0,
  "dedupe_hits_total": 0,
  "dedupe_ttl_ms": 300000,
  "cv_index_entries_active": 0,
  "cv_index_topics_active": 0,
  "cv_index_overflow_total": 0,
  "cv_index_cap_per_topic": 1000
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
  `RateGovernor::evict_idle` (T-2137 wired this into hub startup —
  see "Reading rate_buckets_active" below).
- `rate_buckets_evicted_total` *(T-2139)*: monotonic count of buckets
  dropped by the eviction loop since hub start. **The smoking-gun
  signal that the T-2137 eviction loop is actually firing.** Zero on
  fresh hub (no buckets to evict yet) or on a hub running a pre-T-2137
  binary (loop never wired); non-zero and rising means eviction is
  running and keeping `rate_buckets_active` bounded. Pair with
  `rate_buckets_active` for the full retention picture: a stuck-at-zero
  counter alongside a growing `rate_buckets_active` is the explicit
  pre-T-2137 binary signal — upgrade the hub.
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

T-2110 added four cv_index telemetry fields surfacing the
broadcast-with-replay primitive (substrate #9) pressure surface (older
hubs omit them — CLI/MCP renderers fall back to `n/a`):

- `cv_index_entries_active`: total `(topic, cv_key)` entries live in the
  hub's per-topic cv_index map across every topic. Bounded by
  `cv_index_topics_active × cv_index_cap_per_topic`. Read-side accessed
  by `channel cv-keys` / `channel.subscribe --include-current-value`.
- `cv_index_topics_active`: number of distinct topics that carry at
  least one cv_index entry. Steady-state under normal traffic; a
  steady climb signals new producers wiring `metadata.cv_key=...`.
- `cv_index_overflow_total`: monotonic counter; every time a
  `channel.post` carrying `metadata.cv_key` exceeded the per-topic cap
  and the cv_key annotation was dropped (post stayed atomic), this
  increments. **Binary signal — ANY non-zero value is operator-actionable.
  Almost always means a producer is mis-emitting `cv_key`** (e.g. timestamp
  instead of stable id) and silently exhausting the cap. T-2118 wires
  this into the `--only-pressured` predicate so the cv-key bug surfaces
  alongside cap_hits / rate_hits in fleet rollups; T-2119 wires it into
  the watch/notify/log/history surfaces so operators get paged on each
  new overflow event.
- `cv_index_cap_per_topic`: per-topic entry cap at hub-start; matches
  `TERMLINK_CV_INDEX_CAP_PER_TOPIC` (default 1000). Bumping requires a
  hub restart. Cap-overflow drops the cv_key annotation but keeps the
  post — substrate-correctness is preserved, only discovery cost
  degrades to the O(N_events) walk.

See `docs/operations/substrate-broadcast-with-replay.md` for the
producer/consumer wiring; this surface is the read-side health signal.

### Reading rate_buckets_active (T-2138)

The natural question after seeing a `rate_buckets_active` value is
"is this number normal?". Answer:

- **Post-T-2137 (eviction wired):** bounded by `sender rate × eviction
  interval`. For a steady-state fleet with K distinct senders heartbeating
  every 30 s and the default 60 s eviction interval / 5 min idle
  threshold, expect roughly `K` buckets (one per active sender) plus a
  trailing tail of senders idle <5 min. A 5-agent fleet should sit at
  5–10. A 30-agent fleet should sit at 30–50.

- **Anomalously high.** Either:
  - **Burst event in the last eviction interval.** Many distinct
    sender_ids hit the hub briefly (e.g. a fan-out test, a probe
    storm). The count will fall on the next eviction sweep. Wait one
    `TERMLINK_RATE_EVICT_INTERVAL_SEC` cycle (default 60 s) and re-probe.
  - **Eviction not running (pre-T-2137 binary).** The hub binary
    pre-dates T-2137 and the bucket HashMap grows unbounded. Field
    diagnosis: `rate_buckets_active` keeps climbing across multiple
    minutes without falling. Pre-T-2137 production observation:
    `258_236` against a 5-agent fleet (~31 MB held). Fix: upgrade the
    hub binary — `scripts/fleet-deploy-binary.sh`.

- **Anomalously low (zero, with active workers).** The rate governor
  hasn't seen any RPC yet. Either the hub just started, or all callers
  are bypassing the rate path. Not actionable unless paired with
  zero `rate_hits_total` across a working day.

Tune the eviction loop via `TERMLINK_RATE_EVICT_INTERVAL_SEC` (clamped
5..=3600 s) and `TERMLINK_RATE_EVICT_IDLE_THRESHOLD_MS` (clamped
1000..=3_600_000 ms) at hub start. Defaults: 60 s / 300_000 ms (5 min).

### Version-skew diagnosis (T-2138)

When `fleet governor-status` returns `-32001 / Missing 'target' in
params` for a hub, that hub is running a **pre-T-2048 binary** —
older than the `hub.governor_status` RPC. The older hub's
unknown-method dispatch routes through `event.emit_to`, which fails
on the missing `target` param and returns `-32001` instead of the
correct `-32601 / Method not found`.

Symptom (real fleet output):

```
ring20-dashboard         RPC error -32001: Missing 'target' in params
ring20-management        RPC error -32001: Missing 'target' in params
```

This is NOT a config bug, NOT a target-param problem, NOT a routing
issue. It's strictly "that hub needs a binary upgrade." Fix:

```sh
# Deploy current binary to the lagging hub (probe first per PL-100):
bash scripts/fleet-deploy-binary.sh --probe --target ring20-management
bash scripts/fleet-deploy-binary.sh --target ring20-management
# Hub will restart; existing client TOFU pins are preserved (T-985).
```

After upgrade, re-probe — `fleet governor-status` should now return
the full counter block for the upgraded hub. If it doesn't, the
binary swap may have failed silently (check `fw fleet doctor` for
auth-mismatch).

### Recipe — operator probe

The hub.governor_status envelope is reachable through four parallel
routes — pick the one that fits the caller:

```sh
# 1. Local hub status with governor counters inline (T-2060; same envelope)
termlink hub status --governor --json

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

For cv_index overflow (T-2119) — a producer is mis-emitting `cv_key` and
the per-topic cap saturated — gate on `CV_OVERFLOW_DELTA` and page the
producer team:

```sh
#!/bin/sh
# /usr/local/bin/page-on-cv-overflow.sh — page producer-team when cv_index
# saturates (almost always means a producer is mis-emitting cv_key —
# e.g. timestamp instead of stable id).
[ -n "$TERMLINK_GOV_CV_OVERFLOW_DELTA" ] && [ "$TERMLINK_GOV_CV_OVERFLOW_DELTA" -gt 0 ] || exit 0
pd-send-event --routing-key="$PD_PRODUCER_KEY" \
  --summary="Hub $TERMLINK_GOV_HUB cv_index overflow +$TERMLINK_GOV_CV_OVERFLOW_DELTA (producer mis-emitting cv_key — investigate)" \
  --severity=warning \
  --custom-detail "ts=$TERMLINK_GOV_TS" \
  --custom-detail "cv_overflow=$TERMLINK_GOV_OLD_CV_OVERFLOW → $TERMLINK_GOV_NEW_CV_OVERFLOW" \
  --custom-detail "diagnostic=termlink channel cv-keys <topic> to inspect which producer is saturating the cap"
```

cv_overflow is binary — no threshold tuning, any non-zero delta is an
operator-actionable producer bug. T-2110 emits the counter, T-2118
wires it into `--only-pressured`, T-2119 surfaces per-event deltas in
`--notify` / `--log` / `fleet governor-history`.

For the rate-bucket eviction loop (T-2140) — the T-2137 GC sweep that
keeps `rate_buckets_active` bounded as senders churn — gate on
`TERMLINK_GOV_EVICTED_DELTA`. **Unlike cap/rate/cv_overflow which are
pressure signals, `evicted` is a HEALTH signal:** non-zero is the
desired steady state (the GC loop is doing its job, keeping memory
bounded). Page on a *sudden surge* (sender churn, DDoS, or a regex
bug producing synthetic senders), not on the absolute value and not
on zero:

```sh
# /usr/local/bin/page-on-eviction-surge.sh — alert when the bucket-GC loop
# is evicting WAY more than steady state (suggests sender churn / DDoS).
#!/bin/sh
[ -n "$TERMLINK_GOV_EVICTED_DELTA" ] || exit 0   # pre-T-2139 hub — counter not exposed
[ "$TERMLINK_GOV_EVICTED_DELTA" -gt 100 ] || exit 0   # 100/cycle = surge
pagerctl alert \
  --summary="Hub $TERMLINK_GOV_HUB rate-bucket eviction surge +$TERMLINK_GOV_EVICTED_DELTA (sender churn?)" \
  --severity=warning \
  --custom-detail "ts=$TERMLINK_GOV_TS" \
  --custom-detail "evicted=$TERMLINK_GOV_OLD_EVICTED → $TERMLINK_GOV_NEW_EVICTED" \
  --custom-detail "diagnostic=watch sender distribution; check for regex bug producing fake senders"
```

Pre-T-2139 hubs return an empty `TERMLINK_GOV_EVICTED_DELTA` (the
counter didn't exist before T-2139 exposed it via the snapshot RPC).
The `[ -n "$..." ]` guard ensures the script silently no-ops on older
hubs instead of erroring. Once the fleet is fully on T-2139+, drop
the `-n` guard.

T-2137 wires the eviction loop into hub startup, T-2139 exposes the
counter, T-2140 surfaces per-event deltas in `--notify` / `--log` /
`fleet governor-history`. The counter is intentionally NOT in
`--only-pressured` — the loop firing is healthy, not pressure.

The script's environment is documented inline in the `--help` text and on the
CLAUDE.md BACKPRESSURE row.

### Recipe — `--log` audit trail (Track G)

The `--log <PATH>` flag appends one NDJSON line per transition / new /
removed event during `--watch`. Mirror of T-1671's `~/.termlink/rotation.log`
pattern, applied to governor telemetry. Use when "I need a forensic trail
of capacity events" matters but the operator can't keep a watch terminal
open continuously.

```sh
# Just the log — no paging, no terminal output stays around.
termlink fleet governor-status --watch 30 --log ~/.termlink/governor.log

# Composed: paging via --notify AND forensic trail via --log in one command.
# One-liner form: termlink fleet governor-status --watch 30 --log ~/.termlink/governor.log --notify /usr/local/bin/page-on-cap.sh
termlink fleet governor-status --watch 30 \
  --log    ~/.termlink/governor.log \
  --notify /usr/local/bin/page-on-cap.sh
```

NDJSON schema (one event per line, flat for jq-friendliness):

```json
{
  "ts": "2026-06-08T22:58:02Z",
  "hub": "workstation-107-public",
  "kind": "transition",
  "old_reach": "ok", "new_reach": "ok",
  "old_conn_active": 3, "new_conn_active": 4,
  "old_cap_hits": 0, "new_cap_hits": 0, "cap_hits_delta": 0,
  "old_rate_hits": 0, "new_rate_hits": 0, "rate_hits_delta": 0,
  "old_dedupe_hits": null, "new_dedupe_hits": null, "dedupe_hits_delta": null,
  "old_cv_overflow": 0, "new_cv_overflow": 0, "cv_overflow_delta": 0,
  "old_evicted": 0, "new_evicted": 15, "evicted_delta": 15
}
```

Counters are numeric (jq filters work directly), `kind` is
`"transition" | "new" | "removed"`, and missing-side fields render as JSON
`null` (NOT omitted — so jq filters never error on missing keys). Reach
serializes as `"ok" | "fail" | null`.

#### Common forensic queries

```sh
# When did hub X last refuse connections this week?
jq -c 'select(.hub=="ring20-management" and .cap_hits_delta>0) | {ts, cap_hits_delta}' \
  ~/.termlink/governor.log

# All rate-limit incidents in the last 24h.
since="$(date -u -d '24 hours ago' +%Y-%m-%dT%H:%M:%SZ)"
jq -c --arg s "$since" 'select(.ts>$s and .rate_hits_delta>0) | {ts, hub, rate_hits_delta}' \
  ~/.termlink/governor.log

# Dedupe absorption — spoke retries landing more than expected (steady-state suspicious).
jq -c 'select(.dedupe_hits_delta>0) | {ts, hub, dedupe_hits_delta}' \
  ~/.termlink/governor.log

# cv_index overflow (T-2119) — a producer mis-emitted cv_key and saturated the
# per-topic cap. ANY non-zero delta is operator-actionable (no threshold tuning).
# Pair with `termlink channel cv-keys <topic>` to identify the saturating topic.
jq -c 'select(.cv_overflow_delta>0) | {ts, hub, cv_overflow_delta}' \
  ~/.termlink/governor.log

# Rate-bucket eviction surges (T-2140) — the GC loop firing way more than
# steady state. evicted is a HEALTH signal (non-zero is healthy), but a
# *surge* above the steady-state baseline points at sender churn / DDoS /
# regex bug producing synthetic senders. Tune the threshold per-fleet.
jq -c 'select(.evicted_delta>100) | {ts, hub, evicted_delta}' \
  ~/.termlink/governor.log

# What's the steady-state evicted rate this week? (gives you a baseline
# to set the page-on-surge threshold.)
jq -s 'map(.evicted_delta // 0) | add / length' ~/.termlink/governor.log
```

#### Operational notes

- **Append-only.** The watch loop never truncates. Manage size via `logrotate`
  or manual archival — same pattern as `rotation.log` (T-1671).
- **Best-effort writes.** Disk-full / permission-denied errors emit one
  stderr line per failed append but never crash the watch. The next
  successful cycle resumes appending — so a brief log-disk problem doesn't
  break the surveillance loop.
- **Parent dir auto-created.** `~/.termlink/governor.log` works on a fresh
  install without `mkdir -p`.
- **Native read-side: `fleet governor-history` (T-2068).** See the next
  recipe — it walks `~/.termlink/governor.log` (or a `--log PATH`
  override matching the watch loop), filters by window + hub, renders
  per-event lines + per-hub aggregate footers. Closes the §6 #10
  substrate-governor arc end-to-end.

### Recipe — `fleet governor-history` (Track G read-side)

The retrospective companion to `--watch --log`. Reads the NDJSON
audit trail without keeping a watch terminal open. Mirror of
`fleet history` (T-1671) but pointed at `governor.log` instead of
`rotation.log`.

```sh
# Default: last 7 days, every hub, human format.
termlink fleet governor-history

# Last 24 hours, only ring20-management.
termlink fleet governor-history --since 1 --hub ring20-management

# 30 days, JSON for dashboards / pipelines.
termlink fleet governor-history --since 30 --json

# Custom log path (watch-loop was run with a non-default --log).
termlink fleet governor-history --log /var/log/termlink/governor.log
```

Human output is one anchored line per matching entry plus a
per-hub aggregate footer:

```
2026-06-08T23:34:02Z  local-test               transition  conn=3→4 cap=0→0(+0) rate=0→0(+0) dedupe=n/a→n/a(+n/a) cv_overflow=0→0(+0) evicted=0→15(+15)
2026-06-08T23:51:11Z  ring20-management        transition  conn=251→256(...) cap=0→4(+4) rate=12→18(+6) dedupe=n/a→n/a(+n/a) cv_overflow=0→3(+3) evicted=200→42(+42)

Summary: 2 event(s) in last 7 day(s):
  local-test                 1 event(s)  cap_hits=+0 rate_hits=+0 dedupe_hits=+0 cv_overflow=+0 evicted=+15
  ring20-management          1 event(s)  cap_hits=+4 rate_hits=+6 dedupe_hits=+0 cv_overflow=+3 evicted=+42
```

The aggregate footer is the high-signal output: `cap_hits=+4` on a
hub answers "was this hub turning anyone away in this window?" at
a glance.

`--json` mode emits one NDJSON line per matching entry followed by
a single summary object:

```json
{"ts":"2026-06-08T23:34:02Z","hub":"local-test","kind":"transition","old_conn_active":3,"new_conn_active":4,"old_cap_hits":0,"new_cap_hits":0,"cap_hits_delta":0,"old_rate_hits":0,"new_rate_hits":0,"rate_hits_delta":0,"old_dedupe_hits":null,"new_dedupe_hits":null,"dedupe_hits_delta":null,"old_cv_overflow":0,"new_cv_overflow":0,"cv_overflow_delta":0,"old_evicted":0,"new_evicted":15,"evicted_delta":15}
{"total":1,"per_hub":{"local-test":{"events":1,"cap_hits_total":0,"rate_hits_total":0,"dedupe_hits_total":0,"cv_overflow_hits_total":0,"evicted_total":15}},"since_days":7,"hub_filter":null,"malformed_lines_skipped":0,"log_path":"/root/.termlink/governor.log"}
```

Empty/missing log prints a hint pointing back at the watch verb so
the operator knows how to start capturing. Malformed lines are
skipped with stderr warnings (first 3 surfaced) and tallied in
`malformed_lines_skipped`. Out-of-range `--since` (must be 1..=365)
errors with a useful message rather than silently clamping.

Read-only by contract: no auth, no network, no log mutation. Safe
for cron / dashboards / CI.

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
  termlink agent listen &
done
wait

# Check the counter.
termlink hub status --governor --json | jq .governor.capacity_hits_total
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
| Late-joiner replay slows to O(N_events) walk despite producer setting `cv_key` | Producer mis-emitting `cv_key` (e.g. timestamp instead of stable id) saturates per-topic cv_index cap; cv_key annotations silently drop | `cv_index_overflow_total > 0` (T-2110); fires `--only-pressured` (T-2118); per-event delta in `--watch --notify --log` + `fleet governor-history` (T-2119) |

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

- **Master integration recipe (T-2124):** [`substrate-orchestrator-recipe.md`](substrate-orchestrator-recipe.md) — shows where governor counters fit in the end-to-end work-stealing pattern, plus the failure-mode table mapping `cap_hits / rate_hits / cv_overflow` to operator action.
- T-2018 ADR §6 #10 — the framing this resolves
- T-2028 inception — PARTIAL-GO recommendation that fanned this out
- T-1991 — agent-presence bloat in production (precedent for "found
  not predicted")
- T-1166 — Track A retention audit follows separately (different
  task; this one is Track B exclusively)
- `governor::evict_idle` — wire-up reserved for the bucket-eviction
  housekeeping follow-up (not in T-2048's scope; bounded by sender
  diversity in practice).
- T-2110 — cv_index telemetry (entries / topics / overflow / cap)
  surfaced via this same `hub.governor_status` envelope. Closes the §6
  #9↔#10 cross-reference at the counter level.
- T-2118 — `--only-pressured` predicate fires on `cv_index_overflow_total > 0`
  (CLI + MCP). Producer bug surfaces alongside cap_hits / rate_hits in
  fleet rollups.
- T-2119 — watch / notify / log / history surfaces carry cv_overflow
  deltas end-to-end. `page-on-cv-overflow.sh` recipe above.
- `docs/operations/substrate-broadcast-with-replay.md` — substrate #9
  producer/consumer wiring (the surface cv_index telemetry monitors).
