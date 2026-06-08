# Substrate: Hub-Owned Idle/Busy Agent Registry

> **For:** Orchestrators building "dispatch to the next free worker" logic
> against a TermLink hub. Part of the arc-parallel-substrate ADR (T-2018) —
> the second substrate primitive after the claim trio (T-2019). T-2020 GO.

## What it gives you

One read-only RPC that answers "which agents are LIVE and not currently
holding any active claim?" — derived server-side from existing state, no
new persistence layer.

| Surface | Form |
|---|---|
| RPC | `agent.find_idle` with `{role?, capabilities?, limit?}` → `{ok, idle: [{agent_id, last_heartbeat_ms, role, capabilities}, ...]}` |
| CLI | `termlink agent find-idle [--role R] [--capability C ...] [--limit N] [--json]` |
| MCP tool | `termlink_agent_find_idle` (same params, JSON shape) |
| Rust API | `Bus::find_idle_agents(role, capabilities, limit, live_window_ms)` |

The derivation:

```
idle_agents = LIVE(agent-presence) \ DISTINCT(claims.claimed_by)
```

- `LIVE(agent-presence)` = agents whose latest heartbeat on the
  `agent-presence` topic is newer than `live_window_ms` (60 s by default,
  i.e. 2× the canonical 30 s heartbeat interval).
- `DISTINCT(claims.claimed_by)` = every agent that currently holds at least
  one active claim (`claimed_until > now`).
- Set difference — an agent that's heart-beating but not claiming anything
  is "idle and findable."

No new persistent state. The presence topic and the claims table already
exist; this RPC just joins them.

## The minimum viable orchestrator loop

```text
                ┌──────────────────────────────┐
                │ agent.find_idle              │
                │  {role, capabilities, limit} │
                └──────────────┬───────────────┘
                               │
                               │ idle: [agent_a, agent_b, ...]
                               ▼
                ┌──────────────────────────────┐
                │ channel.claim                │
                │  topic, offset, claimer      │
                └──────────────┬───────────────┘
                               │ claim_id
                               ▼
                ┌──────────────────────────────┐
                │ dispatch to agent_a via      │
                │ agent.contact / DM / …       │
                └──────────────┬───────────────┘
                               │
                               ▼
                ┌──────────────────────────────┐
                │ channel.release  ack=true    │
                └──────────────────────────────┘
```

The orchestrator picks one of the returned agents, claims an offset on its
behalf (or asks the agent to self-claim), dispatches the work, and lets the
claim's lifecycle take care of exclusive-delivery from there. While the
chosen agent is holding the claim, `find_idle` excludes it — the next call
returns a different agent.

## Filter semantics

- **`role`** (optional) — exact equality against `metadata.role`. Heartbeat
  emitters set this; default is `"listener"`. Use to scope a dispatch to a
  particular kind of worker (e.g. `"claude-code"`, `"build-runner"`).
- **`capabilities`** (optional) — AND-subset match. Every requested
  capability must appear in the agent's `metadata.capabilities` (comma-
  separated). An agent that doesn't emit the field is treated as the empty
  set and never matches a non-empty filter. This is backward-compatible
  with workers from before T-2045.
- **`limit`** (optional) — cap result count after sorting by freshness
  (latest heartbeat first). Default unlimited.

## Runnable example: end-to-end on a local hub

This sequence reproduces the T-2045 live smoke. Assumes the local hub is up
and the CLI binary is on `$PATH`.

```bash
# 1. Become discoverable with a capability tag
TERMLINK_CAPABILITIES="claude-code,rust" \
  bash scripts/be-reachable.sh start --agent-id "worker-1"

# 2. Confirm worker-1 is now LIVE and idle
termlink agent find-idle --json
# {
#   "idle": [
#     { "agent_id": "worker-1",
#       "capabilities": ["claude-code", "rust"],
#       "last_heartbeat_ms": 1780924555851,
#       "role": "listener" }
#   ],
#   "ok": true
# }

# 3. AND-filter on capability
termlink agent find-idle --capability rust --json   # ← matches
termlink agent find-idle --capability python --json # ← empty (none advertised)

# 4. Hand work to worker-1 and watch it disappear from the roster
termlink channel claim my-work-topic 0 --claimer worker-1 --ttl-ms 60000
termlink agent find-idle --json   # ← worker-1 excluded (holding a claim)
termlink channel release <claim_id> --claimer worker-1 --ack
termlink agent find-idle --json   # ← worker-1 returns to the roster

# 5. Clean up
bash scripts/be-reachable.sh stop
```

## Heartbeat schema (producer side)

To appear in `find-idle`, an agent must post heartbeats to `agent-presence`
with the conventional metadata. The shipped emitter (T-1832
`scripts/listener-heartbeat.sh`) handles all of this:

| Metadata key | Source | Purpose |
|---|---|---|
| `agent_id` | `--agent-id` | The handle returned by `find_idle` |
| `role` | `--role` (default `listener`) | Filters via `--role` |
| `capabilities` | `--capabilities` / `$TERMLINK_CAPABILITIES` | Filters via `--capability` (T-2045) |
| `started_at`, `interval_secs`, `host`, … | auto | Informational |

`metadata.capabilities` is omitted entirely when not provided — older
emitters continue to work, but will never match a non-empty
`--capability` filter.

## What this primitive is NOT

Per the T-2020 inception (§5.4 "What's NOT in this primitive"):

- **Not cross-hub.** Each hub answers for its own presence + claims tables.
  Fleet-wide finding is the orchestrator's job — walk `hubs.toml` and call
  `find_idle` per hub.
- **Not a scheduler.** It returns a roster; it doesn't pick. The caller
  decides which idle agent to dispatch to (round-robin, freshness,
  capability rank — orchestrator policy).
- **Not a reservation.** Two parallel callers can each see the same idle
  agent. Reservation is `channel.claim` (T-2019). `find_idle` is the
  "candidate list" step before the claim.
- **No persistence of its own.** Restarting the hub doesn't lose
  "registrations" — there are none. The next heartbeat from a worker
  re-establishes it in the next 30 s.

## Related primitives + tasks

- T-2019 — `channel.claim` / `renew` / `release` (the exclusive-delivery
  ledger that `find_idle` anti-joins against).
- T-2046 — `channel.transfer_claim` (next foundation primitive; lets an
  orchestrator re-assign a held claim to a different idle agent).
- T-1832 — `scripts/listener-heartbeat.sh` (the producer side).
- T-1841 — `/be-reachable` skill (one-keystroke session presence).
- T-1833 — `termlink agent listeners` (read presence directly; `find_idle`
  is presence + claims anti-join on top of it).
- T-2020 — inception docs/reports/T-2020-idle-busy-registry-inception.md.
- T-2018 — arc-parallel-substrate ADR; the umbrella for the substrate.
