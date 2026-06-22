# agent-presence retention reset (T-2059, rewritten T-2247)

Operator runbook to shrink `agent-presence` (or any high-rate topic) and
change its retention policy off `Retention::Forever`. Originally filed as
the operator-facing follow-through of the T-2057 retention audit (which
surfaced 13,443 envelopes on a single topic as the largest in-the-wild
T-1991 reproduction surface).

> **Rewritten 2026-06-22 (T-2247).** The arc-substrate-fitness verticals
> shipped the missing verbs this runbook used to work around:
> `channel.set_retention` (T-2244 / R2a), `channel.sweep` +
> `Retention::LatestPerCvKey` (T-2245 / R2b), and MCP parity for both
> (T-2246 / R2c). The old `sqlite3 UPDATE topics` + hub-stop dance is now
> **legacy** — see §6. Use §3 (the safe online path) by default.

## §1 When to use

Run this when EITHER:

- `termlink channel list --json | jq '.topics[] | select(.name=="agent-presence")'`
  shows `count` in the thousands AND `retention.kind == "forever"`.
- `fw doctor` or `fleet doctor` surfaces topic-growth pressure.
- You're following up on a T-2058 warn-log: a `tracing::warn!` fired at
  create time naming a high-rate pattern with `Forever`.
- `/governor` reports `cv_overflow > 0` or you otherwise suspect
  `agent-presence` is unbounded.

Do NOT run this on operator-named durable record topics — these are
intentionally `Forever` and shrinking them loses real history. The
audit's §5 explicitly excluded `channel:learnings`, `policy-decisions`,
`framework:pickup`, and `broadcast:global` from the high-rate-pattern
list for this reason.

## §2 Understanding the model (two verbs, two jobs)

TermLink now has a first-class, **online, race-free** retention-management
surface. There are two distinct steps — persisting a policy and enforcing
it — because the bus runs **no background sweep thread** (T-1155 design:
enforcement is explicit, never implicit).

| Verb | What it does | Hot path? |
|---|---|---|
| `channel set-retention <topic> --retention <policy>` | PERSISTS a new policy on an existing topic. Does **not** delete anything by itself. | no |
| `channel sweep <topic>` | ENFORCES the topic's current policy NOW, pruning out-of-policy records. The trigger. | no |

Both go through the running hub (no `sqlite3` edits, no hub-stop, no race).
Both have MCP twins (`termlink_channel_set_retention`,
`termlink_channel_sweep`) for agent callers.

**Retention policies** (the `--retention` value):

| Policy | Keeps | Right for |
|---|---|---|
| `forever` | everything | operator-named durable records |
| `days:N` | records newer than N days | time-windowed logs |
| `messages:N` | the most-recent N records (whole topic) | flat-capped streams |
| `latest` | the single most-recent record (whole topic) | single-value-per-topic state |
| `latest-per-cv-key` | the most-recent record **per distinct `metadata.cv_key`** | **`agent-presence`** and any current-state-per-agent topic |

### Why `latest-per-cv-key` is the proper T-1991 fix for presence

`agent-presence` carries one heartbeat per agent every ~30s, each tagged
`metadata.cv_key=<agent_id>` (T-2107). The live, useful state is "one
record per agent." Under `messages:N` you must guess N and you still grow
with heartbeat *rate*; under `latest` you'd collapse the whole topic to a
single record and lose every agent but one. `latest-per-cv-key` keeps
exactly one record per agent, so the record count converges to the agent
**count** — not the heartbeat count. That is the only mode that closes the
T-1991 *agent-count* scaling problem. Records with no `cv_key` are retained
(never silently dropped).

## §3 The safe path (recommended) — set-retention + sweep

For `agent-presence`, switch to per-key compaction and enforce it once:

```sh
TOPIC=agent-presence

# 1. Inspect current state.
termlink channel list --json \
  | jq '.topics[] | select(.name=="'"$TOPIC"'") | {name, count, retention}'

# 2. Persist the per-key compaction policy (online, no hub stop).
termlink channel set-retention "$TOPIC" --retention latest-per-cv-key

# 3. Enforce it NOW — prunes all but the latest record per agent.
termlink channel sweep "$TOPIC"

# 4. Verify (see §5).
```

For a time-windowed topic instead, substitute step 2:

```sh
termlink channel set-retention some-log-topic --retention days:2
termlink channel sweep some-log-topic
```

That's the whole operation. No `socat`, no `sqlite3`, no downtime.

## §4 Keeping it bounded — periodic sweep (cron)

`set-retention` persists the policy; `sweep` enforces it at a point in time.
Because the bus runs no background thread, the topic **re-grows** between
sweeps. To keep `agent-presence` permanently bounded, run `sweep` on a
schedule. A daily cron is plenty for presence (per-key compaction means a
single sweep collapses a full day of beats back to one-per-agent):

```cron
# /etc/cron.d/termlink-presence-sweep  (or a user crontab)
# Compact agent-presence to one record per agent, daily at 04:17.
17 4 * * *  termlink channel sweep agent-presence >> ~/.termlink/presence-sweep.log 2>&1
```

Notes:
- The policy only needs to be set ONCE (`set-retention`); the cron just
  re-runs `sweep`. If the hub's meta survives restarts (it should — see
  `docs/operations/termlink-hub-runtime-migration.md`), the policy persists
  and the cron keeps the topic flat indefinitely.
- Sweep is idempotent: a sweep on an already-compact topic prunes 0 and
  costs only an O(live-set) scan.
- To sweep several high-rate topics, add one line each, or loop:
  `for t in agent-presence agent-listeners-foo; do termlink channel sweep "$t"; done`.
- See `docs/operations/substrate-cron-recipes.md` for the canary/cron
  conventions used across the substrate safety set.

## §5 Verification

After §3 (and/or a cron sweep), confirm the new state:

```sh
termlink channel list --json \
  | jq '.topics[] | select(.name=="agent-presence")'
```

Expected: `retention.kind == "latest_per_cv_key"`, and `count` near the
number of live agents (NOT the thousands it was before). Re-run after the
next heartbeat cycle; `channel list` reads live from the metadata table.

You can also confirm the per-key view directly:

```sh
termlink channel cv-keys agent-presence   # one entry per advertising agent
```

If `count` did not drop after a sweep: confirm the policy actually
persisted (`retention.kind` in the list output). If it still reads
`forever`, the `set-retention` call targeted a different hub — check
`--hub` / `TERMLINK_RUNTIME_DIR`.

If `retention.kind` did not change: you may be talking to a hub binary
that predates R2a/R2b. Confirm with `termlink channel set-retention --help`
showing `latest-per-cv-key`, and that the hub was restarted onto the new
binary (see `/preflight` Check 5).

## §6 Legacy fallback — sqlite3 metadata edit (deprecated)

> **Deprecated as of T-2247.** Use §3 instead. This path bypasses the
> running hub's invariants and risks racing an in-flight write; it exists
> only as an emergency fallback for a hub too old to have the §3 verbs, or
> when the hub process is unavailable but its meta DB is reachable.

The pre-R2a workaround changed the policy by editing the metadata table
directly and shrank history with `channel.trim`:

```sh
# EMERGENCY/LEGACY ONLY — prefer `termlink channel set-retention` (§3).
TOPIC=agent-presence
KEEP=200
DB="${TERMLINK_RUNTIME_DIR:-/var/lib/termlink}/meta.sqlite"
# Stop the hub first for any non-presence topic to avoid a write race.
sqlite3 "$DB" \
  "UPDATE topics SET retention_kind='messages', retention_value=$KEEP WHERE name='$TOPIC';"
# Then trim existing history via the hub socket (channel.trim has no CLI verb):
CURRENT=$(termlink channel list --json | jq -r ".topics[] | select(.name==\"$TOPIC\") | .count")
BEFORE=$((CURRENT - KEEP))
if [ "$BEFORE" -ge 1 ]; then
  SOCK=$(termlink hub status --json | jq -r .socket)
  printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"channel.trim","params":{"topic":"'"$TOPIC"'","before_offset":'"$BEFORE"'}}' \
    | socat - "UNIX-CONNECT:$SOCK"
fi
```

Note this path cannot express `latest-per-cv-key` (it predates the mode);
the best it achieves is a flat `messages:N` cap. Prefer §3.

## §7 Related

- T-2244 (R2a) — `channel.set_retention` RPC + CLI (change policy online).
- T-2245 (R2b) — `channel.sweep` RPC + CLI + `Retention::LatestPerCvKey`
  (the trigger + the proper presence-compaction mode).
- T-2246 (R2c) — MCP parity (`termlink_channel_set_retention`,
  `termlink_channel_sweep`) for agent callers.
- T-2057 — Track A retention audit (this runbook's parent).
- T-2058 — high-rate-pattern warn-log at create time (structural prevention).
- T-1991 — original silent-growth incident on `agent-presence`.
- G-058 — the gap this runbook closes for one specific topic.
- `docs/architecture/parallel-execution-substrate.md` §6 #10 — the
  budget/retention primitive framing (T-2028 inception).
- `docs/operations/substrate-governor.md` — operator surface for related
  backpressure telemetry (cv_overflow).
- `docs/operations/substrate-broadcast-with-replay.md` — `cv_key` / cv_index
  background (why per-key compaction is the right model for presence).
