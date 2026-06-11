# Substrate — getting started

**Audience:** operators new to the TermLink substrate, or returning to
it after months away. The goal of this doc is to get you from "what is
this?" to a working claim-lifecycle smoke in five minutes, then point
you at the right deeper doc for whatever you want to do next.

## 1. What the substrate is, in one paragraph

The TermLink substrate is a coordination layer for multiple agents
working in parallel (T-2018 §6). It exposes 11 primitives that compose
into a **work-stealing pattern**: an orchestrator subscribes to a queue
of work envelopes, finds idle workers via `agent find-idle`, claims a
slot atomically via `channel claim`, hands ownership to the worker via
`claim-transfer`, and the worker `release`s with `--ack` on completion.
The orchestrator never assigns the same work to two workers; the
worker never half-completes a unit silently. Read-only observability
verbs let an operator inspect the substrate's state without joining the
work stream. Everything below builds on this picture.

## 2. Is the substrate healthy? Four daily verbs

Four read-only Claude Code skills cover the substrate's running state.
Each takes <2 seconds. Read them in any order; chain them when
investigating an incident.

| Skill | Reads | Answers |
|---|---|---|
| `/find-idle` | `agent find-idle` | "Who's free to take work right now?" |
| `/claims` | `channel claims-summary` | "What's in flight on this topic? Any stuck?" |
| `/queue-status` | `channel queue-status` | "Is my outbound queue draining?" |
| `/governor` | `fleet governor-status` | "Is the hub being rate-limited or at capacity?" |

For a single cold-start digest run `/substrate` — it composes all four
into one parallel view (T-2096). If everything reads green the substrate
is steady; if not, the per-skill output tells you what's wrong.

## 3. Your first claim lifecycle — five minutes

The fastest way to internalise the substrate is to run one envelope
through it end-to-end on a smoke topic. Copy-paste this:

```bash
# Step 1 — create a smoke topic (idempotent; safe to re-run).
termlink channel create smoke:my-first --retention "messages:100"

# Step 2 — post one work envelope. Note the offset returned.
termlink channel post smoke:my-first --payload "unit-of-work-1"
# → Posted to smoke:my-first — offset=0

# Step 3 — run the worker harness against that offset. The script
# claims the slot, renews the lease automatically while your --cmd
# runs, and releases with --ack on cmd exit 0.
scripts/substrate-worker-loop.sh \
    --topic smoke:my-first --offset 0 \
    --cmd 'echo "I am claim=$TERMLINK_CLAIM_ID worker; sleeping 2s"; sleep 2' \
    --claimer my-first-worker --ttl-ms 10000
# → claim_id=clm-...
# → I am claim=clm-... worker; sleeping 2s
# → worker ok — release(--ack)
# exit code: 0

# Step 4 — verify cursor advanced (no active claims, no expired).
termlink channel claims-summary smoke:my-first --json
# → {"active_count":0, "expired_count":0, ...}
```

That's the canonical lifecycle. The worker script handled claim →
auto-renew → run → release for you. You now know what every substrate
primitive does because you ran one.

**One-shot health verifier (T-2151).** For an automated PASS/FAIL of
the full pattern above without typing each step, run
`scripts/substrate-smoke.sh`. It posts → claims → claim-transfers →
adopts via worker-loop → verifies cursor-clean, prints one PASS line
per stage, exits 0 if healthy. CI-runnable; `--json` for piping.
Failure mode: `FAIL at stage <name>: <error>` on stderr + exit 1.

To exercise the orchestrator side (find-idle workers + dispatch work
to them automatically), substitute step 3 with the orchestrator harness
and bring up a worker in another terminal:

```bash
# Terminal A — orchestrator (dispatches whatever appears on the topic).
scripts/substrate-orchestrator-loop.sh \
    --work-topic smoke:my-first \
    --orchestrator-id my-first-orch \
    --ttl-ms 30000 --max-envelopes 3

# Terminal B — make yourself a discoverable worker, then loop on
# inbox to pick up dispatched claims.
/be-reachable --capabilities deploy
# (then write a script that polls agent inbox for incoming DMs and
# runs scripts/substrate-worker-loop.sh against each — see the master
# recipe for the full inbox-poll loop, or use the orchestrator-pairs-with-
# worker pattern below.)
```

For a real worker fleet you'll write a service that wraps
`scripts/substrate-worker-loop.sh` per work unit. The master recipe
(T-2124) shows the full pattern.

## 4. Where to go next

Pick whichever question you have:

- **"How do claims work in detail?"** — `docs/operations/substrate-claim-primitive.md`
  covers the claim → renew → release → claim-transfer state machine,
  ownership invariants, and the `claims-history` retrospective verb.
- **"How do I publish a single value many agents can read fresh?"** —
  `docs/operations/substrate-broadcast-with-replay.md` (substrate #9
  + cv_index + `Retention::Latest`).
- **"My hub is being rate-limited / refused"** — `docs/operations/substrate-governor.md`
  (the BACKPRESSURE primitive, tuning `TERMLINK_MAX_CONNECTIONS` /
  `TERMLINK_RATE_LIMIT_PER_SEC` / cv_index overflow).
- **"My worker disconnected mid-task"** — `docs/operations/substrate-offline-queue-recipe.md`
  (the RESILIENCE primitive — durable FIFO that absorbs hub blips).
- **"Did this work apply once, or did the retry double-apply?"** —
  `docs/operations/substrate-post-idempotency.md` (the `client-msg-id`
  + hub LRU dedupe pattern).
- **"I want the full orchestrator/worker AEF integration walkthrough"** —
  `docs/operations/substrate-orchestrator-recipe.md` (master recipe,
  T-2124) is the long-form canonical doc this quickstart is the
  on-ramp for.
- **"What about substrate health overall?"** — `docs/operations/substrate-status.md`
  + the `/substrate` skill.

When you find yourself reaching for a primitive whose docs you've not
yet read, scan the relevant section above and come back to it later —
the substrate is composable, so the daily verbs and recipe scripts
above will cover most operator work.

## References

- **T-2018 §6** — substrate primitive manifest (claims, dispatch,
  resilience, backpressure, broadcast-with-replay, ...)
- **T-2124** — master orchestrator recipe (consumer-facing walkthrough)
- **T-2146** — `scripts/substrate-worker-loop.sh` (worker harness)
- **T-2148** — `scripts/substrate-orchestrator-loop.sh` (orchestrator harness)
- **T-2149** — this doc
