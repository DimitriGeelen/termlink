# /find-idle-history — retrospective of agent idle/busy transitions (T-2209 skill-layer wrap)

Wraps `termlink agent find-idle-history` (substrate primitive #2
DISPATCH observability arc, shipped under T-2081). Answers the
**retrospective** question the live `/find-idle` view cannot: "did
claude-alpha go busy in the last hour?" / "is this worker flapping
between idle and busy?"

Read-only. Walks the audit log `~/.termlink/find-idle.log` — the NDJSON
trail written by `termlink agent find-idle --watch --log <PATH>`
(T-2080). No auth, no network, no state mutation. If the watch loop
never ran with `--log`, the log won't exist and the verb says so.

`/find-idle-history` is the **DISPATCH-RETROSPECTIVE** verb, completing
the read-side arc alongside its live sibling:

- **/find-idle** (T-2092) — "who's idle RIGHT NOW?" (live snapshot)
- **/find-idle-history** (this skill) — "when did this worker free up
  or go busy?" (forensic walk of the audit log)

Pair pattern: a dispatch lands on a worker that immediately looks busy
→ `/find-idle-history --agent-id <id>` shows its recent idle/busy
timeline → if it flaps, the worker may be mis-heartbeating, not truly
free.

**Invocation:**

| Form | Action |
|------|--------|
| `/find-idle-history` | Last 7 days of idle transitions, all agents |
| `/find-idle-history --since 1` | Narrow to the last day (clamped 1..=365) |
| `/find-idle-history --agent-id claude-alpha` | Filter to one exact agent_id |
| `/find-idle-history --log <path>` | Read a non-default log location |
| `/find-idle-history --json` | Machine-readable envelope (passthrough) |

`find-idle-history` semantics (per T-2081):

- **Reads the audit log, not the hub.** Data is only as complete as the
  `find-idle --watch --log` session that wrote it. No watch session ⇒
  empty log ⇒ the verb prints a hint pointing back at the writer.
- **Window default 7 days**, clamped to `1..=365`.
- **Idle is binary** — there is no `transition` kind. The only event
  kinds are `new` (an agent became idle) and `removed` (an agent went
  busy). A re-heartbeat while still idle is not an event. The per-agent
  aggregate footer counts `new` vs `removed`.

## Step 1: Pre-flight

Run:

```
termlink agent find-idle-history --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
find-idle-history: `termlink` CLI not on PATH, or this build predates
T-2081. Run `termlink --version` to verify; the verb shipped with
substrate primitive #2's observability arc.
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Pass through verbatim.

Examples:

| User typed | Command emitted |
|------------|-----------------|
| `/find-idle-history` | `termlink agent find-idle-history` |
| `/find-idle-history --since 1` | `termlink agent find-idle-history --since 1` |
| `/find-idle-history --agent-id claude-alpha` | `termlink agent find-idle-history --agent-id claude-alpha` |
| `/find-idle-history --agent-id claude-alpha --since 1 --json` | passthrough |

Empty `$ARGUMENTS` is valid (defaults to last 7 days, all agents).

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Render the result

For **default human-format** output:

- Pass the verb's stdout through verbatim (one line per matching entry
  + per-agent aggregate footer `<agent_id>  N new  N removed`).
- If exit code is non-zero, surface stderr and stop.
- If exit 0 with no entries, see Step 5.

For `--json` mode: pass the envelope through verbatim. Shape:
`{ok, entries[], summary{total, per_agent:{<id>:{new, removed}}, since_days, agent_id_filter, malformed_lines_skipped, log_path}}`.
Do not re-render.

## Step 5: Empty-result hint

If exit 0 and zero entries:

When the log is **missing**:

```
No find-idle-history yet — the audit log has not been written.

The log is populated by a watch session:
  termlink agent find-idle --role claude-code --watch 30 --log ~/.termlink/find-idle.log

This is typically an orchestrator-side loop. For who's idle right now,
use:
  /find-idle
```

When the log **exists but no entries match** the window/agent filter:

```
No idle transitions in the last <N> day(s)<agent clause>.

No agent went idle or busy in the window. Widen with --since, or drop
--agent-id to see all agents.
```

Never silent on empty.

## Rules

- **Read-only by contract.** This verb only walks a log file.
- **No `AskUserQuestion`** — just run and report.
- **Local audit log only.** find-idle is local-hub-only by ADR §6
  design; its history reflects this host's watch sessions.
- **Idle is binary** — don't expect `transition` events; only
  `new`/`removed`.
- **Pair with /find-idle for the live view.**

## Common patterns

**"Is this worker flapping?" triage:**

```
/find-idle                                   # who's free now
/find-idle-history --agent-id claude-alpha --since 1   # its recent idle/busy timeline
```

**Pipe to scripting:**

```
/find-idle-history --json | jq '.summary.per_agent'
```

**Start capturing history (lives at the CLI tier):**

```
termlink agent find-idle --role claude-code --watch 30 --log ~/.termlink/find-idle.log
```

## Related

- T-2081 — `agent find-idle-history`, the CLI verb this skill wraps.
- T-2082 — `termlink_agent_find_idle_history` MCP parity.
- T-2080 — `find-idle --watch --log`, the writer that populates
  `~/.termlink/find-idle.log`.
- T-2092 / `/find-idle` — the live-snapshot sibling skill.
- T-2045 / T-2020 — the underlying `agent find-idle` substrate
  primitive #2.
- T-2209 — this skill (history-verb skill-layer completion arc).
