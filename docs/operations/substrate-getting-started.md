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

### Cold-start three-verb sequence

`/substrate` answers "is the substrate healthy right now?" but it's only
the runtime layer. Two more skills answer orthogonal questions and
together form the canonical pickup pattern when landing on a host:

| Verb | Tier | Answers |
|---|---|---|
| `/preflight` (T-2158) | Deploy-time | Is the substrate environment set up correctly? (PL-021 volatile /tmp, hubs.toml, be-reachable state) |
| `/substrate` (T-2096) | Runtime | Is the substrate healthy right now? (composes /find-idle + /claims + /queue-status + /governor) |
| `/canaries` (T-2172/T-2178) | Cron-tier protection | Are my daily watchers firing AND clean? (auto-discovers `.*-canary.log` AND `.heartbeat`) |

Run them in that order at session start. They take <5 seconds combined,
they read-only by contract, and they fail loudly if anything is
structurally wrong — instead of letting you discover the regression
later under pressure. See `substrate-cron-recipes.md` § "Checking that
the canaries are firing" for the meta-canary layered-protection layer
behind `/canaries`.

## 3. Your first claim lifecycle — five minutes

Before deploying, run the pre-flight (T-2154):

```bash
scripts/substrate-preflight.sh
# → [PASS] runtime_dir   TERMLINK_RUNTIME_DIR=/var/lib/termlink (not on /tmp …)
# → [PASS] hubs.toml     /root/.termlink/hubs.toml present (N hub(s) declared)
# → [PASS] be-reachable  alive | absent
# → Summary: 3 pass, 0 warn, 0 fail — substrate-ready.
```

Exit 2 means **stop**: the single largest production failure mode is
`TERMLINK_RUNTIME_DIR` on volatile `/tmp` (PL-021 — hub regenerates
secret + TLS cert every reboot). The preflight catches this BEFORE the
fleet wedges. Exit 1 means warn — you can proceed but the warnings
should be addressed (missing hubs.toml, stale be-reachable.state).
Read-only; safe in CI; `--json` available for piping.

Now run one envelope through end-to-end on a smoke topic. Copy-paste this:

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

Both loop scripts run `substrate-preflight.sh` at startup (T-2163) — on
FAIL they refuse to start with exit 4 (no hub call attempted), on WARN
they print and continue, on PASS silent. systemd will see the exit 4
and restart-loop loudly rather than silently wedging — exactly the
failure mode you want for a misconfigured host. Bypass with
`--skip-preflight` only in CI/test paths where preflight is already
known clean.

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
- **T-2154** — `scripts/substrate-preflight.sh` (deploy-time correctness; the underlying script behind `/preflight`)
- **T-2158** — `/preflight` skill (one-keystroke deploy-time check)
- **T-2159** — [substrate-tunables.md](substrate-tunables.md) — canonical
  reference for every `TERMLINK_*` env var that tunes hub or client
  behavior. Read this before adjusting any knob.
- **T-2163** — preflight startup gate on `scripts/substrate-{orchestrator,worker}-loop.sh`.
  The loops now run substrate-preflight.sh before any hub call: FAIL refuses
  to start (exit 4), WARN prints and continues, PASS silent. `--skip-preflight`
  bypasses (CI / test paths where preflight is already known clean — see
  `scripts/substrate-smoke.sh`). Closes the substrate-arc safety set: CLI
  (T-2154) → skill (T-2158) → nightly cron (T-2160) → runtime-entry gate
  (T-2163). Production deployments now get the PL-021 catch automatically;
  no operator discipline required.
- **T-2165** — [substrate-systemd.md](substrate-systemd.md) +
  `systemd-templates/termlink-substrate-orchestrator@.service` — production
  systemd templates for running `scripts/substrate-orchestrator-loop.sh`
  as a long-running service. Covers how T-2163's preflight exit code 4
  interacts with `Restart=on-failure` (loud restart-loop, not silent
  wedge) and the three worker-side patterns (DM-driven, external-queue,
  short-lived). Sibling of T-1840's listener-heartbeat unit.
- **T-2166** — preflight startup gate on `scripts/substrate-worker-pickup.sh`.
  Mirror of T-2163 for the third long-running substrate supervisor.
  Closes the supervisor-preflight asymmetry: pickup runs preflight at
  startup, refuses with exit 4 on FAIL, prints+continues on WARN, silent
  on PASS. Bypass via `--skip-preflight` for CI/test paths. Under systemd
  Restart=on-failure this turns a misconfigured host into a loud
  restart-loop instead of a silently-running supervisor that fails per
  envelope arrival.
- **T-2167** — `systemd-templates/termlink-substrate-worker@.service` +
  [substrate-systemd.md](substrate-systemd.md) "Worker template" + "Worker-
  side pattern §1" sections — production systemd template for running
  `scripts/substrate-worker-pickup.sh` as a long-running service.
  Symmetric to T-2165's orchestrator template; composes with T-2166's
  exit-4 contract for the loud-restart-loop guarantee. The instance
  specifier `%i` becomes the worker-id substrate identity; multiple
  worker instances per host via different `.service` filenames.
- **T-2169** — `scripts/substrate-systemd-smoke.sh` — static-verify
  regression test for both unit templates (T-2165 + T-2167). Run as
  pre-deploy verification before staging the templates on a new host;
  see [substrate-systemd.md "Pre-deploy verification"](substrate-systemd.md).
  Symmetric pair with T-2170.
- **T-2170** — `scripts/substrate-preflight-smoke.sh` — regression test
  for the preflight (T-2154) exit-code contract that the three loop
  scripts (T-2163/T-2166) gate on. Catches a fail-classified check
  silently returning the wrong exit code — the regression class that
  would lose PL-021 prevention across every production install. Surfaced
  in [substrate-systemd.md "Pre-deploy verification"](substrate-systemd.md).
  Symmetric pair with T-2169 — together they close the substrate-arc
  regression-protection surface end-to-end.
- **T-2172** — `scripts/canary-status.sh` + `/canaries` skill — read-only
  visibility verb for the cron-tier protection layer. Auto-discovers every
  `.context/working/.*-canary.log` + companion `.heartbeat`, classifies
  per-canary status (HEALTHY / FIRING / STALE / NO_HEARTBEAT), surfaces
  the latest-firing log entry inline. Closes the PL-168 dormant-tooling
  antipattern for the cron canaries above. Substrate-arc framing:
  completes the safety set visibility tier (CLI/T-2154 → skill/T-2158 →
  smoke/T-2170 → cron/T-2160 → THIS). Pair with `/preflight` (deploy-
  time) + `/substrate` (runtime) for the three-orthogonal-questions
  cold-start digest. See [substrate-cron-recipes.md "Checking that
  the canaries are firing"](substrate-cron-recipes.md).
- **T-2175 / T-2176 / T-2177** — meta-canary symmetry: substrate-preflight
  and fleet-doorbell-mail canaries now get T-1723-style heartbeat
  freshness checks 80–90 min after their daily run, via the parameterized
  `scripts/check-canary-aliveness.sh`. All three daily canaries
  (release-mirror / substrate-preflight / fleet-doorbell-mail) are
  symmetrically protected against silent cron stoppage. See
  `substrate-cron-recipes.md` § "Layered protection".
- **T-2178** — `/canaries` discovers never-fired canaries via `.heartbeat`
  glob (was log-only). A healthy install whose log file doesn't exist
  yet (the steady state for the empty-log=healthy convention) was
  previously invisible to the operator-pull view; now it renders as
  HEALTHY with `log=--`. Closes a real visibility gap caught during
  T-2175 smoke.
- **T-2162** — [substrate-cron-recipes.md](substrate-cron-recipes.md) —
  ready-to-install cron + notify-script templates for every
  observability surface (preflight-nightly, page-on-cap-hits, page-on-
  cv-overflow, page-on-stuck-claims, page-on-queue-pending,
  dispatch-on-idle). Operators copy six cron lines and get production
  monitoring.
