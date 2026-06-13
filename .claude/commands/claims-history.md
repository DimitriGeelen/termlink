# /claims-history — retrospective of stuck-claim state changes (T-2209 skill-layer wrap)

Wraps `termlink channel claims-history` (substrate primitive #1
channel.claim observability arc, shipped under T-2074). Answers the
**retrospective** question the live `/claims` view cannot: "has this
topic been stuck repeatedly, or is this the first time?"

Read-only. Walks the audit log `~/.termlink/claims.log` — the NDJSON
trail written by `termlink channel claims-summary --watch --log <PATH>`
(T-2073). No auth, no network, no state mutation. If the watch loop
never ran with `--log`, the log won't exist and the verb says so.

`/claims-history` is the **CLAIM-RETROSPECTIVE** verb, completing the
read-side arc alongside its live sibling:

- **/claims** (T-2093) — "what's claimed RIGHT NOW?" (live snapshot)
- **/claims-history** (this skill) — "how often has it been stuck?"
  (forensic walk of the audit log)

Pair pattern: `/claims --all --only-stuck` spots a wedge →
`/claims-history --topic <name>` answers "first time or Nth?" → if
recurring, the topic likely has a structural producer problem, not a
one-off.

**Invocation:**

| Form | Action |
|------|--------|
| `/claims-history` | Last 7 days of stuck-state transitions, all topics |
| `/claims-history --since 30` | Widen the window to 30 days (clamped 1..=365) |
| `/claims-history --topic work-queue` | Filter to one exact topic name |
| `/claims-history --log <path>` | Read a non-default log location |
| `/claims-history --json` | Machine-readable envelope (passthrough) |

`claims-history` semantics (per T-2074):

- **Reads the audit log, not the hub.** The data is only as complete as
  the `claims-summary --watch --log` session that wrote it. No watch
  session ⇒ empty log ⇒ the verb prints a hint pointing back at the
  writer.
- **Window default 7 days**, clamped to `1..=365`.
- **Per-topic aggregate footer** counts `transition` / `new` / `removed`
  events so you can see at a glance which topic flaps most.
- **Event kinds:** `transition` (a topic crossed the stuck/not-stuck
  boundary), `new` (a topic first appeared under `--all`), `removed`
  (a topic went away).

## Step 1: Pre-flight

Run:

```
termlink channel claims-history --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
claims-history: `termlink` CLI not on PATH, or this build predates T-2074.
Run `termlink --version` to verify; the verb shipped with substrate
primitive #1's observability arc.
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Pass through verbatim — the verb
validates and errors with usage on malformed input.

Examples:

| User typed | Command emitted |
|------------|-----------------|
| `/claims-history` | `termlink channel claims-history` |
| `/claims-history --since 30` | `termlink channel claims-history --since 30` |
| `/claims-history --topic work-queue` | `termlink channel claims-history --topic work-queue` |
| `/claims-history --topic work-queue --since 14 --json` | passthrough |

No normalization needed — empty `$ARGUMENTS` is valid (defaults to the
last 7 days across all topics).

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Render the result

For **default human-format** output:

- Pass the verb's stdout through verbatim (one line per matching entry
  + the per-topic aggregate footer).
- If exit code is non-zero, surface stderr and stop.
- If exit 0 with no entries, see Step 5.

For `--json` mode: pass the envelope through verbatim. Shape:
`{ok, entries[], summary{total, per_topic:{<topic>:{transitions, new_events, removed_events}}, since_days, topic_filter, malformed_lines_skipped, log_path}}`.
Do not re-render — callers parsing the JSON rely on the verb's schema.

## Step 5: Empty-result hint

If exit 0 and zero entries:

When the log is **missing** (the verb's own hint usually says so):

```
No claims-history yet — the audit log has not been written.

The log is populated by a watch session:
  termlink channel claims-summary --all --watch 30 --log ~/.termlink/claims.log

Until that runs (typically as an orchestrator-side cron or supervisor),
there is no retrospective to read. For the live picture right now, use:
  /claims --all --only-stuck
```

When the log **exists but no entries match** the window/topic filter:

```
No claims state-changes in the last <N> day(s)<topic clause>.

This is the healthy case — no topic crossed the stuck/not-stuck
boundary in the window. Widen with --since, or drop --topic to see all
topics.
```

Never silent on empty.

## Rules

- **Read-only by contract.** Never claim, release, or modify state.
  This verb only walks a log file.
- **No `AskUserQuestion`** — just run and report.
- **Local audit log only.** There is no fleet-wide history merge; each
  host's log reflects the watch sessions that ran on that host.
- **Pair with /claims for the live view.** History answers "how often";
  `/claims` answers "right now".

## Common patterns

**Flap triage (the canonical use):**

```
/claims --all --only-stuck          # something wedged?
/claims-history --topic work-queue --since 30   # first time or recurring?
```

**Pipe to scripting:**

```
/claims-history --json | jq '.summary.per_topic | to_entries | max_by(.value.transitions)'
```

**Start capturing history (lives at the CLI tier):**

```
termlink channel claims-summary --all --watch 30 --log ~/.termlink/claims.log
```

## Related

- T-2074 — `channel claims-history`, the CLI verb this skill wraps.
- T-2075 — `termlink_channel_claims_history` MCP parity.
- T-2073 — `claims-summary --watch --log`, the writer that populates
  `~/.termlink/claims.log`.
- T-2093 / `/claims` — the live-snapshot sibling skill.
- T-2042 — the underlying `channel claims-summary` verb + "stuck"
  heuristic.
- T-2209 — this skill (history-verb skill-layer completion arc).
- T-2018 — arc-parallel-substrate ADR; this skill operationalizes #1's
  retrospective read at the daily-verb tier.
