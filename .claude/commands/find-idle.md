# /find-idle — list currently-idle agents available for dispatch (T-2092 skill-layer wrap)

Wraps `termlink agent find-idle` (substrate primitive #2 DISPATCH,
shipped under T-2020 / T-2045). Answers "who's free right now to take
work?" — the canonical orchestrator question. Read-only, no state
mutation, no auth.

`/find-idle` is the **DISPATCH** counterpart to `/peers` in the
six-corner skill set:

- **/peers** (T-1859) — "who's around at all?" (LIVE listeners, idle or busy)
- **/find-idle** (this skill) — "who's around AND idle right now?"
- **/agent-handoff** (T-1431) — "deliver this work to a specific agent"

Pair pattern: `/find-idle --capability X` → pick one → `/agent-handoff`
with that `<agent_id>` and your task ID. Mirrors the
T-2046 substrate primitive #3 claim-transfer pattern (find idle →
hand them the work).

**Invocation:**

| Form | Action |
|------|--------|
| `/find-idle` | All currently-idle agents on the local hub |
| `/find-idle --role claude-code` | Only agents with `metadata.role == claude-code` |
| `/find-idle --capability rust` | Only agents advertising `rust` (AND-subset semantics; repeat for multi-cap) |
| `/find-idle --capability rust --capability deploy` | Agents advertising BOTH `rust` AND `deploy` |
| `/find-idle --limit 3` | Cap result list at N |
| `/find-idle --json` | Machine-readable envelope (passes through to verb) |

`find-idle` semantics (per docs/operations/agent-find-idle.md):

- **Local-hub only** by design — cross-hub fan-out is an orchestrator
  responsibility (no auto-fleet sweep here).
- **AND-subset capability match** — every requested cap must appear in
  the agent's `metadata.capabilities`. Missing field = empty set
  (backward-compat with workers that don't emit it).
- **LIVE = heartbeat newer than 60s.** Anything older is not idle, it's
  unreachable.
- **Excludes agents holding any active claim** — substrate primitive
  #1 (T-2019) anti-join. An agent claiming work is by definition not
  idle.
- **No reservation.** Two callers can see the same idle agent. Use
  `channel.claim` to reserve.

## Step 1: Pre-flight

Run:

```
termlink agent find-idle --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
find-idle: `termlink` CLI not on PATH or substrate primitive #2 (DISPATCH)
not available in this build. Ensure you're on a version >= 0.10.x with
T-2045 shipped (run `termlink --version` to verify).
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Pass through verbatim — the
underlying verb validates and errors with usage on malformed input.
Strip nothing; do NOT translate aliases.

Examples:

| User typed | Command emitted |
|------------|-----------------|
| `/find-idle` | `termlink agent find-idle` |
| `/find-idle --role claude-code` | `termlink agent find-idle --role claude-code` |
| `/find-idle --capability rust` | `termlink agent find-idle --capability rust` |
| `/find-idle --capability rust --capability deploy --limit 5` | passthrough |
| `/find-idle --json` | `termlink agent find-idle --json` |

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Render the result

For **default human-format** output:

- Pass the verb's stdout through verbatim (it's a one-line-per-agent
  table per T-2045).
- If exit code is 0 and the table is empty (no agents returned), the
  hub has zero idle agents — see Step 5.
- If exit code is non-zero, surface stderr and stop.

For `--json` mode: pass the JSON envelope through verbatim. Do not
re-render — callers piping/parsing rely on the substrate verb's schema
(`{ok, idle: [{agent_id, last_heartbeat_ms, role, capabilities}, ...]}`).

## Step 5: Empty-result hint

If exit 0 and zero idle agents:

```
No idle agents found.

Possible causes:
- All LIVE listeners are currently holding active claims (busy).
- No LIVE listeners at all on this hub. Check with: /peers
- Capability filter is too narrow. Try: /find-idle (no filter)
- This hub has no presence at all. Check: termlink fleet doctor

To make YOURSELF discoverable as idle here:
  /be-reachable --capabilities "your,caps,csv"
```

Never silent on empty.

## Step 6: Per-agent handoff hints

For non-empty results in human-format mode (NOT json mode), after the
verb's table, append a "Reach them with:" section listing one
`/agent-handoff` invocation per idle agent:

```
Reach them with:
  /agent-handoff <agent_id> <T-XXX> "<msg>"

Idle agents:
  /agent-handoff claude-alpha T-2092 "your message"
  /agent-handoff worker-bravo T-2092 "your message"
```

Use the current focus task (read `.context/working/focus.yaml`
`current_task:`) for `<T-XXX>` placeholder; fall back to `T-XXX`
literal if no focus.

Skip this section in `--json` mode (machine output stays pure).

## Rules

- **Read-only by contract.** Never claim, never modify presence.
- **Local-hub only.** Do NOT fan out to hubs.toml — that's the
  orchestrator's job, and find-idle is intentionally scoped to one
  hub at a time (see ADR §6 #2).
- **Never auto-claim.** Surfacing an idle agent is NOT a reservation.
  The user (or a downstream verb) decides whether to claim.
- **No `AskUserQuestion`** — just run and report.
- **Pair with /peers for full context.** When operating in a strange
  environment, `/peers --all` shows the full presence picture and
  `/find-idle` filters to the idle subset.

## Common patterns

**Cold-start dispatch session:**

```
/peers                                 # who's around?
/find-idle --capability rust           # who's idle AND can do rust?
/agent-handoff claude-alpha T-2092 "do thing X"   # hand off
```

**Find-idle then watch-loop dispatch (orchestrator pattern):**

```
termlink agent find-idle --watch 30 --notify /usr/local/bin/dispatch-on-idle.sh
```

The `--watch` form lives at the CLI tier (T-2078..T-2082); the skill
intentionally doesn't wrap it (long-running watch loops sit awkwardly
inside a slash-command).

## Related

- T-2020 / T-2045 — the underlying `termlink agent find-idle` verb.
- T-2018 — arc-parallel-substrate ADR; this skill operationalizes #2.
- T-2019 — `channel.claim` (the next-step "actually reserve" verb).
- T-2046 — `channel.claim-transfer` (atomic hand-off after find-idle).
- T-1859 / `/peers` — sibling LIST verb (broader presence view).
- T-2091 — `--filter-capability` on `/peers` (sibling capability-filter).
- T-1841 / `/be-reachable` — the SELF-advertise counterpart (set
  `--capabilities` to be findable here).
- T-1431 / `/agent-handoff` — the verb this skill feeds into.
- docs/operations/agent-find-idle.md — the master recipe doc.
- PL-187 (verb-stack pattern rung 6: ephemeral session skills)
