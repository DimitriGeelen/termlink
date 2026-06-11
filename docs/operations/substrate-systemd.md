# Persistent substrate components via systemd

> T-2165. Sibling of [listener-heartbeat-systemd.md](listener-heartbeat-systemd.md)
> (T-1840) — that doc covers presence-emission as a service; this doc covers
> the substrate's coordination loops (T-2148 orchestrator + T-2146 worker) as
> services.

`scripts/substrate-orchestrator-loop.sh` (T-2148) runs in foreground —
operators currently background it manually or wire it into ad-hoc supervisors;
reboots reset everything. This recipe wraps the orchestrator loop in a
systemd template unit so the dispatcher comes back automatically on boot,
and covers the worker-side pattern (which has a different shape than the
orchestrator).

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

`scripts/substrate-orchestrator-loop.sh` (T-2148) and
`scripts/substrate-worker-loop.sh` (T-2146) both run
`scripts/substrate-preflight.sh` at startup (T-2163). The contract:

| Preflight outcome | Script behavior | systemd outcome under `Restart=on-failure` |
|---|---|---|
| PASS (exit 0) | Silent, continue | Service runs normally |
| WARN (exit 1) | Print to stderr, continue | Service runs normally; warnings in journal |
| FAIL (exit 2) | Print to stderr, refuse to start (**exit 4**) | systemd restart-loops loudly; misconfigured host surfaces in seconds, NOT silent wedge |

The unit template ships with `Restart=on-failure` + `RestartSec=10s` for
exactly this reason. A misconfigured host (volatile `/tmp` runtime_dir,
missing `hubs.toml`, dead be-reachable) produces a visible restart-loop
in `systemctl status` and `journalctl -u`, instead of the
production-killing failure mode of a silently-wedging service.

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

`substrate-worker-loop.sh` (T-2146) has a different shape than the
orchestrator: it is **per-work-unit**, not long-running. It claims one
specific `(topic, offset)`, runs your `--cmd`, and releases. Wrapping it
in a long-running systemd unit doesn't fit — there's no natural
work-source.

Three production patterns:

### 1. Workers are spawned by the orchestrator's DM dispatch

The orchestrator-loop fires a fire-and-forget `agent contact` DM to the
chosen worker with `claim=<id> topic=<t> offset=<n>`. The worker host
runs an inbox-poll daemon (using `termlink agent dms` or
`channel subscribe dm:<self>:*`) that, on receipt of a claim DM, spawns
`substrate-worker-loop.sh --claim-id <id> --topic <t> --offset <n>
--cmd '<your-work-cmd>'`. The daemon can be a systemd `Type=simple`
service or a shell loop. See `docs/operations/substrate-orchestrator-recipe.md`
§ "Canonical worker pattern" for the full pattern.

### 2. Workers process their own work-source

Some integrations want workers to pull from an external queue (a file
queue, an SQS-style API, a directory of pending work). The worker
service polls that queue, picks one unit, calls
`substrate-worker-loop.sh --topic <substrate-topic> --offset <derived>
--cmd 'process <unit>'` to gate the run through substrate claims, and
loops. Systemd `Type=simple` + `Restart=always` is the right shape.

### 3. Workers are short-lived (CI / ad-hoc)

If the worker only ever runs once per envelope (CI, batch job, manual
operator dispatch), don't wrap it in systemd — invoke
`substrate-worker-loop.sh` directly from the invoker (Jenkins step, ssh
command, etc.). The script's `--skip-preflight` flag is useful for CI
runners where preflight is known broken-on-purpose.

For all three patterns the T-2163 preflight gate behaves identically:
silent on PASS, warn-and-continue on WARN, refuse-to-start (exit 4) on
FAIL. The exit-4 contract gives whichever supervisor wraps the worker
(systemd, the inbox-poll daemon, the CI runner) a clean signal to
either restart-loop loudly or fail the build.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| Service in restart-loop with exit 4 | Preflight FAIL — runtime_dir on /tmp, missing hubs.toml, etc. | `journalctl -u <unit>` shows the [FAIL] line + remediation hint. Fix the host then `systemctl restart`. |
| Service runs, no envelopes dispatched | Work-topic empty OR no LIVE workers match `--capability` | `termlink channel topic-stats <topic>`, `/find-idle --capability X` |
| `CLAIM_CONFLICT` on every envelope | Multiple orchestrators racing on the same topic | One orchestrator per topic. If you need throughput, partition the topic. |
| Service ok but worker DMs not received | Worker host has no inbox-poll daemon OR `agent-presence` heartbeat dead | Check `/be-reachable status` on the worker host; the orchestrator's `agent contact` is fire-and-forget. |
| `TERMLINK_AGENT_ID` warnings on startup | Orchestrator-id not resolvable | The unit template sets it via `%i` — your `.service` filename must be `termlink-substrate-orchestrator@<id>.service`. |

## References

- **T-2148** — `scripts/substrate-orchestrator-loop.sh` (the loop)
- **T-2146** — `scripts/substrate-worker-loop.sh` (the worker harness)
- **T-2154** — `scripts/substrate-preflight.sh` (deploy-time correctness check)
- **T-2158** — `/preflight` skill
- **T-2160** — nightly preflight cron canary
- **T-2163** — preflight startup gate on both loops (exit 4 = systemd-restart-loop contract)
- **T-2159** — [substrate-tunables.md](substrate-tunables.md) — every
  `TERMLINK_*` env var. Read this before adjusting any knob.
- **T-1840** — [listener-heartbeat-systemd.md](listener-heartbeat-systemd.md) —
  the sibling presence-side service template.
- **T-2124** — [substrate-orchestrator-recipe.md](substrate-orchestrator-recipe.md) —
  master recipe (the orchestrator/worker patterns in long-form prose).
