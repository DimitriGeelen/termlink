# Substrate tunables — TERMLINK_* env var reference (T-2159)

Single operator-facing reference for every `TERMLINK_*` env var that
tunes substrate behavior. Pre-T-2159 these were scattered across 5+
per-primitive ops docs; this doc consolidates them so an operator
tuning a new deployment can find every knob in one place.

For each variable: name, default, valid range, when-to-tune-up,
when-to-tune-down, symptom-when-misconfigured, and a link back to
the per-primitive doc that owns the deep context.

This is a reference, not a tutorial. New to substrate? Start at
`substrate-getting-started.md` and come back here when you need to
adjust a knob.

---

## Hub-side tunables — set BEFORE `termlink hub start`

These shape the hub process itself (memory ceilings, refusal thresholds,
where state lives across restarts). Wrong values here cause silent
substrate corruption that surfaces hours or days later as auth-mismatch,
TOFU drift, or backpressure cascades. Always set them via the launcher
(systemd unit / watchdog script / shell wrapper) — the process must
read them at startup; runtime change requires `hub restart`.

| Variable | Default | Range | Purpose | Tune up when | Tune down when | Symptom if wrong | Deep doc |
|----------|---------|-------|---------|--------------|----------------|------------------|----------|
| `TERMLINK_RUNTIME_DIR` | `/tmp/termlink-0` | any writable path | Where the hub persists `hub.secret`, `hub.cert.pem`, `hub.sock` across restarts | N/A — this is **categorical**, not numeric. **Always move OFF /tmp on production hosts.** | N/A | PL-021 cascade: hub regenerates secret + TLS cert every reboot. Every client sees auth-mismatch + TOFU drift. Surfaces 1+ days after install when the host reboots for the first time. | `termlink-hub-runtime-migration.md`; CLAUDE.md §"Special case — volatile runtime_dir" |
| `TERMLINK_MAX_CONNECTIONS` | `256` | 1..=65535 | Concurrent TCP connection cap; refusals return `HUB_AT_CAPACITY` (-32019) with `retry_after_ms` | Hosting >100 simultaneous agents/workers; cap-hits accumulating in `hub.governor_status` | Memory-constrained host; want loud refusal before OOM | Capacity at 256 + high concurrency → `capacity_hits_total > 0` in `/governor`; new connections fail with HUB_AT_CAPACITY error | `substrate-governor.md` |
| `TERMLINK_RATE_LIMIT_PER_SEC` | `1000` | 1..=u32::MAX | Per-sender RPC rate cap; refusals return `RATE_LIMITED` (-32008) with `retry_after_ms` + `data.sender` | High-throughput orchestrator (>1000 ops/sec/sender legit traffic); rate_hits accumulating | Defending against runaway producer; tighter SLA on neighbor isolation | rate_hits_total grows + legitimate workers see RATE_LIMITED storms; or producers misbehaving and never hit a ceiling | `substrate-governor.md` |
| `TERMLINK_DEDUPE_TTL_MS` | `300000` (5min) | 0..=u32::MAX | Post-idempotency LRU TTL — how long a `(sender_id, client_msg_id)` cache entry survives | Workers that may retry posts 5min+ after the first attempt (long-flush offline queue, sleeping reconnect) | Memory-constrained hub; very-short-cycle workloads | `dedupe_hits_total` lower than expected → spoke retries DOUBLE-APPLYING (look for duplicate work-units processed). 0 means dedupe disabled (NOT recommended — every retry double-applies). | `substrate-post-idempotency.md` |
| `TERMLINK_DEDUPE_CAPACITY` | `10000` | 1..=u32::MAX | Max LRU entries before LRU eviction starts dropping old entries (regardless of TTL) | High-rate fleet (>10000 distinct `(sender, msg_id)` pairs per 5min window) | Memory-constrained hub with low message rate | LRU evicting entries before their TTL expires → late retries from a slow sender double-apply because their entry was evicted (look for unexpected duplicate posts after a hub blip + reconnect) | `substrate-post-idempotency.md` |
| `TERMLINK_CV_INDEX_CAP_PER_TOPIC` | `1000` | 1..=u32::MAX | Max distinct `cv_key` entries the `cv_index` tracks per topic; overflow drops the annotation (post still lands) | High-fanout broadcast-with-replay topic (e.g. agent-presence in a 1000+ agent fleet) | Memory pressure + low cv_key cardinality use | `cv_index_overflow_total > 0` in `/governor` → late-joiners on subscribe-with-current-value miss state for evicted keys; producer mis-emitting cv_key (e.g. timestamp instead of stable id) | `substrate-broadcast-with-replay.md` |
| `TERMLINK_RATE_EVICT_IDLE_THRESHOLD_MS` | `600000` (10min) | 0..=u32::MAX | Per-sender rate bucket eviction — buckets idle longer than this are removed from memory | Memory pressure from churning-sender population (many short-lived agents) | Want senders to retain their bucket across short pauses (avoid re-allocating on every reconnect) | Bucket evicted mid-burst → next message after eviction gets a fresh allowance window (legitimate looking, but may mask runaway producers in observability if you're correlating to bucket-age) | `substrate-governor.md` |
| `TERMLINK_RATE_EVICT_INTERVAL_SEC` | `60` | 1..=u32::MAX | Eviction sweep interval — how often the hub walks rate buckets to evict idle ones | High-churn fleet (eviction lag = memory pressure) | Stable fleet (sweep is wasted CPU when buckets aren't aging out) | Memory creep on the hub process correlated with churning senders + long eviction interval | `substrate-governor.md` |

**Critical operational rule:** Always set `TERMLINK_RUNTIME_DIR` on
production hosts. The default `/tmp/termlink-0` is volatile on every
modern Linux distro (either tmpfs OR systemd-tmpfiles D-rule, the two
distinct PL-021 mechanisms). `scripts/substrate-preflight.sh` (T-2154)
detects this at deploy time — run `/preflight` after every fresh
install, container rebuild, or systemd unit change.

---

## Client-side tunables — set BEFORE invoking the CLI

These shape per-client behavior (identity, queue depth, capability
advertisement). They can change per-session or per-invocation — the
client reads them at process start.

| Variable | Default | Range | Purpose | When to set | When to leave unset | Symptom if wrong | Deep doc |
|----------|---------|-------|---------|-------------|---------------------|------------------|----------|
| `TERMLINK_AGENT_ID` | `(none — resolves from `~/.termlink/be-reachable.state`)` | any string | Identity override for `--sender` / `--claimer` (T-1857 resolution chain step 1) | Scripted automation that doesn't go through `/be-reachable`; CI runs | Interactive sessions using `/be-reachable` (auto-sets the state file) | Sender unresolved → CLI refuses with hint (loud, not silent — T-1857 invariant). Mismatched value → -32014 sender_id mismatch (T-1427 strict binding refuses) | substrate-claim-primitive.md; CLAUDE.md §Identity resolution |
| `TERMLINK_CLAIMER` | `(none — resolves from $TERMLINK_AGENT_ID then state file)` | any string | Claimer-identity override for `channel.claim` / `claim-transfer` / `release` (T-1857 step 1) | Multi-identity orchestrators where the claimer differs from sender | Single-identity sessions | CLAIM_NOT_OWNED (-32017) on release/renew/transfer because the hub thinks a different identity owns the claim | substrate-claim-primitive.md |
| `TERMLINK_CAPABILITIES` | `(empty csv)` | comma-separated tokens | Capabilities advertised on agent-presence heartbeat (consumed by `/find-idle --capability X` filter) | Worker advertising specific skills (e.g. `rust,deploy,sql`) | Generic listener that should match any capability filter | `/find-idle --capability X` excludes this worker silently; orchestrator can't dispatch | `agent-find-idle.md`; T-2091 (`/peers --filter-capability`) |
| `TERMLINK_OUTBOUND_CAP` | `1000` | 1..=u32::MAX | Offline-queue (`~/.termlink/outbound.sqlite`) max depth; reaching cap returns `QueueError::QueueFull` (R3 loud-fail) | Worker may face long hub blips (>1000 queued posts before drain) | Default is fine for almost all cases | Queue-full loud-fail at unexpectedly low depth, OR queue grows unboundedly during long blip if cap raised too aggressively + disk fills | `substrate-offline-queue-recipe.md` |
| `TERMLINK_IDENTITY_DIR` | `~/.termlink/identity` | any writable path | Where the client persists its identity keypair (used for sender_id signing per T-1427) | Multi-user host with shared `~/` (set per user); CI with ephemeral filesystem | Single-user host | Identity keypair regenerated on every invocation → sender_id rotates → all peer pinning breaks; or shared keypair across multiple identities → strict-binding refusals | (no per-primitive doc; T-1427 strict identity binding inception) |

---

## Watch-loop event-hook env vars — set BY the substrate watch loops, READ BY operator-provided notify scripts

When operators wire `--notify <cmd>` into a watch loop (e.g.
`/governor --watch --notify /usr/local/bin/page.sh`), the watch loop
exports per-event env vars for the notify script to read. **These are
NOT tunables operators set** — they are the contract between the watch
loop and the notify script. Listed here for one-place discovery.

| Watch loop | Env vars exported per event |
|------------|-----------------------------|
| `fleet governor-status --watch --notify` (T-2065) | `TERMLINK_GOV_HUB`, `TERMLINK_GOV_CHANGE_KIND`, `TERMLINK_GOV_TS`, `TERMLINK_GOV_OLD_REACH`, `TERMLINK_GOV_NEW_REACH`, `TERMLINK_GOV_OLD_CAP_HITS`, `TERMLINK_GOV_NEW_CAP_HITS`, `TERMLINK_GOV_CAP_HITS_DELTA`, `TERMLINK_GOV_OLD_RATE_HITS`, `TERMLINK_GOV_NEW_RATE_HITS`, `TERMLINK_GOV_RATE_HITS_DELTA`, `TERMLINK_GOV_OLD_DEDUPE_HITS`, `TERMLINK_GOV_NEW_DEDUPE_HITS`, `TERMLINK_GOV_DEDUPE_HITS_DELTA`, `TERMLINK_GOV_OLD_CV_OVERFLOW`, `TERMLINK_GOV_NEW_CV_OVERFLOW`, `TERMLINK_GOV_CV_OVERFLOW_DELTA`, `TERMLINK_GOV_OLD_EVICTED`, `TERMLINK_GOV_NEW_EVICTED`, `TERMLINK_GOV_EVICTED_DELTA` (T-2119 / T-2140) |
| `fleet doctor --watch --notify` (T-1669) | `TERMLINK_WATCH_HUB`, `TERMLINK_WATCH_CHANGE_KIND`, `TERMLINK_WATCH_TS`, `TERMLINK_WATCH_OLD_CONN`, `TERMLINK_WATCH_NEW_CONN`, `TERMLINK_WATCH_OLD_PIN`, `TERMLINK_WATCH_NEW_PIN`, `TERMLINK_WATCH_OLD_LEGACY`, `TERMLINK_WATCH_NEW_LEGACY` |
| `channel claims-summary --watch --notify` (T-2072) | `TERMLINK_CLAIM_TOPIC`, `TERMLINK_CLAIM_CHANGE_KIND`, `TERMLINK_CLAIM_TS`, `TERMLINK_CLAIM_HUB`, `TERMLINK_CLAIM_OLD_STUCK`, `TERMLINK_CLAIM_NEW_STUCK`, `TERMLINK_CLAIM_ACTIVE_COUNT`, `TERMLINK_CLAIM_EXPIRED_COUNT`, `TERMLINK_CLAIM_OLDEST_AGE_MS` |
| `agent find-idle --watch --notify` (T-2079) | `TERMLINK_IDLE_AGENT_ID`, `TERMLINK_IDLE_CHANGE_KIND`, `TERMLINK_IDLE_TS`, `TERMLINK_IDLE_ROLE`, `TERMLINK_IDLE_CAPABILITIES`, `TERMLINK_IDLE_LAST_HEARTBEAT_MS` |
| `channel queue-status --watch --notify` (T-2084) | `TERMLINK_QUEUE_CHANGE_KIND`, `TERMLINK_QUEUE_TS`, `TERMLINK_QUEUE_OLD_PENDING`, `TERMLINK_QUEUE_NEW_PENDING`, `TERMLINK_QUEUE_OLDEST_AGE_MS`, `TERMLINK_QUEUE_PATH` |
| `substrate status --watch --notify` (T-2113) | `TERMLINK_SUBSTRATE_CHANGE_FIELD`, `TERMLINK_SUBSTRATE_CHANGE_OLD`, `TERMLINK_SUBSTRATE_CHANGE_NEW`, `TERMLINK_SUBSTRATE_TS` |

Common gate pattern in notify scripts:

```sh
#!/bin/sh
# only fire on transition to stuck:
[ "$TERMLINK_CLAIM_NEW_STUCK" = "true" ] || exit 0
# delegate to paging:
exec /usr/local/bin/page-team.sh "topic $TERMLINK_CLAIM_TOPIC went stuck"
```

The gate-then-delegate pattern keeps notify scripts focused; the watch
loop never blocks even if the script hangs (fire-and-forget by design).

---

## Operational recipes

**Production hub bring-up** (hub-side env vars):

```sh
# systemd drop-in: /etc/systemd/system/termlink-hub.service.d/env.conf
[Service]
Environment="TERMLINK_RUNTIME_DIR=/var/lib/termlink"
Environment="TERMLINK_MAX_CONNECTIONS=512"
Environment="TERMLINK_RATE_LIMIT_PER_SEC=2000"
# leave dedupe / cv-index / eviction at defaults until governor surfaces pressure
```

After install:

```sh
systemctl daemon-reload && systemctl restart termlink-hub
/preflight                   # confirm runtime_dir + state look right
/governor                    # confirm no immediate cap/rate hits
```

**Multi-identity worker** (client-side env vars):

```sh
# worker process — distinct identity from the orchestrator
export TERMLINK_AGENT_ID=worker-alpha-rust
export TERMLINK_CAPABILITIES=rust,deploy
exec /usr/local/bin/substrate-worker-loop.sh
```

**Long-blip-tolerant worker** (client-side queue depth):

```sh
# tolerate a 4-hour hub blip at 1msg/sec:
export TERMLINK_OUTBOUND_CAP=15000
```

Pair with `disk_free` monitoring — at 15000 queued envelopes the
SQLite file can reach 100MB+.

---

## Related

- `substrate-getting-started.md` — entry point; run `/preflight` first
- `substrate-orchestrator-recipe.md` — master AEF integration walkthrough
- `substrate-governor.md` — deep on connection-cap / rate-limit / dedupe
- `substrate-post-idempotency.md` — deep on dedupe LRU
- `substrate-broadcast-with-replay.md` — deep on cv_index
- `substrate-offline-queue-recipe.md` — deep on outbound queue
- `substrate-claim-primitive.md` — deep on identity resolution + claim lifecycle
- `termlink-hub-runtime-migration.md` — PL-021 prevention
- `agent-find-idle.md` — capability advertisement consumer
- T-2154 (`scripts/substrate-preflight.sh`) — deploy-time validation that
  enforces the categorical tunable (`TERMLINK_RUNTIME_DIR` not on /tmp)
- `/preflight` (T-2158) — skill-layer wrap of the same
