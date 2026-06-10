# /substrate — single-shot substrate cold-start digest (T-2096 composition)

The **"is the substrate healthy and what's it doing right now?"** verb.
Composes the four substrate-read daily verbs into one unified
situational-awareness view so an operator landing fresh can answer the
question in one keystroke instead of four.

Read-only by composition — never writes state, no auth side-effects.
Parallel-by-default: total latency = max(four reads), not sum-of-four.
Graceful degradation: a failed sub-query renders as one stderr line, not
a hard stop.

This is the **substrate-side companion** to T-1860 `/pulse` (which
composes the conversation arc — peers + recent-chat). Same design
pattern, different domain.

## What it composes

| Skill | Substrate primitive | Question |
|-------|---------------------|----------|
| `/find-idle` (T-2092) | #2 DISPATCH | Who's free to take work? |
| `/claims --all --only-stuck` (T-2093) | #1 CLAIM | Any wedged claims? |
| `/queue-status` (T-2094) | #5 RESILIENCE | Is my queue draining? |
| `/governor --only-pressured` (T-2095) | #10 BACKPRESSURE | Any hub pressured? |

The four answer orthogonal questions; together they form a full
substrate-state snapshot.

**Invocation:**

| Form | Action |
|------|--------|
| `/substrate` | Default: four-section snapshot, human-format |
| `/substrate --json` | Merged JSON envelope (four sub-envelopes nested) |

## Step 1: Pre-flight

Run:

```
termlink agent find-idle --help >/dev/null 2>&1 && \
termlink channel claims-summary --help >/dev/null 2>&1 && \
termlink channel queue-status --help >/dev/null 2>&1 && \
termlink fleet governor-status --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
substrate: one or more substrate-read verbs unavailable in this build.
Required: T-2045 (find-idle), T-2042 (claims-summary), T-2051 (queue-status),
T-2048 (governor-status). Run `termlink --version` and upgrade if needed.
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Normalize:

- Empty → defaults (human-format, four sections)
- `--json` → emit merged envelope (no sub-section rendering)

No other flags. Sub-flags don't pass through — this is a digest, not
a flag-forwarding wrapper. Operators wanting fine control run the
individual `/find-idle` / `/claims` / `/queue-status` / `/governor`
skills directly.

## Step 3: Run all four verbs in parallel

Execute concurrently via Bash. A timeout or failure in one MUST NOT
block the other three:

```
{
  TMPDIR=$(mktemp -d)
  termlink agent find-idle --json > "$TMPDIR/find-idle.json" 2> "$TMPDIR/find-idle.err" &
  PID_FI=$!
  termlink channel claims-summary --all --only-stuck --json > "$TMPDIR/claims.json" 2> "$TMPDIR/claims.err" &
  PID_CL=$!
  termlink channel queue-status --json > "$TMPDIR/queue.json" 2> "$TMPDIR/queue.err" &
  PID_QU=$!
  termlink fleet governor-status --only-pressured --json > "$TMPDIR/gov.json" 2> "$TMPDIR/gov.err" &
  PID_GV=$!
  wait "$PID_FI" "$PID_CL" "$PID_QU" "$PID_GV"
}
```

Capture each verb's exit code separately for graceful degradation.

## Step 4: Render the result

### Default (human-format)

Render a four-section block:

```
═══ substrate snapshot ═══

DISPATCH (substrate #2 — who's free to take work?):
  <ID>  role=<R>  last_heartbeat_ms=<N>  capabilities=<csv>
  ... (per idle agent; "no idle agents (see /find-idle for diagnostic)" if zero)

CLAIM (substrate #1 — any stuck claims?):
  <topic>  active=<N>  expired=<M>  oldest_age_ms=<X>
  ... (per stuck topic; "All N topics healthy (0/N stuck)" if zero — affirmative)

RESILIENCE (substrate #5 — is my queue draining?):
  pending=<N>  oldest_age=<Mms>  queue=<path>
  ... (one-line summary; appended state hint: steady / draining / blipped)

BACKPRESSURE (substrate #10 — any hub pressured?):
  <hub>  conn=A/B  cap_hits=N  rate_hits=M  dedupe_hits=K  cv_overflow=V
  ... (per pressured hub; "All N hubs healthy (0/N pressured)" if zero)
```

The BACKPRESSURE row's `cv_overflow=V` segment (T-2110/T-2118/T-2119)
surfaces broadcast-with-replay (substrate #9) pressure: a non-zero
value means a producer is mis-emitting `cv_key` (e.g. timestamp
instead of stable id) and silently saturating the per-topic cv_index
cap. The cv-overflow predicate fires `--only-pressured` (T-2118) so
this skill's filtered view catches it. Pre-T-2110 hubs render as
`cv_overflow=n/a` (NOT `0`).

Read each section from the corresponding sub-envelope:

- DISPATCH: `.idle[]` from find-idle.json
- CLAIM: `.topics[]` filtered by `summary.only_stuck=true` from claims.json
- RESILIENCE: top-level `pending`/`oldest_age_ms`/`queue_path` from queue.json
- BACKPRESSURE: `.hubs[]` from gov.json (already filtered by `--only-pressured`)

### Substrate-healthy path (all four zero/clean)

After the four-section render, append:

```
substrate steady-state: dispatch=0 idle, 0 stuck claims, queue drained, 0 hubs pressured.

This is the green-light. Substrate is healthy and quiet — no in-flight
work needing attention. If the fleet should be busier, check:
  /peers --all      # is anyone actually around?
  /pulse            # what's the conversation arc say?
```

### Partial-success path (one or more verbs failed)

Render the successful sections + a stderr note per failure:

```
(governor section unavailable: <first line of err file>)
```

Do NOT silently drop the failed section.

### --json mode

Emit one merged envelope, no rendering:

```json
{
  "ok": true,
  "ts": "<RFC3339>",
  "dispatch":     { ... passthrough from find-idle --json ... },
  "claim":        { ... passthrough from claims-summary --all --only-stuck --json ... },
  "resilience":   { ... passthrough from queue-status --json ... },
  "backpressure": { ... passthrough from governor-status --only-pressured --json ... }
}
```

If any sub-verb errored, include its `ok: false` in its sub-envelope.
Do NOT synthesize an `ok: true` over a failed sub-section.

## Step 5: Suggest next actions

After the digest, contextual hints based on what was found:

If DISPATCH shows idle agents AND CLAIM/RESILIENCE/BACKPRESSURE clean:
```
Ready state: idle workers available, no blockers.
Engage: /agent-handoff <peer> <T-XXX> "..." to dispatch work.
```

If RESILIENCE shows queue-pending:
```
Tip: your local queue is buffering. /queue-status for details.
```

If BACKPRESSURE shows pressured hubs:
```
Tip: hub backpressure detected. /governor for per-hub details.
Continuous monitor: termlink fleet governor-status --watch 30

If cv_overflow > 0 (T-2118 fires --only-pressured on this):
a producer is mis-emitting cv_key. Run `termlink channel cv-keys <topic>`
to identify the saturating topic, then fix the producer. See
docs/operations/substrate-governor.md § page-on-cv-overflow.sh recipe
for an automated --notify hook (T-2119).
```

If CLAIM shows stuck topics:
```
Tip: stuck claims detected. /claims --all --only-stuck for details.
Recovery: claim-transfer (cooperative) or claim-force-release (Tier-0).
```

The all-healthy case is already handled in Step 4.

## Rules

- **Read-only by composition.** Never post, never modify substrate state.
- **Parallel-by-default.** The four reads MUST overlap — total latency
  must equal the slower of the four, not their sum.
- **Graceful degradation.** Partial result is better than no result. A
  failed sub-query is shown as one stderr line, not a hard stop.
- **No `AskUserQuestion`** — just run and report.
- **Don't compose with conversation-arc verbs.** `/pulse` handles
  presence + recent-chat. `/substrate` handles the four substrate-read
  primitives. Keep the domains separate so each digest reads cleanly.

## Common patterns

**Cold-start substrate check:**

```
/substrate
```

**Pipe for scripting / dashboarding:**

```
/substrate --json | jq '.backpressure.summary.hubs_at_capacity'
```

**Pair with /pulse for full cold-start picture:**

```
/pulse        # conversation arc state
/substrate    # substrate state
```

Two keystrokes → full operational picture across both domains.

## Related

- T-2018 — arc-parallel-substrate ADR; this skill composes the
  read-side of the four most-relevant primitives at the daily-digest
  tier.
- T-2092 / `/find-idle` — DISPATCH read sub-verb.
- T-2093 / `/claims` — CLAIM read sub-verb.
- T-2094 / `/queue-status` — RESILIENCE read sub-verb.
- T-2095 / `/governor` — BACKPRESSURE read sub-verb.
- T-2110 — cv_index telemetry surfaced via the BACKPRESSURE rollup
  (closes substrate §6 #9↔#10 cross-reference at the counter level).
- T-2118 — `--only-pressured` predicate fires on `cv_index_overflow_total > 0`
  so this skill's filtered view catches producer-side `cv_key` bugs.
- T-2119 — cv_overflow deltas in watch / notify / log / history surfaces.
- T-1860 / `/pulse` — the conversation-arc analog (peers + recent-chat
  parallel composition). Same design pattern.
- PL-187 — verb-stack pattern rung 6 (ephemeral session integrators).
