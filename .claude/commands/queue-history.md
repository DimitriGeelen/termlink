# /queue-history — retrospective of offline-queue backpressure (T-2209 skill-layer wrap)

Wraps `termlink channel queue-history` (substrate primitive #5
RESILIENCE observability arc, shipped under T-2086). Answers the
**retrospective** question the live `/queue-status` view cannot: "how
often has this host's outbound queue backed up?" / "how frequently does
this host blip?"

Read-only. Walks the audit log `~/.termlink/queue.log` — the NDJSON
trail written by `termlink channel queue-status --watch --log <PATH>`
(T-2085). No auth, no network, no state mutation. If the watch loop
never ran with `--log`, the log won't exist and the verb says so.

`/queue-history` is the **RESILIENCE-RETROSPECTIVE** verb, completing
the read-side arc alongside its live sibling:

- **/queue-status** (T-2094) — "is my queue draining RIGHT NOW?"
  (live snapshot)
- **/queue-history** (this skill) — "how often does it back up?"
  (forensic walk of the audit log)

Pair pattern: a slow operation makes you suspect a hub blip →
`/queue-status` shows the queue is pending now → `/queue-history`
answers "is this host chronically blippy or is this rare?"

**Invocation:**

| Form | Action |
|------|--------|
| `/queue-history` | Last 7 days of queue state changes |
| `/queue-history --since 30` | Widen the window to 30 days (clamped 1..=365) |
| `/queue-history --kind pending` | Filter to backpressure events only |
| `/queue-history --kind drained` | Filter to recovery events only |
| `/queue-history --log <path>` | Read a non-default log location |
| `/queue-history --json` | Machine-readable envelope (passthrough) |

`queue-history` semantics (per T-2086):

- **Reads the audit log, not the SQLite queue.** Data is only as
  complete as the `queue-status --watch --log` session that wrote it.
  No watch session ⇒ empty log ⇒ the verb prints a hint.
- **Window default 7 days**, clamped to `1..=365`.
- **Queue state is binary** — there is no `transition` kind. Event
  kinds are `pending` (the queue started buffering — a hub blip
  absorbed) and `drained` (it emptied — recovery). The summary footer
  counts `pending` vs `drained` events.

## Step 1: Pre-flight

Run:

```
termlink channel queue-history --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
queue-history: `termlink` CLI not on PATH, or this build predates
T-2086. Run `termlink --version` to verify; the verb shipped with
substrate primitive #5's observability arc.
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Pass through verbatim.

Examples:

| User typed | Command emitted |
|------------|-----------------|
| `/queue-history` | `termlink channel queue-history` |
| `/queue-history --since 30` | `termlink channel queue-history --since 30` |
| `/queue-history --kind pending` | `termlink channel queue-history --kind pending` |
| `/queue-history --kind pending --since 7 --json` | passthrough |

Empty `$ARGUMENTS` is valid (defaults to last 7 days, both kinds).

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Render the result

For **default human-format** output:

- Pass the verb's stdout through verbatim (one line per matching entry
  + the `pending=N  drained=M` aggregate footer).
- If exit code is non-zero, surface stderr and stop.
- If exit 0 with no entries, see Step 5.

For `--json` mode: pass the envelope through verbatim. Shape:
`{ok, entries[], summary{total, pending_events, drained_events, since_days, kind_filter, malformed_lines_skipped, log_path}}`.
Do not re-render.

## Step 5: Empty-result hint

If exit 0 and zero entries:

When the log is **missing**:

```
No queue-history yet — the audit log has not been written.

The log is populated by a watch session:
  termlink channel queue-status --watch 5 --log ~/.termlink/queue.log

Leave that running if you suspect this host blips. For the live queue
depth right now, use:
  /queue-status
```

When the log **exists but no entries match** the window/kind filter:

```
No queue state-changes in the last <N> day(s)<kind clause>.

The queue stayed drained the whole window — no hub blip caused
buffering. This is the healthy case. Widen with --since, or drop
--kind to see both pending and drained.
```

Never silent on empty.

## Rules

- **Read-only by contract.** This verb only walks a log file; it never
  flushes or modifies the outbound queue.
- **No `AskUserQuestion`** — just run and report.
- **Local audit log only.** The offline queue is this host's local
  SQLite FIFO; its history is host-local.
- **Queue state is binary** — only `pending`/`drained`, no
  `transition`.
- **Pair with /queue-status for the live view.**

## Common patterns

**"How blippy is this host?" triage:**

```
/queue-status                          # pending right now?
/queue-history --kind pending --since 7   # how many blips this week?
```

**Pipe to scripting:**

```
/queue-history --json | jq '.summary.pending_events'
```

**Start capturing history (lives at the CLI tier):**

```
termlink channel queue-status --watch 5 --log ~/.termlink/queue.log
```

## Related

- T-2086 — `channel queue-history`, the CLI verb this skill wraps.
- T-2087 — `termlink_channel_queue_history` MCP parity.
- T-2085 — `queue-status --watch --log`, the writer that populates
  `~/.termlink/queue.log`.
- T-2094 / `/queue-status` — the live-snapshot sibling skill.
- T-2051 — the underlying offline-queue substrate primitive #5.
- T-2209 — this skill (history-verb skill-layer completion arc).
