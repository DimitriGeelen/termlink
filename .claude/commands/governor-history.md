# /governor-history — retrospective of hub backpressure events (T-2209 skill-layer wrap)

Wraps `termlink fleet governor-history` (substrate primitive #10
BACKPRESSURE observability arc, shipped under T-2068). Answers the
**retrospective** question the live `/governor` view cannot: "has this
hub been backpressured recently?" / "how many rate-limit or
at-capacity hits across the fleet this week?"

Read-only. Walks the audit log `~/.termlink/governor.log` — the NDJSON
trail written by `termlink fleet governor-status --watch --log <PATH>`
(T-2066). No auth, no network, no state mutation. If the watch loop
never ran with `--log`, the log won't exist and the verb says so.

`/governor-history` is the **BACKPRESSURE-RETROSPECTIVE** verb,
completing the read-side arc alongside its live sibling:

- **/governor** (T-2095) — "is the hub being refused RIGHT NOW?"
  (live snapshot)
- **/governor-history** (this skill) — "has it been backpressured
  before?" (forensic walk of the audit log)

Pair pattern: `/governor --only-pressured` flags a hub at capacity →
`/governor-history --hub <name>` answers "is this a recurring squeeze
(needs a higher `TERMLINK_MAX_CONNECTIONS`) or a one-off spike?"

**Invocation:**

| Form | Action |
|------|--------|
| `/governor-history` | Last 7 days of backpressure events, all hubs |
| `/governor-history --since 30` | Widen the window to 30 days (clamped 1..=365) |
| `/governor-history --hub ring20-management` | Filter to one hub profile name |
| `/governor-history --log <path>` | Read a non-default log location |
| `/governor-history --json` | Machine-readable envelope (passthrough) |

`governor-history` semantics (per T-2068):

- **Reads the audit log, not the hubs.** Data is only as complete as
  the `governor-status --watch --log` session that wrote it. No watch
  session ⇒ empty log ⇒ the verb prints a hint.
- **Window default 7 days**, clamped to `1..=365`.
- **Event kinds:** `transition` (a hub's counters changed across a
  cycle), `new` (a hub first appeared), `removed` (a hub went away).
  Each line carries the before→after counters with `(+delta)` for
  cap_hits / rate_hits / dedupe_hits / cv_overflow.
- **Per-hub aggregate footer** sums `cap_hits` / `rate_hits` /
  `dedupe_hits` / `cv_overflow` deltas over the window — a fast read
  on which hub took the most pressure.

## Step 1: Pre-flight

Run:

```
termlink fleet governor-history --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
governor-history: `termlink` CLI not on PATH, or this build predates
T-2068. Run `termlink --version` to verify; the verb shipped with
substrate primitive #10's observability arc.
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Pass through verbatim.

Examples:

| User typed | Command emitted |
|------------|-----------------|
| `/governor-history` | `termlink fleet governor-history` |
| `/governor-history --since 30` | `termlink fleet governor-history --since 30` |
| `/governor-history --hub ring20-management` | `termlink fleet governor-history --hub ring20-management` |
| `/governor-history --hub ring20-management --since 14 --json` | passthrough |

Empty `$ARGUMENTS` is valid (defaults to last 7 days, all hubs).

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Render the result

For **default human-format** output:

- Pass the verb's stdout through verbatim (one line per matching entry
  + per-hub aggregate footer).
- If exit code is non-zero, surface stderr and stop.
- If exit 0 with no entries, see Step 5.

For `--json` mode: pass the envelope through verbatim. Shape:
`{ok, entries[], summary{total, per_hub:{<hub>:{events, cap_hits_total, rate_hits_total, dedupe_hits_total, cv_overflow_hits_total}}, since_days, hub_filter, malformed_lines_skipped, log_path}}`.
Do not re-render.

## Step 5: Empty-result hint

If exit 0 and zero entries:

When the log is **missing**:

```
No governor-history yet — the audit log has not been written.

The log is populated by a watch session:
  termlink fleet governor-status --watch 30 --log ~/.termlink/governor.log

Leave that running to capture backpressure events. For the live
picture right now, use:
  /governor --only-pressured
```

When the log **exists but no entries match** the window/hub filter:

```
No governor events in the last <N> day(s)<hub clause>.

No hub crossed a backpressure boundary in the window — no connection
refused, no rate-limit, no cv_index overflow. This is the healthy
case. Widen with --since, or drop --hub to see all hubs.
```

Never silent on empty.

## Rules

- **Read-only by contract.** This verb only walks a log file; it never
  probes hubs or tunes limits.
- **No `AskUserQuestion`** — just run and report.
- **Fleet-scoped by log.** Unlike the local-only claim/queue/find-idle
  histories, governor data is fleet-wide because the writing watch loop
  walks `hubs.toml`. History reflects whatever hubs that watch session
  covered.
- **Pair with /governor for the live view.**

## Common patterns

**"Recurring squeeze or one-off?" triage:**

```
/governor --only-pressured                       # which hub is pressured now
/governor-history --hub ring20-management --since 30   # has it happened before
```

**cv_index overflow follow-up (T-2119):** a non-zero `cv_overflow`
delta means a producer is mis-emitting `metadata.cv_key`. Identify the
offending topic with `/cv-keys <topic> --hub <addr>`.

**Pipe to scripting:**

```
/governor-history --json | jq '.summary.per_hub | to_entries | map(select(.value.cap_hits_total > 0))'
```

**Start capturing history (lives at the CLI tier):**

```
termlink fleet governor-status --watch 30 --log ~/.termlink/governor.log
```

## Related

- T-2068 — `fleet governor-history`, the CLI verb this skill wraps.
- T-2069 — `termlink_fleet_governor_history` MCP parity.
- T-2066 — `governor-status --watch --log`, the writer that populates
  `~/.termlink/governor.log`.
- T-2095 / `/governor` — the live-snapshot sibling skill.
- T-2048 — the underlying hub governor substrate primitive #10.
- T-2119 — cv_index overflow fields surfaced in the history line.
- T-2209 — this skill (history-verb skill-layer completion arc).
