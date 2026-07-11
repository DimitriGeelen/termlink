# fleet-rearm-wakers — roll new push-waker code onto running agents (T-2404)

`scripts/fleet-rearm-wakers.sh` restarts an agent's **push-waker daemon** onto the
current `be-reachable-pushwaker.sh` code **without relaunching that agent's claude
REPL**.

## The problem it solves

The push-waker (`be-reachable-pushwaker.sh`, armed by `/be-reachable`) is a
long-running daemon that subscribes to the hub's push rails (`inbox.queued` +
`dm.queued`) and rings the agent's PTY when a message arrives. When the waker's
**code** changes — e.g. the T-2402 Stage-3 idle-gating fix (probe-for-READY,
defer-while-busy, never blind-inject) — an agent that is already running keeps the
**old code in memory** until its waker process restarts.

The blunt way to pick up new code is to relaunch the whole claude REPL. That is
**destructive**: it kills the agent's live session. For a code change that only
touches the comms daemon, that is far too heavy.

## The surgical model (why it is zero-outage)

`/be-reachable` spawns two independent things:

| Process | Role | Restarting it costs |
|---|---|---|
| heartbeat / listener (`pid` in state) | advertises LIVE presence on `agent-presence` | **presence drop** — agent goes dark on the fleet |
| push-waker (`pushwaker_pid` in state) | rings the PTY on a push event | only push-wake; poll-floor still delivers |

The push-waker is spawned as a `setsid` **process-group leader** (`pgid == pid`),
separate from the heartbeat pid. So this verb:

1. Reaps **only the waker process-group** (`kill -TERM -<pgid>`, leader-checked) —
   the heartbeat pid is never touched, so **presence never drops**.
2. Respawns the current-code waker with **identical args** reconstructed from the
   per-agent state file (`--inbox-id <a> --pty-session <a> --self-fp <fp>`).
3. Faithfully updates `pushwaker_pid` in `~/.termlink/be-reachable-<agent>.state`
   (all other fields preserved), so the agent's own later `/be-reachable stop`
   reaps the new waker correctly.

**Worst case** (respawn fails): the agent keeps its LIVE presence **and** poll-floor
reachability, losing only push-wake. A degradation, never a blackout.

## Staleness is self-updating

There is **no hardcoded epoch**. "Stale" = the running waker's `/proc/<pid>`
start-mtime predates the **current** `be-reachable-pushwaker.sh` file mtime. So the
moment you edit the waker script, every already-running waker becomes "stale" to
this verb automatically. A waker started after the last code edit is a **NOOP**
(unless `--force`).

## Recipes

```bash
# Preview what would change across the whole fleet (kills/spawns nothing):
bash scripts/fleet-rearm-wakers.sh --all --dry-run

# Roll the current waker code to every agent whose waker is stale:
bash scripts/fleet-rearm-wakers.sh --all

# Re-arm one agent:
bash scripts/fleet-rearm-wakers.sh workshop-designer

# Force a re-arm even if the waker looks current (e.g. after a same-second edit):
bash scripts/fleet-rearm-wakers.sh sonnenstall --force
```

Exit code is non-zero if any targeted agent could not be re-armed (missing state,
respawn failure). Each agent is independent — one failure does not abort `--all`.

## What it does NOT do

- It does **not** touch the claude REPL. If an agent needs a REPL relaunch for a
  *different* reason (e.g. a leaked `PROJECT_ROOT` misrouting its framework project —
  see CLAUDE.md §"tl-claude PROJECT_ROOT launch-hygiene" / T-2403), that stays the
  agent's own action. This verb fixes the **doorbell**, not project-gating.
- It does **not** arm an agent that was never reachable — there must be an existing
  `be-reachable-<agent>.state`. Use `/be-reachable start` for first-time arming.

## Verifying a re-arm

```bash
# waker on current code + both rails subscribed:
tail -4 ~/.termlink/be-reachable-<agent>.pushwaker.log   # expect "watching inbox.queued" + "watching dm.queued"
# presence never dropped:
bash scripts/agent-listeners-fleet.sh --json | jq '.listeners[] | select(.agent_id=="<agent>") | .status'   # LIVE
```

## Related

- T-2402 — the Stage-3 idle-gated waker this verb exists to roll out
  (`docs/operations/pushwaker-idle-gating.md`).
- T-2359 — the fleet **binary**-freshness canary (`hub_version` floors). This verb
  is the waker-**code** analogue at the process layer; a stale-waker-code detection
  canary is the natural follow-up (detection to pair with this remediation).
- `/be-reachable` (T-1841) — the per-agent lifecycle this composes with.
