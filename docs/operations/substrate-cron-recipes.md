# Substrate cron recipes — production monitoring patterns (T-2162)

Ready-to-install cron + notify-script templates for every substrate
observability surface. Operators copy six cron lines and six small shell
scripts and get production-grade monitoring with empty-log-is-healthy
audit trails, fire-on-transition paging, and forensic retrospective.

Pre-T-2162 these patterns were scattered across per-primitive ops docs
as one-liners. This doc consolidates them so an operator wiring a fresh
host is one copy-paste away from a full monitoring stack.

The recipes share a pattern (T-2065 / T-2072 / T-2079 / T-2084 / T-2113):

1. A `--watch <secs>` form polls the substrate verb on a tight loop.
2. A `--notify <CMD>` form fires the operator's script per-event,
   fire-and-forget (hangs and crashes don't block the watch).
3. A `--log <PATH>` form appends NDJSON per-event to a forensic trail.
4. A separate `-history` verb walks the trail retrospective.

The notify scripts in this doc all follow the same shape: gate on the
event field that indicates "this is the bad-state transition", then
exec a paging command (operator-specific — substitute Slack webhook,
PagerDuty trigger, email, etc.).

## Common preamble — once per host

All recipes assume:

- `termlink` binary on PATH (`/usr/local/bin/termlink` typically)
- `~/.termlink/hubs.toml` populated (validate with `/preflight`)
- Notify scripts live under `/usr/local/bin/` with mode 0755
- Cron files live under `/etc/cron.d/` (USER-field syntax)
- Logs live under a stable path operators monitor (e.g. `~/.termlink/` for
  agent-owned, `/var/log/termlink/` for system-owned)

> ⚠️ **PL-208 / T-2052 — chmod gap.** Every notify script MUST be
> executable. `chmod +x /usr/local/bin/<script>.sh` after creating it,
> and re-check after any package upgrade that touches the script
> (shipped framework scripts have lost their +x bit on container rebuild
> in the past — silent no-op for 17 days in one incident).

If you're new to the substrate, run `/preflight` first to confirm the
environment is correctly configured before wiring monitoring.

---

## Recipe 1: Nightly preflight canary (T-2160)

The deploy-time correctness check fires daily at 05:23 UTC and only
writes to its log when something is wrong.

**Cron file** (`/etc/cron.d/termlink-substrate-preflight-canary`):

```cron
# Source: /opt/termlink/.context/cron/substrate-preflight-canary.crontab
SHELL=/bin/bash
PATH=/usr/local/bin:/usr/bin:/bin

23 5 * * * root cd /opt/termlink && bash scripts/substrate-preflight.sh --quiet >> .context/working/.substrate-preflight-canary.log 2>&1
```

**What fires when:** PL-021 runtime_dir regression (post-reboot /tmp
wipe), missing `~/.termlink/hubs.toml`, dead `be-reachable.state` PID.

**Tuning:** None — the three checks are categorical (binary fail/pass).
If you want a SECOND check daily for higher cadence, copy the line and
change minute/hour offsets.

**Silence false positives:** No false positives — every entry indicates
a real PL-021/G-061-class regression.

**Diagnose on fire:** `tail -50 .context/working/.substrate-preflight-canary.log`
— each entry is framed `=== <ts> ===\n<full preflight output>\n---` so
the failing check + remediation hint are inline.

See `substrate-getting-started.md` for the underlying script (T-2154)
and `/preflight` skill (T-2158).

---

## Recipe 2: Page on hub capacity hits (T-2065)

Fire when a hub starts refusing connections (`capacity_hits_total > 0`
new in a polling cycle). Indicates `TERMLINK_MAX_CONNECTIONS` is too low
for current load, OR a runaway client.

**Notify script** (`/usr/local/bin/page-on-cap-hits.sh`, `chmod +x`):

```sh
#!/bin/sh
# Fire only on cap-hit deltas. Other governor changes (rate, dedupe, cv) ignored.
[ "${TERMLINK_GOV_CAP_HITS_DELTA:-0}" -gt 0 ] || exit 0

# Substitute your paging command. Example: Slack webhook.
exec curl -sS -X POST \
  -H 'Content-Type: application/json' \
  -d "{\"text\":\":rotating_light: hub ${TERMLINK_GOV_HUB} cap_hits +${TERMLINK_GOV_CAP_HITS_DELTA} (now ${TERMLINK_GOV_NEW_CAP_HITS}) — substrate refusing connections, raise TERMLINK_MAX_CONNECTIONS or investigate runaway producer\"}" \
  "$SLACK_WEBHOOK_URL"
```

**Cron file** (`/etc/cron.d/termlink-governor-watch-cap`):

```cron
SHELL=/bin/bash
PATH=/usr/local/bin:/usr/bin:/bin
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/REPLACE_ME

@reboot root cd /opt/termlink && nohup setsid termlink fleet governor-status --watch 30 --notify /usr/local/bin/page-on-cap-hits.sh --log ~/.termlink/governor.log > /var/log/termlink-governor-watch.log 2>&1 &
```

**What fires when:** `capacity_hits_total` increases between cycles on
any hub in `~/.termlink/hubs.toml`.

**Tuning:** Raise `TERMLINK_MAX_CONNECTIONS` (see
`substrate-tunables.md`) for legitimate high-concurrency load; investigate
the rate-limiter (Recipe 3) for runaway producers.

**Silence false positives:** A single cap-hit during a fleet upgrade is
not actionable (existing clients re-pin and re-handshake). Gate the
notify script on `TERMLINK_GOV_CAP_HITS_DELTA > 5` if upgrade cycles
cause noise.

See `substrate-governor.md` for the underlying RPC.

---

## Recipe 3: Page on rate-limit hits (T-2065)

Fire when a hub refuses RPCs (`rate_hits_total > 0` new). Indicates
`TERMLINK_RATE_LIMIT_PER_SEC` is too low OR a misbehaving sender.

**Notify script** (`/usr/local/bin/page-on-rate-hits.sh`):

```sh
#!/bin/sh
[ "${TERMLINK_GOV_RATE_HITS_DELTA:-0}" -gt 0 ] || exit 0
exec curl -sS -X POST \
  -H 'Content-Type: application/json' \
  -d "{\"text\":\":warning: hub ${TERMLINK_GOV_HUB} rate_hits +${TERMLINK_GOV_RATE_HITS_DELTA} — check senders for runaway producer\"}" \
  "$SLACK_WEBHOOK_URL"
```

Reuses the same `governor-status --watch` invocation as Recipe 2 (one
watch loop drives all notify scripts via its single shell-script
dispatch). Operators can either run one watch with one router script
that branches on the changed field, OR (cleaner) run multiple watches
with dedicated scripts — both work; the notify is fire-and-forget so
running 2x watches at 30s isn't expensive.

**Tuning:** See `substrate-tunables.md` for
`TERMLINK_RATE_LIMIT_PER_SEC` semantics. Look at error responses to
identify the offending sender (the hub embeds `data.sender` in the
RATE_LIMITED -32008 error per IW-3).

See `substrate-governor.md`.

---

## Recipe 4: Page on cv_index overflow (T-2119)

Fire when the `cv_index` overflows on any topic — indicates a producer
is mis-emitting `metadata.cv_key` (e.g. timestamp instead of stable id),
saturating the per-topic cap and silently dropping cv-tag annotations on
new posts. Overflow is binary — ANY non-zero is operator-actionable
(producer fix), no tuning threshold needed.

**Notify script** (`/usr/local/bin/page-on-cv-overflow.sh`):

```sh
#!/bin/sh
[ "${TERMLINK_GOV_CV_OVERFLOW_DELTA:-0}" -gt 0 ] || exit 0
exec curl -sS -X POST \
  -H 'Content-Type: application/json' \
  -d "{\"text\":\":rotating_light: hub ${TERMLINK_GOV_HUB} cv_overflow +${TERMLINK_GOV_CV_OVERFLOW_DELTA} — producer mis-emitting metadata.cv_key on some topic, run termlink channel cv-keys <topic> to identify it\"}" \
  "$SLACK_WEBHOOK_URL"
```

**Diagnose:** Run `termlink channel cv-keys <suspect-topic> --json` to
see which keys are near-cap; T-2107 wires `metadata.cv_key=$agent_id`
into heartbeats so `agent-presence` typically reports one entry per
agent. Near-cap on any other topic = producer bug.

See `substrate-broadcast-with-replay.md` (cv_index) and
`substrate-governor.md` (overflow telemetry).

---

## Recipe 5: Page on stuck claims (T-2072)

Fire when a topic enters `stuck` state — `expired_count > 0` OR
`oldest_active_age_ms > 60_000`. Indicates a worker crashed mid-task
and didn't release, OR claims are being acquired but not renewed.

**Notify script** (`/usr/local/bin/page-on-stuck-claim.sh`):

```sh
#!/bin/sh
# Fire only on transition INTO stuck (not stuck→still-stuck noise).
[ "${TERMLINK_CLAIM_NEW_STUCK:-false}" = "true" ] || exit 0
[ "${TERMLINK_CLAIM_OLD_STUCK:-false}" = "false" ] || exit 0

exec curl -sS -X POST \
  -H 'Content-Type: application/json' \
  -d "{\"text\":\":warning: topic ${TERMLINK_CLAIM_TOPIC} on ${TERMLINK_CLAIM_HUB} went stuck (active=${TERMLINK_CLAIM_ACTIVE_COUNT}, expired=${TERMLINK_CLAIM_EXPIRED_COUNT}, oldest=${TERMLINK_CLAIM_OLDEST_AGE_MS}ms) — run /claims ${TERMLINK_CLAIM_TOPIC} and consider claim-force-release\"}" \
  "$SLACK_WEBHOOK_URL"
```

**Cron file** (`/etc/cron.d/termlink-claims-watch`):

```cron
SHELL=/bin/bash
PATH=/usr/local/bin:/usr/bin:/bin
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/REPLACE_ME

@reboot root cd /opt/termlink && nohup setsid termlink channel claims-summary --all --watch 30 --notify /usr/local/bin/page-on-stuck-claim.sh --log ~/.termlink/claims.log > /var/log/termlink-claims-watch.log 2>&1 &
```

**Tuning:** "Stuck" is conservative (60s for oldest_age). Most legitimate
work units complete well under 60s. If you have long-running tasks, the
worker should be issuing periodic `renew` (substrate is biased toward
short work — see `substrate-claim-primitive.md`).

**Silence false positives:** Stuck-on-bootstrap is benign — when a
worker just started its first claim, it shows `oldest_age=0` and ramps
up. The transition gate (`OLD=false NEW=true`) handles this; only fires
once a topic flips.

See `substrate-claim-primitive.md`.

---

## Recipe 6: Page on queue backing up (T-2084)

Fire when this host's outbound queue has anything in it — hub blip
absorbed by the durable FIFO. Loud-on-blip pattern.

**Notify script** (`/usr/local/bin/page-on-queue-pending.sh`):

```sh
#!/bin/sh
[ "${TERMLINK_QUEUE_CHANGE_KIND:-}" = "pending" ] || exit 0

exec curl -sS -X POST \
  -H 'Content-Type: application/json' \
  -d "{\"text\":\":warning: $(hostname) outbound queue went pending (depth=${TERMLINK_QUEUE_NEW_PENDING}, oldest=${TERMLINK_QUEUE_OLDEST_AGE_MS}ms) — hub blip absorbed by FIFO. Check /governor + fleet doctor\"}" \
  "$SLACK_WEBHOOK_URL"
```

**Cron file** (`/etc/cron.d/termlink-queue-watch`):

```cron
SHELL=/bin/bash
PATH=/usr/local/bin:/usr/bin:/bin
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/REPLACE_ME

@reboot root cd /opt/termlink && nohup setsid termlink channel queue-status --watch 5 --notify /usr/local/bin/page-on-queue-pending.sh --log ~/.termlink/queue.log > /var/log/termlink-queue-watch.log 2>&1 &
```

**Tuning:** Watch interval here is tighter (5s) because queue state is
binary — operators usually want to know the moment the hub is
unreachable. Raise to 30s on hosts where Slack noise is a concern.

**Silence false positives:** Brief hub restarts (`hub restart`) cause a
1-2 second queue blip that drains immediately. Add a 5-second sleep +
re-check in the notify script if upgrade cycles cause noise:

```sh
#!/bin/sh
[ "${TERMLINK_QUEUE_CHANGE_KIND:-}" = "pending" ] || exit 0
sleep 5
# Re-check: still pending after 5s = real blip, not an upgrade
current=$(termlink channel queue-status --json 2>/dev/null | jq -r '.pending // 0')
[ "$current" -gt 0 ] || exit 0
exec curl -sS ... # as above
```

See `substrate-offline-queue-recipe.md`.

---

## Recipe 7: Dispatch on idle (T-2079)

Not a paging recipe — an orchestrator-pattern recipe. Fire a dispatch
job when a worker frees up. Skill-level shippable as well, but for
production where the orchestrator is its own process, the watch loop is
the right tier.

**Notify script** (`/usr/local/bin/dispatch-on-idle.sh`):

```sh
#!/bin/sh
# Fire only on NEW idle (worker just freed up), not REMOVED (gone offline).
[ "${TERMLINK_IDLE_CHANGE_KIND:-}" = "new" ] || exit 0

# Optional: gate on capability.
echo "$TERMLINK_IDLE_CAPABILITIES" | grep -qE '(^|,)rust(,|$)' || exit 0

# Dispatch the next available work unit to this worker.
# Example: pull from a redis-backed work queue, then DM:
unit_id=$(redis-cli LPOP work-queue:rust 2>/dev/null)
[ -n "$unit_id" ] || exit 0

exec termlink agent contact "$TERMLINK_IDLE_AGENT_ID" \
  --task "$unit_id" \
  --payload "claim and run unit $unit_id"
```

**Cron file** (`/etc/cron.d/termlink-dispatch-on-idle`):

```cron
SHELL=/bin/bash
PATH=/usr/local/bin:/usr/bin:/bin

@reboot root cd /opt/termlink && nohup setsid termlink agent find-idle --role claude-code --watch 30 --notify /usr/local/bin/dispatch-on-idle.sh --log ~/.termlink/find-idle.log > /var/log/termlink-find-idle-watch.log 2>&1 &
```

**What fires when:** A LIVE listener completes a claim and re-appears
as idle, OR a new worker comes online and advertises capabilities.

**Tuning:** The capability gate in the notify script is the operator's
policy. AND-multiple capabilities with multiple `grep -qE` lines if
needed.

See `agent-find-idle.md`.

---

## Combining the watches

The simplest production setup is six independent watches (one per recipe)
because notify is fire-and-forget and the watches don't block each other.
Total memory footprint is small (each watch is ~10MB). For very small
homelabs, combine into one composite watch via:

```cron
@reboot root cd /opt/termlink && nohup setsid termlink substrate status --watch 30 --notify /usr/local/bin/page-on-substrate-change.sh --log ~/.termlink/substrate.log > /var/log/termlink-substrate-watch.log 2>&1 &
```

The composite `substrate status --watch` exposes `TERMLINK_SUBSTRATE_FIELD`
+ `TERMLINK_SUBSTRATE_OLD` + `TERMLINK_SUBSTRATE_NEW` so the notify
script can branch on which sub-section changed. Trade-off: less
information per event (no `cap_hits_delta` for example, just a "field
changed" signal).

See `substrate-status.md` (SUBSTRATE-PULSE composition, T-2111..T-2117).

---

## Verifying the wiring

After installing the cron files + notify scripts, validate end-to-end:

```sh
# 1. Confirm cron picked up the file
systemctl restart cron
grep -r 'termlink' /etc/cron.d/

# 2. Confirm watch loops are running
ps auxf | grep -E 'termlink.*--watch'

# 3. Test a notify script manually
TERMLINK_GOV_HUB=test-hub TERMLINK_GOV_CAP_HITS_DELTA=1 TERMLINK_GOV_NEW_CAP_HITS=10 /usr/local/bin/page-on-cap-hits.sh
```

Expected: a single Slack post saying "test-hub cap_hits +1". If you see
nothing, the chmod bit was lost (PL-208) — `ls -l` the script.

To verify the audit-log paths are accumulating:

```sh
ls -la ~/.termlink/{rotation,heal,governor,claims,queue,find-idle,substrate}.log
```

A growing log file means the watch is alive and capturing transitions.
An empty log is the healthy state — the watch is running but nothing has
transitioned. A missing log file means the `--log` flag wasn't passed,
the path is wrong, or the watch hasn't run yet.

Forensic retrospective on any audit log:

```sh
# Last 50 governor events for one hub
jq -c 'select(.hub=="ring20-management")' ~/.termlink/governor.log | tail -50

# Or via the dedicated history verb
termlink fleet governor-history --hub ring20-management --since 7
```

Each `--log` flag has a matching `*-history` verb (T-2068 / T-2074 /
T-2081 / T-2086) that renders the same lines plus per-hub/topic
aggregate footers.

---

## Related

- `substrate-getting-started.md` — entry point; deploy preflight first
- `substrate-orchestrator-recipe.md` — master AEF integration walkthrough
- `substrate-tunables.md` — every `TERMLINK_*` env var
- `substrate-governor.md` — RPC behind Recipes 2/3/4
- `substrate-claim-primitive.md` — RPC behind Recipe 5
- `substrate-offline-queue-recipe.md` — local queue behind Recipe 6
- `agent-find-idle.md` — RPC behind Recipe 7
- `substrate-status.md` — composition behind composite watch
- T-2160 / `.context/cron/substrate-preflight-canary.crontab` — Recipe 1's git-tracked source
- PL-187 (verb-stack rung 7: ephemeral session integrators → durable cron monitors)
- PL-208 (chmod gap — every notify script must be `chmod +x`)
