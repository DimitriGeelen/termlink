# /claims — list active claims on a topic (T-2093 skill-layer wrap)

Wraps `termlink channel claims-summary` (substrate primitive #1
channel.claim read-side, shipped under T-2019 / T-2042). Answers
"what's claimed on this topic right now?" / "are any claims stuck?" —
the orchestrator's situational-awareness question.

Read-only, no state mutation, no auth. Local-hub by default; `--all`
walks every topic on the local hub.

`/claims` is the **CLAIM-READ** verb in the substrate primitive #1
arc, completing the read-side surface alongside:

- **/find-idle** (T-2092) — "who's idle?" (DISPATCH / substrate #2)
- **/claims** (this skill) — "what's claimed?" (CLAIM-READ / substrate #1)
- **/peers** (T-1859) — "who's around?" (PRESENCE / agent-presence topic)

Pair pattern: `/claims --all --only-stuck` to spot wedges →
`/find-idle` to find a free worker → `/agent-handoff` to dispatch the
recovery.

**Invocation:**

| Form | Action |
|------|--------|
| `/claims <topic>` | List active claims on `<topic>` on the local hub |
| `/claims --all` | Walk every topic on the local hub; render per-topic summary |
| `/claims --all --only-stuck` | Fleet-wide filter — only topics with stuck claims (T-2076) |
| `/claims <topic> --json` | Machine-readable envelope (passthrough to verb) |
| `/claims --all --json` | Multi-topic JSON envelope |

`claims-summary` semantics (per substrate primitive #1 + T-2042 + T-2076):

- **Local-hub only** by ADR §6 design — cross-hub fan-out is an
  orchestrator responsibility.
- **"Stuck" heuristic** (T-2042): `expired_count > 0` OR
  `oldest_active_age_ms > 60_000`. A topic is "stuck" if at least one
  claim has timed out OR the oldest active claim is older than 1
  minute.
- **`--only-stuck`** (T-2076) is presentation-level — the JSON
  envelope still carries fleet-wide `topic_count` + `stuck_count` so
  the operator sees both "1/N stuck" and the raw subset.
- **No reservation.** A `/claims` read is observational. To actually
  reserve work, callers run `termlink channel claim` (or a higher-level
  wrapper).

## Step 1: Pre-flight

Run:

```
termlink channel claims-summary --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
claims: `termlink` CLI not on PATH or substrate primitive #1 (channel.claim)
not available in this build. Ensure you're on a version with T-2019 + T-2042
shipped (run `termlink --version` to verify).
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Pass through verbatim — the
underlying verb validates and errors with usage on malformed input.

Common-case normalization (only when convenient):

- Empty `$ARGUMENTS` → reject with hint pointing to `--all` or a
  topic name. A blind `claims-summary` with no scope returns nothing
  useful.

Examples:

| User typed | Command emitted |
|------------|-----------------|
| `/claims work-queue` | `termlink channel claims-summary work-queue` |
| `/claims --all` | `termlink channel claims-summary --all` |
| `/claims --all --only-stuck` | `termlink channel claims-summary --all --only-stuck` |
| `/claims work-queue --json` | `termlink channel claims-summary work-queue --json` |
| `/claims --all --only-stuck --json` | passthrough |

If `$ARGUMENTS` is empty:

```
claims: scope required.
Usage:
  /claims <topic>            # single-topic view
  /claims --all              # all topics on local hub
  /claims --all --only-stuck # only topics with stuck claims
```

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Render the result

For **default human-format** output:

- Pass the verb's stdout through verbatim (per-topic block with
  active/expired counts + oldest-age).
- If exit code is 0 and the output is empty (no claims to render), see
  Step 5 for the empty-result hint.
- If exit code is non-zero, surface stderr and stop.

For `--json` mode: pass the JSON envelope through verbatim. Do not
re-render — callers piping/parsing rely on the substrate verb's
schema.

## Step 5: Empty-result hint

If exit 0 and zero claims:

For single-topic mode (`/claims <topic>`):

```
No active claims on topic '<topic>'.

Possible causes:
- Topic exists but no workers have claimed work on it yet.
- Topic does not exist on this hub. Check with: termlink topics
- Claims were released or completed and the topic is now idle.

To start a claim cycle:
  termlink channel claim <topic> <unit-id> --owner <self>
```

For `--all` mode:

```
No active claims on any topic on this hub.

The hub is idle — no in-flight work-stealing. This is the steady-state
between dispatches.

To see who's around to take work:
  /find-idle
```

For `--all --only-stuck` mode with non-zero topic_count but zero
stuck:

```
All N topic(s) healthy — zero stuck claims.

(A claim is "stuck" if it has an expired lease OR is older than 60s.)
```

Never silent on empty.

## Step 6: Stuck-claim diagnostic hint

For human-format mode (NOT json), after the verb's output, if any
topic is reported as stuck, append a diagnostic ladder:

```
Stuck-claim recovery options:
- termlink channel claim-force-release <claim-id> --by <operator>   # Tier-0 ownership bypass
- termlink channel claim-transfer <claim-id> --to-owner <new> --by <current>  # cooperative handoff (T-2046)
- termlink channel claims-summary --all --watch 30 --notify <cmd>   # real-time stuck-state alerts (T-2072)
```

Skip this section in `--json` mode (machine output stays pure).

## Rules

- **Read-only by contract.** Never claim, never force-release, never
  modify claim state.
- **Local-hub only.** Do NOT fan out to hubs.toml — that's the
  orchestrator's job (see ADR §6 design rationale for claim).
- **No `AskUserQuestion`** — just run and report.
- **Pair with /find-idle for full work-stealing context.** `/claims
  --all` shows what's already claimed, `/find-idle` shows who's free
  to claim more.
- **The watch/notify/log/history forms stay at the CLI tier.**
  T-2072..T-2075 give long-running monitor/audit/retrospective surfaces
  for orchestrator scripts. `/claims` is the one-shot daily verb.

## Common patterns

**Cold-start orchestrator check:**

```
/claims --all --only-stuck         # any topics wedged?
/find-idle                          # who's free to take recovery?
/agent-handoff <peer> T-XXX "fix"   # dispatch
```

**Topic-scoped triage:**

```
/claims work-queue                  # see what's in flight on a known topic
```

**Pipe to scripting:**

```
/claims --all --only-stuck --json | jq '.summary.stuck_count'
```

**Long-running stuck-state alerting (lives at CLI tier):**

```
termlink channel claims-summary --all --watch 30 --notify /usr/local/bin/page-on-stuck.sh --log ~/.termlink/claims.log
```

The watch/notify/log forms are T-2072 / T-2073; the retrospective
read of `~/.termlink/claims.log` is T-2074's `channel claims-history`.

## Related

- T-2019 — substrate primitive #1 channel.claim (the underlying
  reservation verb this skill reads).
- T-2042 — the underlying `termlink channel claims-summary` verb.
- T-2046 — `channel claim-transfer` (cooperative ownership handoff).
- T-2072 / T-2073 — the `--watch --notify --log` CLI-tier forms.
- T-2074 — `channel claims-history` retrospective verb.
- T-2075 — `termlink_channel_claims_history` MCP parity.
- T-2076 — `--only-stuck` presentation-level filter.
- T-2092 / `/find-idle` — sibling daily-verb skill for substrate #2.
- T-2018 — arc-parallel-substrate ADR; this skill operationalizes #1
  read-side at the daily-verb tier.
- PL-187 — verb-stack pattern rung 6 (ephemeral session skills).
