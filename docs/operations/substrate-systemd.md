# Persistent substrate components via systemd

> T-2165 (orchestrator) + T-2167 (worker). Sibling of
> [listener-heartbeat-systemd.md](listener-heartbeat-systemd.md) (T-1840) —
> that doc covers presence-emission as a service; this doc covers the
> substrate's coordination loops (T-2148 orchestrator + T-2152 worker pickup
> + T-2146 worker per-unit) as services.

`scripts/substrate-orchestrator-loop.sh` (T-2148) and
`scripts/substrate-worker-pickup.sh` (T-2152) both run in foreground —
operators previously backgrounded them manually or wired them into ad-hoc
supervisors; reboots reset everything. This recipe wraps both loops in
systemd template units so dispatch + pickup come back automatically on
boot, and covers the per-unit worker pattern (which has a different shape
than the supervisors).

## When to install

Install the orchestrator unit when:

- The host runs persistently and should dispatch substrate work without
  operator intervention.
- The work-topic queue is long-lived (envelopes arrive at any time, not
  bounded by a single deploy).
- The orchestrator identity should appear on the fleet's `agent-presence`
  topic as a LIVE producer.

You **don't** want this when:

- The host's hub is volatile (`TERMLINK_RUNTIME_DIR` on `/tmp`,
  PL-021). The T-2163 preflight gate will refuse to start (exit 4) and
  systemd will restart-loop loudly — **good**, that's the failure mode
  you want — but fix the hub persistence first
  ([CLAUDE.md hub-auth-rotation protocol](../../CLAUDE.md)) before
  enabling the unit, or it'll just generate journal noise.
- You're running a single-shot orchestrator pass with `--max-envelopes N`.
  Use a one-shot systemd `Type=oneshot` unit or just run the script
  directly — Restart=on-failure isn't the right shape there.

## How preflight interacts with `Restart=`

All three long-running substrate scripts —
`scripts/substrate-orchestrator-loop.sh` (T-2148),
`scripts/substrate-worker-loop.sh` (T-2146), and
`scripts/substrate-worker-pickup.sh` (T-2152, gate added T-2166) — run
`scripts/substrate-preflight.sh` at startup (T-2163 / T-2166). The contract:

| Preflight outcome | Script behavior | systemd outcome under `Restart=on-failure` |
|---|---|---|
| PASS (exit 0) | Silent, continue | Service runs normally |
| WARN (exit 1) | Print to stderr, continue | Service runs normally; warnings in journal |
| FAIL (exit 2) | Print to stderr, refuse to start (**exit 4**) | systemd restart-loops loudly; misconfigured host surfaces in seconds, NOT silent wedge |

Both unit templates (orchestrator @ T-2165, worker @ T-2167) ship with
`Restart=on-failure` + `RestartSec=10s` for exactly this reason. A
misconfigured host (volatile `/tmp` runtime_dir, missing `hubs.toml`, dead
be-reachable) produces a visible restart-loop in `systemctl status` and
`journalctl -u`, instead of the production-killing failure mode of a
silently-wedging service.

If you want to bypass the preflight gate (e.g. on a CI runner where the
host is known broken-on-purpose), append `--skip-preflight` via
`TERMLINK_SO_EXTRA_ARGS` in the per-instance `.env` file.

## Install

```bash
# 1. Stage the unit template
sudo install -d /etc/systemd/system
sudo cp /opt/termlink/systemd-templates/termlink-substrate-orchestrator@.service \
        /etc/systemd/system/

# 2. Author an env file per orchestrator instance
sudo install -d /etc/termlink/substrate-orchestrator

sudo tee /etc/termlink/substrate-orchestrator/my-orchestrator.env >/dev/null <<'EOF'
# T-2165 per-instance config — read by
# termlink-substrate-orchestrator@my-orchestrator.service
#
# Required: --work-topic. The substrate channel to subscribe to (work
# envelopes arrive on this topic).
TERMLINK_SO_WORK_TOPIC=aef:deploy

# Optional: --capability filter. If set, only LIVE workers advertising
# this capability are eligible to receive dispatched claims. See
# `termlink agent find-idle --help` and /be-reachable --capabilities.
TERMLINK_SO_CAPABILITY=deploy

# Optional: --ttl-ms (default 60000). Initial claim lease. Worker can
# renew via channel renew (worker-loop does this automatically).
# TERMLINK_SO_TTL_MS=60000

# Optional: --idle-poll-ms (default 5000). Backoff when no worker is
# available.
# TERMLINK_SO_IDLE_POLL_MS=5000

# Optional: --hub. Default is the local hub.
# TERMLINK_SO_HUB=192.168.10.122:9100

# Optional: --max-envelopes. Default 0 = unlimited. Set for bounded
# smoke runs (then use Type=oneshot instead of Type=exec — see "When to
# install" above).
# TERMLINK_SO_MAX_ENVELOPES=

# Load-bearing — see PL-021. Override the default /var/lib/termlink only
# if your hub is configured to read state from a different path.
# TERMLINK_RUNTIME_DIR=/var/lib/termlink

# Optional: extra flags appended verbatim. Reserved for forward-compat,
# also where you'd add --skip-preflight on a CI runner that's
# intentionally misconfigured.
# TERMLINK_SO_EXTRA_ARGS=
EOF

# 3. Enable + start
sudo systemctl daemon-reload
sudo systemctl enable --now termlink-substrate-orchestrator@my-orchestrator.service

# 4. Verify
systemctl status termlink-substrate-orchestrator@my-orchestrator
journalctl -u termlink-substrate-orchestrator@my-orchestrator -f
```

The shipped template (`systemd-templates/termlink-substrate-orchestrator@.service`)
translates env vars → CLI flags inline:

```ini
ExecStart=/bin/sh -c '\
    set -e; \
    flags="--orchestrator-id %i --work-topic $${TERMLINK_SO_WORK_TOPIC} --ttl-ms $${TERMLINK_SO_TTL_MS} --idle-poll-ms $${TERMLINK_SO_IDLE_POLL_MS}"; \
    if [ -n "$${TERMLINK_SO_CAPABILITY:-}" ]; then flags="$$flags --capability $${TERMLINK_SO_CAPABILITY}"; fi; \
    ...
    exec $${TERMLINK_SUBSTRATE_SCRIPT} $${flags} $${TERMLINK_SO_EXTRA_ARGS:-}'

Restart=on-failure
RestartSec=10s
```

`%i` is the instance specifier (orchestrator-id) — `.env` carries the rest.

## The worker-side pattern

The substrate's worker side has two distinct shapes — pick by how the
work-source connects to substrate claims:

### 1. Workers spawned by orchestrator DM dispatch (the canonical pattern)

The orchestrator-loop fires a fire-and-forget `agent contact` DM to the
chosen worker with `claim=<id> topic=<t> offset=<n>`. The worker host
runs `scripts/substrate-worker-pickup.sh` (T-2152) — a long-running
supervisor that polls the agent inbox and spawns
`substrate-worker-loop.sh` per dispatch DM. This is the production
default and the path T-2167's worker template installs.

Use the shipped worker template (T-2167):

```bash
sudo install -d /etc/termlink/substrate-worker
sudo cp /opt/termlink/systemd-templates/termlink-substrate-worker@.service \
        /etc/systemd/system/

sudo tee /etc/termlink/substrate-worker/deploy-worker-a.env >/dev/null <<'EOF'
# T-2167 per-instance config — read by
# termlink-substrate-worker@deploy-worker-a.service
#
# Required: --cmd template. The shell command spawned per claimed unit.
# Env vars TERMLINK_CLAIM_ID, TERMLINK_CLAIM_TOPIC, TERMLINK_CLAIM_OFFSET,
# TERMLINK_CLAIMER are set by pickup.sh per dispatch — quote so the
# spawned worker-loop sees them. See `substrate-worker-pickup.sh --help`.
TERMLINK_SW_CMD='python3 /opt/myapp/run.py --topic "$TERMLINK_CLAIM_TOPIC" --offset "$TERMLINK_CLAIM_OFFSET"'

# Optional: --hub addr. Default: local hub.
# TERMLINK_SW_HUB=192.168.10.122:9100

# Optional: --poll-ms (default 2000). Inbox poll cadence.
# TERMLINK_SW_POLL_MS=2000

# Optional: --max-claims (default 0 = unlimited). Set for bounded smoke runs.
# TERMLINK_SW_MAX_CLAIMS=

# Load-bearing — see PL-021. Override the default /var/lib/termlink only if
# the hub on this host is configured to read state from a different path.
# TERMLINK_RUNTIME_DIR=/var/lib/termlink

# Optional: extra flags appended verbatim. Reserved for forward-compat; also
# where --skip-preflight goes on a CI runner that's intentionally misconfigured.
# TERMLINK_SW_EXTRA_ARGS=
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now termlink-substrate-worker@deploy-worker-a.service

systemctl status termlink-substrate-worker@deploy-worker-a
journalctl -u termlink-substrate-worker@deploy-worker-a -f
```

The instance specifier `%i` (here `deploy-worker-a`) becomes the
worker-id pickup.sh advertises — peers reach this worker via
`agent find-idle` + `agent contact deploy-worker-a`, and orchestrators
claim-transfer ownership to it by that name. One env file per worker
instance; one systemd unit per worker.

If you need custom dispatch logic instead of pickup.sh (e.g. dispatch
through Slack, gate on a feature flag, post to a downstream queue), the
prior hand-roll pattern still works: write your own inbox-poll daemon
using `termlink agent dms` or `channel subscribe dm:<self>:*`, and spawn
`substrate-worker-loop.sh --claim-id <id> --topic <t> --offset <n>
--cmd '<your-work-cmd>'` on receipt. See
`docs/operations/substrate-orchestrator-recipe.md` § "Canonical worker
pattern" for the full custom-supervisor walkthrough.

### 2. Workers process their own work-source

Some integrations want workers to pull from an external queue (a file
queue, an SQS-style API, a directory of pending work) rather than wait
on substrate DM dispatch. The worker service polls that external queue,
picks one unit, calls `substrate-worker-loop.sh --topic <substrate-topic>
--offset <derived> --cmd 'process <unit>'` to gate the run through
substrate claims, and loops. Pickup.sh + the T-2167 template do not fit
this shape; use systemd `Type=simple` + `Restart=always` with your own
service file calling the external-queue-poll script directly.

### 3. Workers are short-lived (CI / ad-hoc)

If the worker only ever runs once per envelope (CI, batch job, manual
operator dispatch), don't wrap it in systemd — invoke
`substrate-worker-loop.sh` directly from the invoker (Jenkins step, ssh
command, etc.). The script's `--skip-preflight` flag is useful for CI
runners where preflight is known broken-on-purpose.

For all three patterns the T-2163/T-2166 preflight gate behaves
identically: silent on PASS, warn-and-continue on WARN, refuse-to-start
(exit 4) on FAIL. The exit-4 contract gives whichever supervisor wraps
the worker (systemd, the inbox-poll daemon, the CI runner) a clean
signal to either restart-loop loudly or fail the build.

## Worker template (T-2167)

The shipped worker template
(`systemd-templates/termlink-substrate-worker@.service`) is symmetric to
the orchestrator template — same env→flags pattern, same hardening, same
Restart=on-failure + exit-4 contract. The ExecStart resolves env vars to
pickup.sh flags inline:

```ini
ExecStart=/bin/sh -c '\
    set -e; \
    if [ -z "$${TERMLINK_SW_CMD:-}" ]; then echo "termlink-substrate-worker@%i: TERMLINK_SW_CMD not set in .env" >&2; exit 2; fi; \
    flags="--worker-id %i --cmd $${TERMLINK_SW_CMD} --poll-ms $${TERMLINK_SW_POLL_MS}"; \
    if [ -n "$${TERMLINK_SW_HUB:-}" ]; then flags="$$flags --hub $${TERMLINK_SW_HUB}"; fi; \
    if [ -n "$${TERMLINK_SW_MAX_CLAIMS:-}" ]; then flags="$$flags --max-claims $${TERMLINK_SW_MAX_CLAIMS}"; fi; \
    exec $${TERMLINK_SUBSTRATE_SCRIPT} $${flags} $${TERMLINK_SW_EXTRA_ARGS:-}'

Restart=on-failure
RestartSec=10s
```

`%i` is the instance specifier (worker-id); `--cmd` is required (the unit
refuses to start with exit 2 if `TERMLINK_SW_CMD` is unset in the .env);
all other knobs are optional with documented defaults.

Multiple worker instances on one host: same template, different
`.service` filename. Each instance gets its own `<id>.env` and its own
systemd unit:

```bash
sudo systemctl enable --now termlink-substrate-worker@worker-1.service
sudo systemctl enable --now termlink-substrate-worker@worker-2.service
sudo systemctl enable --now termlink-substrate-worker@worker-3.service
```

Each appears on `agent-presence` under its own worker-id and receives
dispatch DMs independently. Pickup.sh's signal-handling (SIGTERM/SIGINT
→ exit 130) composes correctly with `systemctl stop` — the in-flight
worker-loop is killed cleanly before the supervisor exits.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| Service in restart-loop with exit 4 | Preflight FAIL — runtime_dir on /tmp, missing hubs.toml, etc. | `journalctl -u <unit>` shows the [FAIL] line + remediation hint. Fix the host then `systemctl restart`. |
| Service runs, no envelopes dispatched | Work-topic empty OR no LIVE workers match `--capability` | `termlink channel topic-stats <topic>`, `/find-idle --capability X` |
| `CLAIM_CONFLICT` on every envelope | Multiple orchestrators racing on the same topic | One orchestrator per topic. If you need throughput, partition the topic. |
| Service ok but worker DMs not received | Worker host has no inbox-poll daemon OR `agent-presence` heartbeat dead | Check `/be-reachable status` on the worker host; the orchestrator's `agent contact` is fire-and-forget. |
| `TERMLINK_AGENT_ID` warnings on startup | Orchestrator-id not resolvable | The unit template sets it via `%i` — your `.service` filename must be `termlink-substrate-orchestrator@<id>.service`. |
| Worker unit exits 2 immediately at start | `TERMLINK_SW_CMD` missing in `/etc/termlink/substrate-worker/<id>.env` | The worker template's ExecStart refuses to start if the required CMD is absent. Add `TERMLINK_SW_CMD='...'` and `systemctl restart`. |
| Worker unit running but no work claimed | No orchestrator is dispatching, OR orchestrator's `find-idle` is filtering this worker out (`--capability` mismatch) | Check orchestrator's `--capability` flag vs this worker's `/be-reachable --capabilities`. Also confirm DM topic reachability via `/check-arc` from the worker. |
| Multiple workers running, all idle | Orchestrator pinned to one capability that none advertise | `termlink agent find-idle --capability <cap>` to confirm reachability. Add `--capabilities <cap>` to each worker's heartbeat config. |

## References

- **T-2148** — `scripts/substrate-orchestrator-loop.sh` (the orchestrator loop)
- **T-2146** — `scripts/substrate-worker-loop.sh` (the per-unit worker harness)
- **T-2152** — `scripts/substrate-worker-pickup.sh` (the worker supervisor — long-running inbox-poll daemon that spawns worker-loop per dispatch DM)
- **T-2154** — `scripts/substrate-preflight.sh` (deploy-time correctness check)
- **T-2158** — `/preflight` skill
- **T-2160** — nightly preflight cron canary
- **T-2163** — preflight startup gate on orchestrator-loop + worker-loop (exit 4 = systemd-restart-loop contract)
- **T-2166** — preflight startup gate on worker-pickup (closes the supervisor-preflight symmetry; same exit 4 contract)
- **T-2167** — worker systemd template `termlink-substrate-worker@.service` + this doc's "Worker template" + "Worker-side pattern §1" sections (production-systemd surface for worker side, symmetric to T-2165 orchestrator template)
- **T-2159** — [substrate-tunables.md](substrate-tunables.md) — every
  `TERMLINK_*` env var. Read this before adjusting any knob.
- **T-1840** — [listener-heartbeat-systemd.md](listener-heartbeat-systemd.md) —
  the sibling presence-side service template.
- **T-2124** — [substrate-orchestrator-recipe.md](substrate-orchestrator-recipe.md) —
  master recipe (the orchestrator/worker patterns in long-form prose).
