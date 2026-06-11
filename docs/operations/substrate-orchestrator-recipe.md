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
`CLAIM_ALREADY_HELD` (the worker IS the current holder via transfer).
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

## Failure modes and recovery

| Symptom | Diagnosis | Recovery |
|---|---|---|
| `CLAIM_CONFLICT` (-32015) on Step 3 of orchestrator | Another orchestrator claimed first | Re-loop; try next offset |
| `CLAIM_NOT_OWNED` (-32017) on `claim_transfer` | `by` argument doesn't match current holder | Re-read `channel.claims`; orchestrator's claim may have lapsed before transfer fired |
| `CLAIM_NOT_FOUND` (-32016) on `renew` | Claim already released or lapsed | Worker must re-acquire; renewal cadence too slow |
| `CLAIM_LAPSED` on `renew` | Lease expired before renewal — substrate-correctness footgun | Re-claim under new claim_id. Tune renewal cadence < 50% of TTL |
| `RATE_LIMITED` (-32008) | Worker is posting too fast | Back off; check `/governor` for `rate_hits_total` growth |
| `HUB_AT_CAPACITY` (-32019) | Hub's connection pool exhausted | Back off; check `/governor` for `capacity_hits_total` growth |
| Worker's `channel.post` queued instead of delivered | Hub blip; outbound queue is buffering (#5 RESILIENCE) | Wait — the flush task drains every 5s once hub returns. Check `/queue-status` |
| `cv_overflow > 0` on `governor.status` | A producer is mis-emitting `cv_key` (e.g. timestamp instead of stable id) saturating per-topic cap | Run `/cv-keys <suspect-topic>` to identify; fix the producer |
| `find_idle` returns empty but operator sees LIVE workers | All workers hold at least one active claim | Wait, or expand worker pool |
| Worker DM never arrives after `claim_transfer` succeeds | Doorbell delivery is best-effort; persistent `dm:*` topic carries the canonical message | Worker's `agent.dms` poll catches it on next iteration |
| Orchestrator restarts mid-flight | Outstanding orchestrator-held claims lapse after TTL; in-flight transfers to workers persist | Restart cleanly; no orchestrator-side state to restore (claims are hub-side) |
| Hub restart | Claims table durable (SQLite); cv_index in-memory only (re-populates within one heartbeat cycle); outbound queues durable | All clients reconnect; lease times survive; cv_index converges within ~30s |
| `fleet governor-status` returns `-32001 / Missing 'target' in params` for a hub | That hub runs a pre-T-2048 binary (older than `hub.governor_status`); its unknown-method dispatch misroutes to `event.emit_to` | Upgrade the hub binary via `scripts/fleet-deploy-binary.sh --probe --target <hub>`; see `substrate-governor.md` § "Version-skew diagnosis" (T-2138) |
| `rate_buckets_active` grows monotonically without bound | Hub runs a pre-T-2137 binary; rate-bucket eviction never wired into startup; HashMap accumulates one entry per distinct sender_id ever seen | Upgrade the hub binary; see `substrate-governor.md` § "Reading rate_buckets_active" (T-2138). Until upgrade, hub memory grows ~120 bytes per distinct sender |

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
