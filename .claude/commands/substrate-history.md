# /substrate-history — retrospective of substrate health flips (T-2209 skill-layer wrap)

Wraps `termlink substrate history` (substrate primitive #11 roll-up
observability arc, shipped under T-2115). Answers the **retrospective**
question the composite `/substrate` digest cannot: "when did substrate
health flip?" — which roll-up field (claim topic count, pressured-hub
count, queue depth, idle count) changed, and when.

Read-only. Walks the audit log `~/.termlink/substrate.log` — the NDJSON
trail written by `termlink substrate status --watch --log <PATH>`
(T-2114). No auth, no network, no state mutation. If the watch loop
never ran with `--log`, the log won't exist and the verb says so.

`/substrate-history` is the **SUBSTRATE-RETROSPECTIVE** verb, completing
the read-side roll-up arc alongside its live sibling:

- **/substrate** (T-2096) — "is the substrate healthy RIGHT NOW?"
  (a parallel composition of `/find-idle` + `/claims` + `/queue-status`
  + `/governor`)
- **/substrate-history** (this skill) — "when did a roll-up metric
  flip?" (forensic walk of the audit log)

Note the asymmetry from the per-primitive histories: `/substrate` is a
*composition* of four base verbs, whereas `substrate history` reads a
single roll-up audit log keyed by `field`. So this skill is the
retrospective for the aggregate health signal, not for any one
primitive — use `/claims-history`, `/queue-history`, etc. to drill into
a specific primitive's events.

**Invocation:**

| Form | Action |
|------|--------|
| `/substrate-history` | Last 7 days of roll-up changes, all fields |
| `/substrate-history --since 30` | Widen the window to 30 days (clamped 1..=365) |
| `/substrate-history --field backpressure_pressured_hubs` | Filter to one roll-up field |
| `/substrate-history --field claim_topic_count` | Filter to claim-topic-count flips |
| `/substrate-history --log <path>` | Read a non-default log location |
| `/substrate-history --json` | Machine-readable envelope (passthrough) |

`substrate history` semantics (per T-2115):

- **Reads the roll-up audit log, not the live substrate.** Data is only
  as complete as the `substrate status --watch --log` session that
  wrote it. No watch session ⇒ empty log ⇒ the verb prints a hint
  (JSON mode returns `{ok, entries:[], ... note:"log file does not
  exist yet"}`).
- **Window default 7 days**, clamped to `1..=365`.
- **Entries are keyed by `field`** — each row records a change to one
  roll-up metric (e.g. `claim_topic_count`, `backpressure_pressured_hubs`).
  `--field <NAME>` filters by exact field match.

## Step 1: Pre-flight

Run:

```
termlink substrate history --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
substrate-history: `termlink` CLI not on PATH, or this build predates
T-2115. Run `termlink --version` to verify; the verb shipped with
substrate primitive #11's roll-up observability arc.
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Pass through verbatim.

Examples:

| User typed | Command emitted |
|------------|-----------------|
| `/substrate-history` | `termlink substrate history` |
| `/substrate-history --since 30` | `termlink substrate history --since 30` |
| `/substrate-history --field backpressure_pressured_hubs` | `termlink substrate history --field backpressure_pressured_hubs` |
| `/substrate-history --field claim_topic_count --since 14 --json` | passthrough |

Empty `$ARGUMENTS` is valid (defaults to last 7 days, all fields).

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Render the result

For **default human-format** output:

- Pass the verb's stdout through verbatim (one row per matching change
  + the summary footer grouped by field).
- If exit code is non-zero, surface stderr and stop.
- If exit 0 with no entries, see Step 5.

For `--json` mode: pass the envelope through verbatim. Shape:
`{ok, entries[], summary{total, per_field, since_days, field_filter, malformed_lines_skipped, log_path}}`.
Do not re-render.

## Step 5: Empty-result hint

If exit 0 and zero entries:

When the log is **missing** (the verb's own hint usually says so):

```
No substrate-history yet — the roll-up audit log has not been written.

The log is populated by a watch session:
  termlink substrate status --watch 30 --log ~/.termlink/substrate.log

Leave that running to capture substrate health flips. For the live
roll-up right now, use:
  /substrate
```

When the log **exists but no entries match** the window/field filter:

```
No substrate roll-up changes in the last <N> day(s)<field clause>.

No tracked roll-up field flipped in the window — substrate health was
stable. Widen with --since, or drop --field to see all fields.
```

Never silent on empty.

## Rules

- **Read-only by contract.** This verb only walks a log file.
- **No `AskUserQuestion`** — just run and report.
- **Roll-up scope, not per-primitive.** To investigate a specific
  primitive's events, use that primitive's own history skill
  (`/claims-history`, `/find-idle-history`, `/queue-history`,
  `/governor-history`).
- **Pair with /substrate for the live view.**

## Common patterns

**"When did things go sideways?" triage:**

```
/substrate                                  # current roll-up health
/substrate-history --since 7                # what flipped in the last week
/substrate-history --field backpressure_pressured_hubs   # narrow to backpressure flips
```

Then drill into the flagged primitive with its own history skill.

**Pipe to scripting:**

```
/substrate-history --json | jq '.summary.per_field'
```

**Start capturing history (lives at the CLI tier):**

```
termlink substrate status --watch 30 --log ~/.termlink/substrate.log
```

## Related

- T-2115 — `substrate history`, the CLI verb this skill wraps.
- T-2114 — `substrate status --watch --log`, the writer that populates
  `~/.termlink/substrate.log`.
- T-2096 / `/substrate` — the live composite-digest sibling skill.
- T-2111 — the substrate roll-up (`substrate status`) primitive #11.
- T-2068 / T-2074 / T-2081 / T-2086 — the per-primitive history verbs
  whose skills (`/governor-history`, `/claims-history`,
  `/find-idle-history`, `/queue-history`) drill into a single primitive.
- T-2209 — this skill (history-verb skill-layer completion arc).
