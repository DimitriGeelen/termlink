# Substrate Orchestrator Recipe — AEF Integration Walkthrough

> **New to the substrate?** Read
> [`substrate-getting-started.md`](substrate-getting-started.md) (T-2149)
> first — it's a five-minute on-ramp that gets you through one claim
> lifecycle end-to-end before you tackle this longer walkthrough.
>
> **For:** Developers building the AEF (Agentic Engineering Framework) layer
> on top of the TermLink substrate. You want to coordinate N parallel
> workers on a shared host without machine-state conflicts, and you need to
> know *exactly* which substrate verbs make that work and how they compose.
>
> **Pairs with:** [`docs/architecture/parallel-execution-substrate.md`](../architecture/parallel-execution-substrate.md)
> (T-2018 ADR — the *why*) and the per-primitive operations docs below
> (the *what*). This doc is the *how*: the end-to-end pattern that
> combines every substrate primitive into a working orchestrator.

## Scope

TermLink ships eleven substrate primitives. This doc walks through the
*canonical work-stealing pattern* using the seven shipped ones, in the
exact order an AEF integration developer will reach for them:

| # | Primitive | Verb | Per-primitive doc |
|---|---|---|---|
| 2 | DISPATCH | `agent.find_idle` | [agent-find-idle.md](agent-find-idle.md) |
| 1 | CLAIM | `channel.claim` / `.renew` / `.release` | [substrate-claim-primitive.md](substrate-claim-primitive.md) |
| 3 | ASSIGN | `channel.claim_transfer` | [substrate-claim-primitive.md § "Hand a unit to a specific worker"](substrate-claim-primitive.md) |
| 5 | RESILIENCE | offline queue + `channel.post` dedupe | [substrate-offline-queue-recipe.md](substrate-offline-queue-recipe.md), [substrate-post-idempotency.md](substrate-post-idempotency.md) |
| 9 | BROADCAST-WITH-REPLAY | `metadata.cv_key` + `channel.cv_keys` | [substrate-broadcast-with-replay.md](substrate-broadcast-with-replay.md) |
| 10 | BACKPRESSURE | `hub.governor_status` | [substrate-governor.md](substrate-governor.md) |
| 11 | OBSERVABILITY | `substrate.status` / `.history` | (this doc + the ADR) |

Out of scope (deferred per ADR §6): #4 filesystem-write observation
(T-2022 DEFER), #6 symmetric auth (T-2024 DEFER), #7 hub-persistent
presence (T-2025 NO-GO — derived from durable heartbeats), #8 typed
agent-launch (T-2026/T-2090 DEFER).


> **Runnable proofs.** Three self-contained scripts exercise this exact pattern
> live against a local hub — run them before integrating to see the substrate
> behave:
>
> - `scripts/substrate-drain-demo.sh` — **work-stealing**: N workers race to
>   claim disjoint units of an M-unit queue; asserts exclusive delivery (every
>   unit won exactly once, zero double-claims). Evidence:
>   `docs/reports/T-2211-substrate-drain-demo.md`.
> - `scripts/substrate-cooperative-handoff-demo.sh` — **directed assignment**:
>   an orchestrator claims a slot and atomically hands the lease to a worker via
>   `claim-transfer`, which then renews and releases; asserts the full lifecycle
>   AND the `CLAIM_NOT_OWNED` ownership gate (7/7 — 3 positive + 3 refusals).
>   Evidence: `docs/reports/T-2212-substrate-cooperative-handoff-demo.md`.
> - `scripts/substrate-lease-expiry-demo.sh` — **worker-death resilience**: a
>   worker claims under a short lease then stops renewing (simulated crash); asserts
>   the slot auto-reopens to another worker AND the lapsed owner is locked out of
>   renew/release (6/6 — 3 positive + 3 refusals). The Antifragility path the other
>   two demos don't cover. Evidence:
>   `docs/reports/T-2214-substrate-lease-expiry-demo.md`.
>
> The "Canonical orchestrator pattern" and "Canonical worker pattern" sections
> below generalise what these two demos prove at minimal scale.

## Mental model

```
              ┌─────────────────────────────────────────┐
              │            Orchestrator (1)             │
              │  - discovers idle workers (find-idle)   │
              │  - reserves work (claim)                │
              │  - hands off atomically (claim-transfer)│
              │  - watches health (governor / queue)    │
              └────────────┬────────────────────────────┘
                           │ (substrate RPCs over TCP/HMAC)
              ┌────────────┴────────────────────────────┐
              │              TermLink hub               │
              │  - exclusive-delivery claims table      │
              │  - durable channel log + cv_index       │
              │  - per-process connection cap           │
              │  - per-sender rate limit + dedupe       │
              └────────────┬────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────▼────┐        ┌────▼────┐        ┌────▼────┐
   │worker-1 │        │worker-2 │        │worker-N │
   │heartbeat│        │heartbeat│        │heartbeat│
   │renew on │        │renew on │        │renew on │
   │long work│        │long work│        │long work│
   │release  │        │release  │        │release  │
   └─────────┘        └─────────┘        └─────────┘
```

Three actors, one shared substrate. The orchestrator and workers
never speak directly to each other — every interaction is mediated by
the hub. This is the [strict star](../architecture/parallel-execution-substrate.md)
invariant; honour it and the failure modes stay bounded.

**Three durable surfaces on the hub** carry every substrate signal:

1. **`agent-presence` topic** — workers heartbeat into this topic every
   ~30s with `metadata.cv_key=$agent_id` (substrate primitive #9 tagging).
   `find_idle` derives "LIVE workers minus those holding any claim".
2. **Work topic (operator-named, e.g. `work-queue`)** — posts represent
   "units of work to do". Claims are reserved on (topic, offset) pairs.
3. **Hub governor counters** — non-persisted: connection cap, rate-limit,
   dedupe, cv_index telemetry. Surfaced via `hub.governor_status`.

## The contract — which RPCs the AEF layer depends on

These are the substrate-side commitments. If your AEF integration uses
only these verbs, you stay inside the contract and the boundary is
clean (per ADR §9: *"contract the hard dependencies up front, then
build quiet"*).

### Read surface (Observe scope — no auth side-effects)

| RPC | What it answers | CLI | MCP |
|---|---|---|---|
| `agent.find_idle` | Which workers are LIVE and not holding any claim? | `termlink agent find-idle` | `termlink_agent_find_idle` |
| `channel.claims` | What's currently claimed on this topic? | `termlink channel claims` | `termlink_channel_claims` |
| `channel.claims_summary` | How busy / how stuck is this topic, in one O(1) call? | `termlink channel claims-summary` | `termlink_channel_claims_summary` |
| `channel.cv_keys` | Who's currently advertising on this topic (one entry per cv_key)? | `termlink channel cv-keys` | `termlink_channel_cv_keys` |
| `channel.queue_status` | Is my local outbound queue draining? | `termlink channel queue-status` | `termlink_channel_queue_status` |
| `hub.governor_status` | Is the hub at-capacity / rate-limiting / dedupe-absorbing? | `termlink fleet governor-status` | `termlink_fleet_governor_status` |
| `substrate.status` | All four substrate-read primitives in one envelope | `termlink substrate status` | `termlink_substrate_status` |
| `substrate.history` | Retrospective audit-trail walk | `termlink substrate history` | `termlink_substrate_history` |

### Write surface (Modify / Send scope — auth side-effects)

| RPC | What it does | CLI | MCP |
|---|---|---|---|
| `channel.post` | Post a unit of work (or a worker heartbeat) | `termlink channel post` | `termlink_channel_post` |
| `channel.claim` | Reserve (topic, offset) exclusively for the claimer | `termlink channel claim` | `termlink_channel_claim` |
| `channel.renew` | Extend a held claim's lease | `termlink channel renew` | `termlink_channel_renew` |
| `channel.release` | Release a claim — ack=true advances cursor, ack=false reopens | `termlink channel release` | `termlink_channel_release` |
| `channel.claim_transfer` | Atomic ownership transfer of an existing claim | `termlink channel claim-transfer` | `termlink_channel_claim_transfer` |

All write-side verbs are subject to the `(sender_id, client_msg_id)`
short-TTL LRU dedupe (substrate primitive #5 / T-2049) so a retried
post or claim during a hub blip does not double-apply.

### Daily slash skills (operator UX layer)

Mirror of the read surface above for interactive operator use. These
are user-facing skills wrapping the underlying CLI verbs:

`/find-idle`, `/claims`, `/queue-status`, `/governor`, `/cv-keys`,
`/substrate` (digest of all four read-side primitives), `/claim`,
`/release`, `/claim-transfer`, `/renew`. See the relevant per-primitive
doc for each.

## Canonical orchestrator pattern

The orchestrator's job is: *for each unit of work on the queue, hand
it to exactly one idle worker, atomically, with no race window*.

**Ready-to-adapt script (T-2148).** `scripts/substrate-orchestrator-
loop.sh` ships a vetted one-orchestrator harness that wires the
canonical dispatch lifecycle for you: subscribe-stream(work-topic)
→ find-idle(capability) backoff → claim → claim-transfer to worker
→ fire-and-forget DM. If `claim-transfer` fails mid-step (worker became
busy), the orchestrator releases the orphaned claim so the slot
reopens — no leak. SIGTERM/SIGINT releases any in-flight claim and
exits 130. Pair with `scripts/substrate-worker-loop.sh` (T-2146) on
the receive side. Sample dispatch run:

```bash
scripts/substrate-orchestrator-loop.sh --work-topic aef:deploy \
    --capability deploy \
    --ttl-ms 60000 --idle-poll-ms 5000
```

The two scripts together implement the full work-stealing pattern
end-to-end. Both are intentionally adaptable — read the script source
for the full inline contract. The hand-rolled loop below shows the
same wiring without the harness for cases where the dispatch logic
is also yours.

Five-step canonical loop, in shell form (the AEF integration will
typically use the MCP variants of the same verbs):

The canonical loop is **stream-based**: subscribe to the work topic,
and as each envelope arrives, claim it and hand it to an idle worker.
`channel subscribe --resume` advances a persisted cursor so the
orchestrator processes each envelope exactly once across restarts —
the substrate gives us the "what's new" view for free, so the
orchestrator never has to derive a "next free offset" itself.

```bash
#!/usr/bin/env bash
# Canonical work-stealing orchestrator
set -euo pipefail

TOPIC="work-queue"
ORCHESTRATOR_ID="$(jq -r .agent_id ~/.termlink/be-reachable.state)"

# Stream every envelope on the work topic as it arrives. `--resume`
# advances a persisted cursor so restarts pick up where we left off.
termlink channel subscribe "$TOPIC" --resume --json | while read -r envelope; do
  offset="$(echo "$envelope" | jq -r '.offset')"

  # Step 1 — wait for an idle worker with the right capability
  worker=""
  while [ -z "$worker" ]; do
    worker="$(termlink agent find-idle --capability deploy --json \
      | jq -r '.idle[0].agent_id // empty')"
    [ -z "$worker" ] && sleep 5             # nobody free, back off
  done

  # Step 2 — orchestrator claims this offset first.
  # CLAIM_CONFLICT (-32015) means another orchestrator beat us OR
  # this offset was already completed in a prior run. Either way: skip.
  claim_id="$(termlink channel claim "$TOPIC" "$offset" \
    --claimer "$ORCHESTRATOR_ID" --ttl-ms 60000 --json 2>/dev/null \
    | jq -r '.claim_id // empty')"
  [ -n "$claim_id" ] || continue            # already-claimed or completed

  # Step 3 — atomically transfer ownership to the worker
  #          (no release-then-reclaim race window — T-2046)
  termlink channel claim-transfer \
    --claim-id "$claim_id" \
    --to-owner "$worker" \
    --by "$ORCHESTRATOR_ID" \
    --reason "orchestrator dispatch" >/dev/null

  # Step 4 — notify the worker via a doorbell DM so it picks up
  termlink agent contact "$worker" \
    --payload "claim=$claim_id topic=$TOPIC offset=$offset"
done
```

Why this pattern is correct:

- **Subscribe-based loop avoids "what's been done" bookkeeping.**
  `channel subscribe --resume` is the substrate's canonical
  "process each envelope once" primitive. We never need to compute
  a derived "next free offset" because the stream IS the offset
  sequence in order. Restarting the orchestrator resumes from the
  persisted cursor.
- **Step 1 is read-only.** A race between two orchestrators picking
  the same idle worker is benign — Step 2's claim attempt is the
  serialization point.
- **Step 2 fails loudly on conflict.** If another orchestrator
  already claimed this offset (or it was completed in a prior run
  via `release --ack`), `channel.claim` returns `CLAIM_CONFLICT`
  (-32015) and we skip. No silent double-assignment.
- **Step 3 is atomic by hub guarantee.** `claim_transfer` moves
  ownership inside the hub with zero gap — there is no moment where
  the slot is "free" between orchestrator and worker. This is the
  T-2046 contract; eliminates the race window that release-then-reclaim
  would expose.
- **Step 4 is fire-and-forget.** If the doorbell fails, the worker
  still discovers the claim via its `agent.inbox` poll. The hub's
  durable claims table is the source of truth, not the doorbell.

## Canonical worker pattern

The worker's job is: *honour assigned work atomically; release with
ack on completion, ack=false on retryable failure, let `Drop` handle
crashes*.

**Ready-to-adapt script (T-2146).** `scripts/substrate-worker-loop.sh`
ships a vetted one-unit worker harness that wires the canonical
lifecycle for you: claim → background auto-renew → run a user-supplied
worker command → release (`--ack` on cmd exit 0, no `--ack` on non-zero
or signal). SIGTERM/SIGINT during work releases-without-ack and exits
130 so no claim leaks. Pass the topic, offset, and worker command:

```bash
scripts/substrate-worker-loop.sh --topic work-queue --offset 42 \
    --cmd 'python3 /opt/myapp/process.py 42' \
    --ttl-ms 60000 --renew-every-ms 20000
```

The worker sees `TERMLINK_CLAIM_ID`, `TERMLINK_CLAIM_TOPIC`,
`TERMLINK_CLAIM_OFFSET`, `TERMLINK_CLAIMER` in its env. Read the
script source for the full inline contract — it's the "hello world"
for substrate users.

**Adopted-claim mode (T-2150).** When pairing with
`scripts/substrate-orchestrator-loop.sh` (T-2148), the orchestrator
already does `channel claim` + `claim-transfer` before DMing the worker.
A second `channel claim` from the worker would fail with
`CLAIM_CONFLICT` (the worker IS the current holder via transfer).
Pass `--claim-id <existing>` to skip the self-claim step and adopt
the pre-transferred claim — auto-renew + work + release proceed
identically. Use this form in worker-side pickup loops:

```bash
# Worker pickup logic: parse claim_id/topic/offset from the
# orchestrator's DM payload, then delegate everything else to the
# vetted lifecycle. No inline renew loop, no inline release branching.
scripts/substrate-worker-loop.sh \
    --topic "$topic" --offset "$offset" \
    --claim-id "$claim_id" \
    --cmd 'python3 /opt/myapp/process.py'
```

**Ready-to-adapt pickup script (T-2152).**
`scripts/substrate-worker-pickup.sh` ships a vetted long-running
inbox-poll loop that closes the orchestrator+worker pair: it polls
`agent inbox` for unread `dm:*` topics, decodes the orchestrator's
`payload_b64`, parses `claim=X topic=Y offset=Z`, and spawns
`substrate-worker-loop.sh --claim-id X --topic Y --offset Z` per
dispatch. Operator passes one `--cmd` template; everything else is
wired:

```bash
scripts/substrate-worker-pickup.sh \
    --worker-id deploy-worker-a \
    --cmd 'python3 /opt/myapp/run.py \
           --topic "$TERMLINK_CLAIM_TOPIC" \
           --offset "$TERMLINK_CLAIM_OFFSET"'
```

`--max-claims N` for bounded smoke runs. `--test-parse 'PAYLOAD'`
prints the parsed claim/topic/offset without hub contact — useful
for verifying orchestrator-DM-format compatibility before deploying.
SIGTERM/SIGINT exits 130 with the in-flight worker killed cleanly.

Use the script. The hand-rolled inline loop below documents the same
wiring for cases where you need to customise (different DM payload
format, additional ownership checks, custom logging).

```bash
#!/usr/bin/env bash
# Canonical work-stealing worker
set -euo pipefail

WORKER_ID="$(jq -r .agent_id ~/.termlink/be-reachable.state)"
RENEW_INTERVAL_S=20                       # claim TTL is 30s by default

# Step 1 — heartbeat into agent-presence so find_idle can see us
bash scripts/be-reachable.sh start --capabilities deploy &

while :; do
  # Step 2 — poll the inbox for an unread DM topic (any dm:* with unread > 0).
  # `agent inbox --watch` is `--json`-incompatible (T-1558), so this is a
  # one-shot poll inside a loop with a short sleep to keep rate-limit cost low.
  # NOTE (T-2153): `agent inbox --json` returns an ARRAY of inbox entries
  # `[{topic, cursor, latest, unread}, ...]`, not an object with a `.topics`
  # field. The jq selector is `.[]?`, not `.topics[]?`. The vetted version
  # of this whole loop ships as `scripts/substrate-worker-pickup.sh`
  # (T-2152) — prefer that script in production.
  next_dm="$(termlink agent inbox --json \
    | jq -r '.[]? | select(.unread > 0 and (.topic | startswith("dm:"))) | .topic' \
    | head -1)"
  if [ -z "$next_dm" ]; then
    sleep 2
    continue
  fi

  # Step 3 — read the next unread envelope from that DM topic; --resume
  # advances the persisted cursor so we don't re-read it next iteration.
  # NOTE (T-2153): envelope payload is base64-encoded in `.payload_b64`,
  # NOT in a `.payload` field (T-1427 envelope canon). Decode before parse.
  dm_payload="$(termlink channel subscribe "$next_dm" \
    --resume --limit 1 --json \
    | jq -r '.payload_b64 // empty' | base64 -d 2>/dev/null)"
  [ -n "$dm_payload" ] || continue

  # Step 3b — extract claim_id, topic, offset from the payload
  claim_id="$(echo "$dm_payload" | grep -oP 'claim=\K\S+')"
  topic="$(echo "$dm_payload" | grep -oP 'topic=\K\S+')"
  offset="$(echo "$dm_payload" | grep -oP 'offset=\K\S+')"

  # Step 4 — verify we own the claim (defensive — orchestrator already transferred).
  # The channel.claims JSON envelope uses `.claimer` (see
  # crates/termlink-bus/src/claim.rs::ClaimInfo); NOT `.claimed_by`.
  current_owner="$(termlink channel claims "$topic" --json \
    | jq -r ".claims[] | select(.claim_id==\"$claim_id\") | .claimer")"
  if [ "$current_owner" != "$WORKER_ID" ]; then
    continue                              # transfer failed or stale DM
  fi

  # Step 5 — do the work; renew lease in background while busy
  (
    while :; do
      sleep "$RENEW_INTERVAL_S"
      termlink channel renew --claim-id "$claim_id" \
        --claimer "$WORKER_ID" --by-ms 30000 >/dev/null 2>&1 || break
    done
  ) &
  RENEW_PID=$!

  if do_unit_of_work "$topic" "$offset"; then
    # Step 6a — happy path: release with ack=true (cursor advances)
    kill "$RENEW_PID" 2>/dev/null || true
    termlink channel release --claim-id "$claim_id" \
      --claimer "$WORKER_ID" --ack
  else
    # Step 6b — retryable failure: release ack=false (slot reopens)
    kill "$RENEW_PID" 2>/dev/null || true
    termlink channel release --claim-id "$claim_id" \
      --claimer "$WORKER_ID"               # no --ack: ack=false (default for /release --retry)
  fi
done
```

Why this pattern is correct:

- **Step 1 advertises liveness.** Without `be-reachable.sh`, the worker
  is invisible to `find_idle`. The `--capabilities` tag is what lets
  the orchestrator pick the right worker for the unit.
- **Step 4 is defence-in-depth.** Orchestrator transfer succeeded
  before sending the DM (Step 5 of orchestrator), so this check is
  expected to pass. It catches the rare case where the DM arrived but
  the transfer rolled back, or the DM is from a previous lapsed claim.
- **Step 5 (background renewal) is mandatory for long work.** Default
  claim TTL is 30s. Without renewal, a worker doing 5min of work loses
  the slot at 30s and another worker reopens it. The renew loop
  refreshes at 20s (`RENEW_INTERVAL_S`), well before lapse.
- **Step 6a vs 6b is the cursor pivot.** `--ack` advances the persisted
  cursor past this offset — the unit is done. Omitting `--ack` (or
  passing `--retry` to the `/release` skill) reopens the slot to the
  next worker.
- **Crash handling is implicit.** If the worker process dies between
  Step 5 and Step 6, the renew loop dies with it. The hub will
  lazy-evict the lapsed claim on the next claim attempt and the slot
  reopens. No explicit "tombstone" required.

## Production deployment via systemd templates

The shell loops above are the canonical wiring. For production —
where you want the orchestrator + workers to survive reboots, log to
the journal, and refuse to start on a misconfigured host — wrap both
sides in the shipped systemd templates instead of writing your own
unit files:

| Side | Template | Wraps | Identity from |
|---|---|---|---|
| Dispatch | `systemd-templates/termlink-substrate-orchestrator@.service` (T-2165) | `scripts/substrate-orchestrator-loop.sh` (T-2148) | `%i` = orchestrator-id |
| Pickup | `systemd-templates/termlink-substrate-worker@.service` (T-2167) | `scripts/substrate-worker-pickup.sh` (T-2152) | `%i` = worker-id |

Both templates use `EnvironmentFile=/etc/termlink/substrate-{orchestrator,worker}/%i.env`
for per-instance config, default `TERMLINK_RUNTIME_DIR=/var/lib/termlink`
(load-bearing per PL-021), translate env vars → CLI flags inline, ship
matching hardening (NoNewPrivileges, PrivateTmp, ProtectSystem=strict),
and log to the journal under `SyslogIdentifier=termlink-substrate-{role}-%i`.

One-host install:

```bash
# Dispatch (one per topic per host)
sudo install -d /etc/termlink/substrate-orchestrator
sudo cp /opt/termlink/systemd-templates/termlink-substrate-orchestrator@.service \
        /etc/systemd/system/
# Author /etc/termlink/substrate-orchestrator/my-orch.env with TERMLINK_SO_WORK_TOPIC=…
sudo systemctl daemon-reload
sudo systemctl enable --now termlink-substrate-orchestrator@my-orch.service

# Pickup (one per worker per host; %i becomes the substrate worker-id)
sudo install -d /etc/termlink/substrate-worker
sudo cp /opt/termlink/systemd-templates/termlink-substrate-worker@.service \
        /etc/systemd/system/
# Author /etc/termlink/substrate-worker/deploy-worker-a.env with TERMLINK_SW_CMD=…
sudo systemctl daemon-reload
sudo systemctl enable --now termlink-substrate-worker@deploy-worker-a.service
```

Multiple worker instances per host: same template, different `.service`
filename — each appears on `agent-presence` under its own worker-id and
receives dispatch DMs independently.

**Restart=on-failure + exit-4 contract.** All three long-running
substrate scripts (orchestrator-loop, worker-pickup, worker-loop) run
`scripts/substrate-preflight.sh` at startup (T-2163/T-2166): silent on
PASS, warn-and-continue on WARN, **refuse to start with exit 4** on
FAIL. The templates ship `Restart=on-failure + RestartSec=10s` for
exactly this reason — a misconfigured host (volatile `/tmp` runtime_dir,
missing `hubs.toml`, dead `be-reachable`) produces a visible restart-loop
in `systemctl status` / `journalctl -u`, instead of a silently-wedging
service that fails per envelope arrival.

Full install walkthrough, env-file templates, troubleshooting table, and
the three worker-side patterns (DM-driven via template, external-queue
hand-roll, short-lived CI): `docs/operations/substrate-systemd.md`.

## Failure modes and recovery

| Symptom | Diagnosis | Recovery |
|---|---|---|
| `CLAIM_CONFLICT` (-32015) on Step 3 of orchestrator | Another orchestrator claimed first | Re-loop; try next offset |
| `CLAIM_NOT_OWNED` (-32017) on `claim_transfer` | `by` argument doesn't match current holder | Re-read `channel.claims`; orchestrator's claim may have lapsed before transfer fired |
| `CLAIM_NOT_FOUND` (-32016) on `renew` | Claim already released or lapsed | Worker must re-acquire; renewal cadence too slow |
| `CLAIM_EXPIRED` (-32018) — but in practice `CLAIM_NOT_FOUND` (-32016) — on `renew`/`release` | Lease expired before renewal (worker stopped renewing / crashed) — the substrate-correctness footgun. The protocol reserves -32018 `CLAIM_EXPIRED`, but a naturally-lapsed claim is **lazy-evicted**, so the operator-observable error is the -32016 *"not found (never existed, released, or expired)"* message. **Grep logs for "not found", not a literal "lapsed"/"expired" string** (no such "lapsed" code exists) | Re-claim under a **new** claim_id (the old row is gone — `renew` cannot resurrect it). Tune renewal cadence < 50% of TTL. Proven live by `scripts/substrate-lease-expiry-demo.sh` (T-2214) |
| `RATE_LIMITED` (-32008) | Worker is posting too fast | Back off; check `/governor` for `rate_hits_total` growth |
| `HUB_AT_CAPACITY` (-32019) | Hub's connection pool exhausted | Back off; check `/governor` for `capacity_hits_total` growth |
| Worker's `channel.post` queued instead of delivered | Hub blip; outbound queue is buffering (#5 RESILIENCE) | Wait — the flush task drains every 5s once hub returns. Check `/queue-status` |
| `cv_overflow > 0` on `governor.status` | A producer is mis-emitting `cv_key` (e.g. timestamp instead of stable id) saturating per-topic cap | Run `/cv-keys <suspect-topic>` to identify; fix the producer |
| `find_idle` returns empty but operator sees LIVE workers | All workers hold at least one active claim | Wait, or expand worker pool |
| Worker DM never arrives after `claim_transfer` succeeds | Doorbell delivery is best-effort; persistent `dm:*` topic carries the canonical message | Worker's `agent.dms` poll catches it on next iteration |
| Orchestrator restarts mid-flight | Outstanding orchestrator-held claims lapse after TTL; in-flight transfers to workers persist | Restart cleanly; no orchestrator-side state to restore (claims are hub-side) |
| Hub restart | Claims table durable (SQLite); cv_index in-memory only (re-populates within one heartbeat cycle); outbound queues durable | All clients reconnect; lease times survive; cv_index converges within ~30s |
| `fleet governor-status` returns `-32001 / Missing 'target' in params` for a hub | That hub runs a pre-T-2048 binary (older than `hub.governor_status`); its unknown-method dispatch misroutes to `event.emit_to` | Upgrade the hub binary via `scripts/fleet-deploy-binary.sh --probe --target <hub>`; see `substrate-governor.md` § "Version-skew diagnosis" (T-2138). **Preemptive catch:** `scripts/substrate-preflight.sh` Check 5 (T-2184) probes `hub.governor_status` for the `rate_buckets_evicted_total` field on every cron run — absence WARNs at deploy time, before the symptom hits an operator |
| `rate_buckets_active` grows monotonically without bound | Hub runs a pre-T-2137 binary; rate-bucket eviction never wired into startup; HashMap accumulates one entry per distinct sender_id ever seen | Upgrade the hub binary; see `substrate-governor.md` § "Reading rate_buckets_active" (T-2138). Until upgrade, hub memory grows ~120 bytes per distinct sender. **Preemptive catch:** same as the row above — `scripts/substrate-preflight.sh` Check 5 (T-2184) probes for the post-T-2139 `rate_buckets_evicted_total` field; absence covers BOTH the pre-T-2048 misroute class and the pre-T-2137 eviction-loop class with one signal |

## Deployment & identity troubleshooting

The table above covers errors a *running* substrate emits. This second
table covers the class that hits earlier — before the orchestrator or
worker is able to talk to the hub at all, or that hits at identity-binding
time. The pattern: orchestrator + worker pattern itself is sound, but
something at the host / fleet / identity layer refuses or silently
no-ops. Symptoms here are deceptively quiet — the operator sees auth
mismatches, refused posts, or "nothing happens", not a clean error from
the substrate primitives.

**Fast path: before debugging, run `scripts/substrate-preflight.sh`
(T-2154).** It catches the top-of-list deployment-time symptoms below
with one command and a remediation hint per failure.

| Symptom | Diagnosis | Recovery |
|---|---|---|
| **Hub regenerates secret + TLS cert every reboot** — `fleet doctor` reports auth-mismatch after each restart; `fleet verify` reports TOFU drift on a fresh cert fingerprint | `TERMLINK_RUNTIME_DIR` lands on volatile `/tmp` (tmpfs mount OR `systemd-tmpfiles` D-rule wipe). Persistent state never survives boot — hub regenerates per cold-start (**PL-021**). Recurring: 4× cascade T-1294 (ring20-management) + T-1296 (ring20-dashboard) | Run `scripts/substrate-preflight.sh` to confirm. Fix: migrate runtime_dir off `/tmp`. Set `Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink` in systemd unit (or watchdog script). Pre-seed with `cp -a /tmp/termlink-0/. /var/lib/termlink/` before restart so persist-if-present preserves. See CLAUDE.md § Hub Auth Rotation Protocol → Special case — volatile runtime_dir |
| **Pickup envelopes never appear on `framework:pickup` topic** — `fw pickup process` exits 0 + creates local task, but `termlink channel info framework:pickup` count doesn't increment | `.agentic-framework/lib/pickup-channel-bridge.sh` not executable. `pickup_process_one` invokes it with `[ -x "$bridge" ] && "$bridge" "$envelope"` — non-executable evaluates false, silently skips (**G-061**). Sibling of T-2052 install-time chmod gap | `chmod +x .agentic-framework/lib/pickup-channel-bridge.sh`. Re-fire the bridge manually on the unposted envelope: `FRAMEWORK_ROOT=… PROJECT_ROOT=… .agentic-framework/lib/pickup-channel-bridge.sh .context/pickup/processed/P-NNN-*.yaml`. Check `.context/working/.pickup-bridge.log` for the success line. Long-term: `fw doctor` should add a `check-bridge-executable` lint |
| **`channel post` returns `-32014 sender_id does not match identity fingerprint`** | T-1427 strict identity binding: `--sender-id` is validated against the local signing key's fingerprint. The substrate cannot be tricked into posting under a different sender's identity on a single host (the fingerprint is derived from the local hub.secret, not the operator's claim) | Don't pass `--sender-id` at all (auto-resolves to the verified identity), OR start a separate process with a different identity (separate hub.secret / runtime_dir). For multi-identity smoke tests, use distinct `~/.termlink-secondary/` runtime dirs |
| **`channel claim-transfer` succeeds but `release --claim-id … --claimer X` returns `CLAIM_NOT_OWNED`** | The claimer LABEL stored at the hub after transfer (`X`) was different from the label the release process passed. T-1857 auto-resolution chain pulls from `$TERMLINK_AGENT_ID` env or `~/.termlink/be-reachable.state` — if these were set by a different orchestrator/skill earlier in the session, they take precedence over what the operator typed | Pass `--claimer X` explicitly on release, matching the transfer's `--to-owner X`. Verify via `/claims <topic>` — the `claimer` column is authoritative. The substrate-worker-pickup.sh script does this correctly (forwards `--claimer "$WORKER_ID"` to worker-loop); ad-hoc CLI use is where it slips |
| **`fleet reauth --bootstrap-from auto` fails with anchor-fetch error** — `ssh: permission denied` / `file not found` / "secret got truncated" | The declared `bootstrap_from` channel on this profile in `hubs.toml` (T-1291) is misconfigured. The substrate's auto-heal arc (T-1680) gated on this declaration but the gate validates declaration syntax, not connectivity | Run `fw fleet bootstrap-check <profile>` (T-1688) — it does the same fetch the live heal does, but read-only and labeled `ok` / `no-anchor` / `fetch-fail` / `invalid-format`. Fix the underlying anchor (correct ssh key, fix path, restore file permissions); re-run `bootstrap-check` until `ok`. Then `fleet reauth --bootstrap-from auto` will succeed |
| **`/be-reachable` reports session is registered, but `agent find-idle` doesn't show this worker** — the agent presents on `agent-presence` topic but isn't returned by the discovery verb | Three sub-cases: (a) heartbeat hasn't propagated yet (default 30s interval — wait one cycle); (b) `~/.termlink/be-reachable.state` points at a dead PID — the listener process died after registering (substrate-preflight check 3 catches this); (c) the worker already holds a claim — `find-idle` does the LIVE-presence MINUS active-claimers anti-join (T-2019 / T-2020) | (a) Wait 30s. (b) `scripts/substrate-preflight.sh` → fix as instructed (`/be-reachable stop && /be-reachable start`). (c) Run `/claims --all` to confirm the worker holds a claim — if so, find-idle correctly excluded it. Wait for release or expand the worker pool |
| **Pickup loop runs but never processes a DM** — `substrate-worker-pickup.sh` is alive (`ps`), log shows `polling inbox for dm:* dispatches`, but envelopes posted to the expected `dm:` topic never trigger spawn | The DM payload format doesn't match `parse_dispatch`'s regex (`claim=X topic=Y offset=Z` literally, space-separated, in any order). Real-world hits: payload is multi-line; payload has extra preamble; orchestrator emitted a different schema | Run `substrate-worker-pickup.sh --test-parse "<the actual payload>"` (T-2152) — it'll print which fields parsed and which didn't, without touching the hub. Fix the orchestrator's emit format OR the pickup parser. Once fixed, the live pickup will process the next matching DM |
| **`/agent-handoff peer T-XXX "..."` reports `delivered=0` despite peer being LIVE** | Either (a) self identity_fingerprint resolution failed silently (the recipient topic name `dm:<self>:<peer>` was constructed with empty `<self>`); (b) peer's `dm:` topic doesn't exist yet on the local hub (channel federation gap — G-060); (c) peer is on a different hub | (a) `termlink whoami --json` — check `session.identity_fingerprint`. Fix via `/be-reachable start` to register identity. (b) `termlink channel create dm:<self>:<peer>` to materialise. (c) Use `--hub <addr>` or `/agent-handoff --fleet` to fan to all hubs. See PL-195 / T-1693 for the shared-host identity edge case |

## Observability hooks

The substrate ships seven read-only diagnostic verbs operators can use
to monitor a running orchestrator + workers system. The `/substrate`
digest skill composes the four most-relevant ones into a single
cold-start snapshot.

| Question | Verb | Surface |
|---|---|---|
| "Who's free to take work?" | `/find-idle` | Skill / CLI / MCP |
| "What's claimed right now?" | `/claims --all` | Skill / CLI / MCP |
| "Is anything stuck?" | `/claims --all --only-stuck` | Skill / CLI / MCP |
| "Is my queue draining?" | `/queue-status` | Skill / CLI / MCP |
| "Any hub backpressure?" | `/governor --only-pressured` | Skill / CLI / MCP |
| "Which cv_keys advertise on this topic?" | `/cv-keys <topic>` | Skill / CLI / MCP |
| "All four read surfaces at once" | `/substrate` | Skill / CLI / MCP |

Continuous monitoring uses the CLI-tier `--watch` + `--notify` + `--log`
forms (deliberately not surfaced as skills — long-running loops sit
awkwardly inside slash commands). Forensic retrospective uses the
`*-history` verbs reading the NDJSON audit logs.

End-to-end real-time-alerting recipe (operator wires once, leaves
running):

```bash
# Continuous monitor with paging + audit trail on every substrate event
termlink substrate status --watch 30 \
  --notify /usr/local/bin/page-on-substrate-event.sh \
  --log ~/.termlink/substrate.log
```

The notify script receives one event per per-section transition
(dispatch zero-to-nonzero, queue drained-to-pending, governor counter
increments, etc.) and decides per-event whether to page. Event schema:
see [`substrate-governor.md` § "page-on-cv-overflow.sh recipe"](substrate-governor.md)
for the template; substrate-status follows the same shape.

## Cross-hub limits — G-060

**TermLink hubs maintain independent substrate state.** Claims on
hub A and claims on hub B are unrelated rows. cv_index, governor
counters, claims tables, dedupe LRUs — all per-hub.

This means:

- **One orchestrator per hub.** Two orchestrators on different hubs
  cannot coordinate via the substrate alone. They will both happily
  hand the same logical work unit to different workers on their
  respective hubs.
- **Workers belong to one hub.** A worker holding a claim on hub A
  cannot satisfy a claim attempt for the same offset on hub B.
- **find_idle is per-hub.** `/find-idle` returns only the local hub's
  idle workers by ADR §6 #2 design (hub-derived state, no federation).

For fleet-wide work distribution, the AEF layer must implement
its own cross-hub federation (sharding by work-key hash → hub, leader
election per hub, etc.). The substrate provides the building blocks
within one hub; the cross-hub policy is the AEF layer's responsibility.

See [`channel-topic-semantics.md`](channel-topic-semantics.md) for the
full G-060 discussion and why this is the right architectural cut.

## AEF integration checklist

When wiring an AEF integration against this substrate:

- [ ] **Pick a work topic name** and document it in the AEF spec.
      Conventionally `aef:work-<purpose>` so it's namespaced (e.g.
      `aef:work-deploy`, `aef:work-test`).
- [ ] **Workers self-advertise** by running `be-reachable.sh start
      --capabilities <comma-csv>` at startup. The capability tags are
      what the orchestrator filters on in `find_idle`.
- [ ] **Orchestrator runs as exactly one process per hub.** Use a
      coarse external lock (cron-anchored, systemd unit, leader-election
      via `claim` on a sentinel topic) to prevent multi-orchestrator
      racing.
- [ ] **Choose claim TTL deliberately.** Default 30s is good for
      sub-minute work. For 10min units, set TTL=120s and renew at 60s.
      Don't push TTL to the hub max (1h) — long TTLs mask crashed workers.
- [ ] **Always wire `--ack` on `release` for completed work.** Forgetting
      the flag silently treats every release as retry, and the orchestrator
      will re-dispatch already-completed units to the next worker.
- [ ] **Run `/substrate` as a cold-start health check** at the top of
      every AEF orchestrator session. Confirms substrate is healthy
      before any work is dispatched.
- [ ] **Monitor `governor.cv_overflow_total`.** Non-zero means a
      producer (probably an AEF worker) is mis-emitting `cv_key`.
      Wire `page-on-cv-overflow.sh` to catch it the moment a misconfig
      ships.
- [ ] **Persist orchestrator identity** in `~/.termlink/be-reachable.state`
      so `--by` arguments resolve consistently across restarts. Without
      this, every restart appears as a different identity and existing
      orchestrator-held claims become unreleasable.
- [ ] **Honour the substrate dedupe contract.** Pass `--client-msg-id`
      on `channel.post` for any post that might be retried (T-2049).
      The CLI mints a random 128-bit id by default; this is correct.
- [ ] **Audit logs live in `~/.termlink/`.** rotation.log, heal.log,
      governor.log, claims.log, queue.log, find-idle.log, substrate.log
      are append-only NDJSON. Operators rotate them with logrotate;
      AEF integrations should not touch them (they're operator-facing).

## Recommended retention settings

The hub's `channel.create` RPC accepts a `retention` policy per topic. The
default if unspecified is `Retention::Forever` — *never trim*. For most
substrate topics this is wrong: high-rate broadcast/heartbeat patterns
will grow unboundedly and eventually wedge subscribers (precedent:
**T-1991** — `agent-presence` bloated to ~1800 envelopes in production
before subscribe-path slowdown was noticed).

The hub fires a `tracing::warn!` at create-time when retention is
`Forever` AND the topic name matches one of these high-rate patterns
(**T-2058**): `agent-presence`, `agent-chat-arc`, `agent-listeners-*`,
`agent-conv-*`, `dm:*`. Operators don't always see hub logs, so set
retention explicitly when wiring an AEF integration — don't rely on the
warn to remind you.

The CLI's `--ensure-topic` flag (`termlink channel post --ensure-topic …`)
currently uses `Retention::Forever` on auto-create. Until that helper is
upgraded to pick high-rate defaults, **prefer `channel.create` first**
with an explicit retention rather than `--ensure-topic`.

### Per-topic-pattern recommendations

| Topic / pattern | Recommended retention | Why |
|---|---|---|
| `agent-presence` | `Messages(1000)` | High-rate heartbeat producer (one envelope per worker per ~30s). T-1991 vector. Latest-per-cv_key is what subscribers want; cv_index (substrate #9) closes the lookup cost — retention just bounds the durable log. 1000 = ~10h of fleet history at 5 LIVE agents. |
| `agent-chat-arc` | `Messages(2000)` | Fleet-wide broadcast topic. Slower than agent-presence per-envelope but every agent posts, so still bounded. 2000 ≈ ~1-2 weeks of fleet chat at typical homelab rate. |
| `agent-listeners-*` | `Messages(500)` | Per-host listener registry rollups. One row per heartbeat per host. |
| `agent-conv-*` | `Messages(500)` | Conversation thread state. Bounded by thread length. |
| `dm:*` | `Messages(1000)` | DM topics. Two-party conversations; growth bounded by activity but ack-acknowledgement reads from history, so don't trim too aggressively. |
| **Work topics** (e.g. `aef:work-deploy`) | `Messages(10000)` | Posts represent units of work. Workers consume via claims; old completed work needs to stick around long enough to be replayed if needed. Tune per fleet throughput. |
| **Audit topics** (e.g. `audit:*`, `routing:lint`) | `Messages(1000)` to `Days(30)` | Pick `Days(N)` for compliance windows, `Messages(N)` for capacity bounds. The hub's `routing:lint` topic ships at `Messages(1000)` by default. |
| **Channel-1 / framework / pickup** | `Forever` | These are durable audit logs of bounded growth (one envelope per task event, not per-second). `Forever` is correct here. |
| **Single-value-per-topic broadcast** (e.g. `state:<key>`, `config:<key>`, presence summaries) | `Latest` | New in T-2142. Keeps exactly one envelope — the most recent. Right answer when the topic's name IS the key and only the freshest value matters. Durable counterpart to cv_index (T-2103) for the "last-write-wins on this topic" pattern. Use when subscribers only ever ask "what's the current value?" and never walk history. |
| **One-off / debug** | `Messages(100)` | Anything operator-created interactively for debugging. Cap small. |

### When to pick `Latest`

`Retention::Latest` (T-2142) is the durable-storage counterpart to the
in-memory cv_index (T-2103, substrate #9). They solve the *same problem*
at different layers:

- **cv_index** — fast O(K) current-value-per-`metadata.cv_key` lookups
  via `channel.subscribe --include-current-value` or `channel.cv_keys`.
  In-memory, per-hub, cleared on restart, repopulated within one
  heartbeat cycle. Use when a single topic carries values for *many*
  keys (e.g. `agent-presence` — one entry per agent_id).
- **`Retention::Latest`** — durable storage that always keeps exactly
  one envelope. Use when the topic *itself* names the key — e.g.
  `state:deploy-mode` or `config:rate-limit-per-sec`. The subscriber
  doesn't pick a cv_key; the topic name IS the key. Restart-safe by
  construction (the SQLite log keeps the single envelope across hub
  restart, unlike cv_index).

Use both together when you want `agent-presence`-style "many keys on
one topic" semantics with restart-safe durable state: pair `cv_key`
metadata on posts with `Retention::Messages(N)` (NOT `Latest` — the
log must hold one envelope *per key*, not just one envelope total).

### Concrete examples

Create `agent-presence` with the recommended retention before starting
any heartbeat producer:

```bash
termlink channel create --name agent-presence \
  --retention messages --retention-value 1000
```

Create a work topic for an AEF integration:

```bash
termlink channel create --name aef:work-deploy \
  --retention messages --retention-value 10000
```

Create an audit topic with a time-based policy:

```bash
termlink channel create --name audit:claim-events \
  --retention days --retention-value 30
```

Create a single-value state topic — only the latest envelope is kept,
restart-safe (T-2142):

```bash
termlink channel create --name state:deploy-mode \
  --retention latest
```

Subscribers see the current value with `channel.subscribe` from offset 0
and get exactly one envelope back. Posting a new value automatically
trims the previous one on the next sweep. Pairs naturally with the
broadcast-with-replay primitive — `--retention latest` topics never
need cv_key annotations because the topic name IS the key.

**Auto-pick (T-2145).** When the CLI's `ensure_topic` (used by
`channel post --ensure-topic` and any auto-create path) creates a
topic whose name starts with `state:`, it auto-picks `Retention::Latest`
— operators don't need to remember the `--retention latest` flag for
single-value-state topics. Sibling of T-2126 (which auto-picks
`Messages(1000)` for high-rate `agent-*` / `dm:*` patterns). The hub
emits a defence-in-depth warn if a `state:*` topic is created with
`Retention::Forever` via a direct `channel.create` call that bypasses
the CLI auto-pick path. The two predicates (high-rate, single-value-state)
are disjoint by prefix — no double-warn.

### Reading current retention

```bash
termlink channel info <topic>
```

Returns `{ok, name, retention: {kind, value}, ...}`. Check the
`retention.kind` and `retention.value` fields to confirm the topic
matches expectations.

### Why this matters

The growth rate of high-rate topics under `Retention::Forever` is
roughly:

```
agents × heartbeats/min × 60 × 24 ≈ envelopes/day
   5  ×       2          × 60 × 24 ≈ 14,400/day
```

At 14,400 envelopes/day on `agent-presence`, the subscribe-path
slowdown that triggered T-1991 will start to show within ~2 weeks. The
hub keeps working; subscribers degrade silently until someone notices
`/peers` is slow.

`Messages(1000)` caps that growth: at 5 agents × 30s heartbeats, the
topic holds ~50 minutes of history. cv_index (substrate #9) covers the
"current value per agent" lookup in O(N_agents), so the durable log
doesn't need to hold every heartbeat for late-joiner state — it only
needs enough for subscribers walking the recent past. 1000 is the
sweet spot.

For *non*-high-rate topics, `Forever` is fine: framework audit logs,
human-curated knowledge bases, anything growing at <1 envelope/minute.

## Worked example

A minimal "two workers process a 5-unit queue" walkthrough on a
single hub.

```bash
# Setup — start the hub and post 5 units of work
termlink hub start &
sleep 2
for i in 0 1 2 3 4; do
  termlink channel post work-queue --payload "unit-$i"
done

# Start two workers in different terminals
# Terminal A:
bash scripts/be-reachable.sh start --agent-id worker-alpha --capabilities deploy
# (then run the worker loop above with WORKER_ID=worker-alpha)

# Terminal B:
bash scripts/be-reachable.sh start --agent-id worker-beta --capabilities deploy
# (then run the worker loop above with WORKER_ID=worker-beta)

# Start the orchestrator in a third terminal:
bash scripts/be-reachable.sh start --agent-id orchestrator-0 --capabilities orchestrate
# (then run the orchestrator loop above with ORCHESTRATOR_ID=orchestrator-0)

# Watch substrate progress from a fourth terminal:
termlink substrate status --watch 5 --log ~/.termlink/substrate.log
```

Expected behaviour:

1. `find_idle` returns `[worker-alpha, worker-beta]` initially.
2. Orchestrator claims offset 0, transfers to (say) worker-alpha.
3. `find_idle` immediately drops worker-alpha (it now holds a claim).
4. Orchestrator claims offset 1, transfers to worker-beta.
5. Both workers process in parallel; `claims-summary` shows
   `active=2, expired=0`.
6. As workers `release --ack`, the orchestrator picks up offsets 2-4
   and assigns to whichever worker frees up first.
7. After unit-4 releases, `claims-summary` shows `active=0`,
   `find_idle` shows both workers idle again.
8. `substrate.history --since 1` shows the full transition timeline.

## In-tree consumer — `scripts/orchestrator-backlog-drain.sh`

T-2204 (T-2018 §6) shipped the first in-tree consumer that exercises
the full claim-pipeline against a real workload. It lives at
`scripts/orchestrator-backlog-drain.sh` and drains the project's own
backlog: agent-eligible tasks in `.tasks/active/` (owner≠human,
horizon∈{now,next}, status∈{captured,started-work}, workflow_type≠
inception).

It differs from the abstract walkthrough above in three ways worth
calling out:

1. **The work-source is `.tasks/active/`, not a hand-posted queue.**
   The script enumerates tasks via the same logic the auditor uses,
   classifies each as `closure-ready` (0 unchecked Agent ACs, just
   needs `Verification` + flip to `work-completed`) or `needs-work`
   (N unchecked Agent ACs, real implementation), and builds a per-unit
   DM brief that links the worker back to the task file.

2. **`--dry-run` is the default; `--live` must be explicit.** The
   script refuses to post/claim/transfer/DM without `--live`. Dry-run
   prints the intended dispatches with full `claim_payload` and
   `dm_body` so the orchestrator policy can be validated against the
   live backlog before any hub writes. This is the substrate
   equivalent of `fleet doctor --auto-heal --dry-run` (T-1684).

3. **Round-robin with per-worker cap.** Workers are picked
   round-robin across the idle set returned by `find-idle`, but each
   worker is capped at `--per-worker-max` (default 3) concurrent
   dispatches. Beyond that, units fall through to `[SKIP]
   no-idle-worker` until releases free up worker slots. This avoids
   piling N units on the first idle worker only because it appeared
   first in the JSON.

Invocation:

```bash
# Validate the policy against the current backlog (no hub writes)
scripts/orchestrator-backlog-drain.sh --dry-run

# Same, scoped to first 5 units (useful for iterating brief copy)
scripts/orchestrator-backlog-drain.sh --dry-run --limit 5

# Actually dispatch — requires LIVE idle workers with capability=backlog-drain
scripts/orchestrator-backlog-drain.sh --live

# Custom worker pool (e.g. AEF workers advertising different capability)
scripts/orchestrator-backlog-drain.sh --live --capability aef-worker --limit 10
```

Output (one `DISPATCH` line per unit, plus a header + summary):

```
# orchestrator-backlog-drain.sh — T-2204
# mode=dry-run capability=backlog-drain queue_topic=work-queue limit=20 per_worker_max=3
# orchestrator=root-claude-dimitrimintdev

# Step 1: governor pre-flight (#10)
#   conn_active=3/256 cap_hits_total=0 rate_hits_total=0

# Step 2: enumerate agent-eligible work-units from .tasks/active/
#   found 21 agent-eligible units

# Step 3: discover idle workers via find-idle (#2)
#   capability=backlog-drain  idle_workers=2 (excluding self=orchestrator-0)
#     - worker-alpha
#     - worker-beta

# Step 4: pair-and-dispatch
DISPATCH [DRY-RUN] worker=worker-alpha unit=T-1166 classification=closure-ready ac_count=0
         claim_payload={"task_id":"T-1166","classification":"closure-ready","ac_count":0,"dispatched_by":"orchestrator-0"}
         dm_body="T-2204 dispatch [closure-ready] T-1166 — … Run the task's ## Verification block. If all pass, commit, then 'fw task update T-1166 --status work-completed'."
DISPATCH [DRY-RUN] worker=worker-beta unit=T-1457 classification=needs-work ac_count=1
…
# Summary: total=21 dispatched=6 no_worker=15 failures=0 mode=dry-run
```

How AEF integrates as a worker: spawn a claude-code session on the
AEF host with `/be-reachable start --capabilities backlog-drain`. The
notify hook at `/tmp/aef-arrived.sh` (or its production analogue) will
auto-greet on first arrival; the orchestrator picks the worker up on
the next dispatch pass. The worker loop is the canonical pattern in
the [Canonical worker pattern](#canonical-worker-pattern) section
above — `release --ack` on success, `release` (no `--ack`) to return
the offset for retry.

The script is read-only against the filesystem (it does not touch
`.tasks/` source files); all mutations happen via hub RPC and are
visible in `fleet history --include-heals`, `claims-history`, and
`substrate.history` for retrospective audit.

### Worker-side companion — `scripts/worker-backlog-drain.sh`

T-2205 shipped the worker-side mirror at `scripts/worker-backlog-drain.sh`.
AEF integrators clone both `orchestrator-backlog-drain.sh` and
`worker-backlog-drain.sh` to have a complete substrate consumer kit.

The worker polls `channel claims` (the LIST verb T-2037, NOT
`claims-summary` which is aggregate-only) for claims held by its
`worker_id`, reads each work-unit envelope via `channel subscribe
--cursor <offset> --limit 1`, decodes the base64-encoded payload, and
renders one of three actions:

| Mode | Behaviour |
|---|---|
| `--dry-run` (default) | Print held claims + intended brief; no hub writes |
| `--live` | Print + prompt operator (stdin) with `ack` / `retry` / `skip` |
| `--auto-noop` | `release --ack` every claim WITHOUT doing the work — substrate smoke-test only, never production |

Invocation:

```bash
# Inspect what I'm holding
scripts/worker-backlog-drain.sh --dry-run --once

# Interactive operator session — poll every 15s, prompt on each claim
scripts/worker-backlog-drain.sh --live

# Substrate smoke-test — releases everything in the pipe without doing work
# (useful only for proving the claim→release lifecycle end-to-end)
scripts/worker-backlog-drain.sh --auto-noop --once
```

Identity resolution mirrors the orchestrator: `--worker-id` flag →
`TERMLINK_AGENT_ID` env → `~/.termlink/be-reachable.state`. SIGINT
exits the poll loop cleanly.

Prerequisite (one-time per hub): create the work-queue topic with the
recommended bounded retention before the orchestrator's first `--live`
pass — the orchestrator surfaces `unknown topic` loudly with the
exact create command if missing:

```bash
termlink channel create --retention "messages:1000" work-queue
```

End-to-end smoke validated on the local hub: orchestrator dry-run
identifies 21 agent-eligible units against the live backlog; worker
dry-run reads a simulated claim and decodes its payload (task_id,
classification, ac_count, dispatched_by) correctly; release --ack
closes the loop with `{"ok":true,"ack":true}` from the hub.

### Validation evidence — first complete end-to-end LIVE pipeline

T-2206 captured the first complete LIVE pipeline execution against
the real backlog (one-keystroke proof the substrate kit closes the
loop). Sequence on a single host with the orchestrator running as
`fake-orch` so the local session shows up as an eligible worker:

```text
$ scripts/orchestrator-backlog-drain.sh --live --orchestrator-id fake-orch --limit 1
# Step 1: governor pre-flight (#10)
#   conn_active=3/256 cap_hits_total=0 rate_hits_total=0
# Step 2: enumerate agent-eligible work-units from .tasks/active/
#   found 20 agent-eligible units
# Step 3: discover idle workers via find-idle (#2)
#   capability=backlog-drain  idle_workers=1 (excluding self=fake-orch)
#     - root-claude-dimitrimintdev
# Step 4: pair-and-dispatch
DISPATCH [LIVE] worker=root-claude-dimitrimintdev unit=T-1166 \
         claim_id=clm-…-work_queue-3 offset=3 classification=closure-ready
# Summary: total=20 dispatched=1 no_worker=0 failures=0 mode=live

$ termlink channel claims work-queue --json | jq '.claims[0]'
{ "claim_id":  "clm-…-work_queue-3",
  "claimer":   "root-claude-dimitrimintdev",   ← claim-transferred from fake-orch
  "offset":    3,
  "topic":     "work-queue" }

$ scripts/worker-backlog-drain.sh --auto-noop --once
CLAIM #1/1
  offset=3 claim_id=clm-…-work_queue-3 ttl_remaining=599535ms
  unit: task=T-1166 classification=closure-ready ac_count=0 dispatched_by=fake-orch
  action: [AUTO-NOOP] releasing without doing work (substrate smoke-test only)
ack: true

$ termlink channel claims work-queue --json | jq '.count'
0
```

The pipeline closes the loop cleanly: orchestrator posts a work-unit,
claims it, atomically transfers to the worker (T-2046 #3 primitive,
no race window), DM dispatches the brief; worker discovers the
held claim, reads the envelope at the offset, decodes the
base64-encoded payload, releases with --ack. Final hub state has
zero active claims and the offset is permanently retired by the ack.

This validation is the load-bearing artifact for the substrate
consumer kit: future AEF integrators can read it once to trust the
end-to-end pipeline before running their own smoke.

## Related primitives — per-primitive docs

The recipe above stitches together every shipped substrate primitive.
For full details on each:

- **#1 CLAIM** — [`substrate-claim-primitive.md`](substrate-claim-primitive.md)
  + lifecycle, cap-overflow, cooperative vs Tier-0 release
- **#2 DISPATCH** — [`agent-find-idle.md`](agent-find-idle.md)
  + role/capability filters, presence wiring
- **#3 ASSIGN** — [`substrate-claim-primitive.md` § Claim-transfer](substrate-claim-primitive.md)
  + atomicity guarantee, distinction from force-release
- **#5 RESILIENCE** — [`substrate-offline-queue-recipe.md`](substrate-offline-queue-recipe.md)
  + the durable FIFO for blip absorption
- **#5 idempotency** — [`substrate-post-idempotency.md`](substrate-post-idempotency.md)
  + the exactly-once contract via dedupe LRU
- **#9 BROADCAST-WITH-REPLAY** — [`substrate-broadcast-with-replay.md`](substrate-broadcast-with-replay.md)
  + cv_index, late-joiner snapshots, producer wiring
- **#10 BACKPRESSURE** — [`substrate-governor.md`](substrate-governor.md)
  + connection cap + rate limit + dedupe + cv_overflow observability
- **SUBSTRATE-PULSE** (composition, not §6 primitive) — [`substrate-status.md`](substrate-status.md)
  + cross-primitive rollup (substrate status CLI/MCP + watch + notify + log + history)
- **G-060 cross-hub** — [`channel-topic-semantics.md`](channel-topic-semantics.md)
  + why hubs don't federate state; how to compose
- **Tunables reference** — [`substrate-tunables.md`](substrate-tunables.md)
  + canonical list of every `TERMLINK_*` env var that tunes hub or
  client behavior — defaults, range, when-to-tune-up/down,
  symptom-when-misconfigured. Read this before raising
  `TERMLINK_MAX_CONNECTIONS`, lowering `TERMLINK_DEDUPE_TTL_MS`, or
  changing any other knob.
- **Cron monitoring recipes** — [`substrate-cron-recipes.md`](substrate-cron-recipes.md)
  + ready-to-install cron + notify-script templates for every
  observability surface. Pair the AEF integration walkthrough here with
  this doc once you're ready to deploy production monitoring.

## Related ADR sections

- [§6 Required primitives (the build manifest)](../architecture/parallel-execution-substrate.md)
  — the primitives this recipe assumes are built
- [§9 Collaboration seam](../architecture/parallel-execution-substrate.md)
  — the contract between substrate and AEF layer; this recipe is the
  consumer-facing half
- [§10 Invariants](../architecture/parallel-execution-substrate.md)
  — what must not be violated (strict star, append-log durability,
  producer ≠ judge at the seam)

## References

- T-2018 — arc-parallel-substrate ADR (`docs/architecture/parallel-execution-substrate.md`)
- T-2019/T-2042/T-2046 — claim primitive build chain
- T-2020/T-2045 — find-idle primitive build chain
- T-2051 — outbound queue (substrate primitive #5 RESILIENCE)
- T-2049 — post idempotency / dedupe
- T-2103..T-2107 — broadcast-with-replay (substrate primitive #9) build
- T-2048..T-2119 — backpressure (substrate primitive #10) build chain
- T-2111..T-2117 — substrate status build chain (SUBSTRATE-PULSE composition; not a §6 manifest primitive — T-2026 reserves #11 for typed agent-launch)
- T-2124 — this doc (master integration recipe)
