# /queue-status — inspect this host's offline-queue depth (T-2094 skill-layer wrap)

Wraps `termlink channel queue-status` (substrate primitive #5 RESILIENCE
offline-queue inspect, shipped under T-2051). Answers "is my queue
draining or backing up right now?" — the operator's quick-check after
suspecting a host blip.

Read-only, no state mutation, no auth, no network. Inspects the
local `~/.termlink/outbound.sqlite` durable FIFO that absorbs hub
blips.

`/queue-status` is the **RESILIENCE-READ** daily verb completing the
substrate primitive #5 skill surface:

- **/find-idle** (T-2092) — DISPATCH (substrate #2 read)
- **/claims** (T-2093) — CLAIM-READ (substrate #1 read)
- **/queue-status** (this skill) — RESILIENCE (substrate #5 read)

The watch/notify/log/history forms stay at the CLI tier (T-2083..T-2087)
because long-running monitor loops sit awkwardly inside slash-commands
— same design rationale as siblings.

**Invocation:**

| Form | Action |
|------|--------|
| `/queue-status` | Show current queue depth + oldest age + queue path |
| `/queue-status --json` | Machine-readable envelope (passthrough to verb) |

**What the verb reports** (per T-2051 + T-2083 schema):

- `pending`: number of envelopes waiting to flush
- `oldest_age_ms`: age of the oldest pending envelope (ms), or null when queue is drained
- `queue_path`: path to the durable FIFO (typically `~/.termlink/outbound.sqlite`)

**Operational reading:**

- `pending=0` → queue drained, hub is reachable, nothing in flight. This is steady-state.
- `pending>0, oldest_age_ms small` → queue is actively draining (recent post). Wait a moment.
- `pending>0, oldest_age_ms large` → host has been blipped; flush task is retrying. Investigate hub reachability.
- `pending=N, growing` → flush is failing repeatedly. Check `termlink fleet doctor` next.

## Step 1: Pre-flight

Run:

```
termlink channel queue-status --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
queue-status: `termlink` CLI not on PATH or substrate primitive #5
(RESILIENCE) not available in this build. Ensure you're on a version
with T-2051 shipped (run `termlink --version` to verify).
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Pass through verbatim — the
underlying verb validates and errors with usage on malformed input.

Examples:

| User typed | Command emitted |
|------------|-----------------|
| `/queue-status` | `termlink channel queue-status` |
| `/queue-status --json` | `termlink channel queue-status --json` |

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Render the result

For **default human-format** output:

- Pass the verb's stdout through verbatim (one-line summary:
  `queue=<path> pending=N oldest_age=Mms`).
- If exit code is non-zero, surface stderr and stop.

For `--json` mode: pass the JSON envelope through verbatim. Do not
re-render — callers piping/parsing rely on the substrate verb's
schema.

## Step 5: Empty-result hint

If `pending=0` (queue drained):

```
Queue drained — hub is reachable and nothing is in flight.

This is steady-state. Posts are flowing through directly without
buffering. If you suspect a blip is brewing, leave a watch running:
  termlink channel queue-status --watch 5
```

If `pending>0` and `oldest_age_ms` is small (<5s):

```
Queue actively draining (pending=N, oldest_age=Mms).

Recent post is in flight. Wait a moment and re-run /queue-status to
see whether it drained. If pending keeps growing across reruns, the
hub may be unreachable — run `termlink fleet doctor` next.
```

If `pending>0` and `oldest_age_ms` is large (>5s):

```
Host has been blipped (pending=N, oldest_age=Mms).

The flush task is retrying. Likely causes:
- Local hub down or restarting
- Network unreachable to remote hub
- Hub rate-limiting (substrate #10 — check: termlink hub status --governor)

Next diagnostic step:
  termlink fleet doctor                       # hub reachability + auth
  termlink channel queue-status --watch 5     # live drain monitor

If the queue keeps growing past TERMLINK_OUTBOUND_CAP (default 1000),
new posts will start failing with QueueFull (R3 loud-fail per T-2051).
```

If the verb returns "queue path not found" or queue uninitialized:

```
No offline queue at the expected path.

The offline queue is created lazily on first hub-blip absorption.
Empty path = no posts have ever needed buffering. This is fine.

If you expected to see queue activity, verify:
  ls -la ~/.termlink/outbound.sqlite           # default path
  echo "${TERMLINK_IDENTITY_DIR:-$HOME/.termlink}/outbound.sqlite"  # custom path via TERMLINK_IDENTITY_DIR
```

Never silent on empty.

## Step 6: Reference the audit trail

For human-format mode (NOT json), after the verb's output, append:

```
Forensic / retrospective:
- termlink channel queue-status --watch 5                 # continuous monitor (T-2083)
- termlink channel queue-status --watch 5 --notify <cmd>  # page on drained↔pending transitions (T-2084)
- termlink channel queue-status --watch 5 --log <path>    # NDJSON audit trail (T-2085)
- termlink channel queue-history --since 7 --kind pending # retrospective (T-2086)
```

Skip this section in `--json` mode (machine output stays pure).

## Rules

- **Read-only by contract.** Never flush, never drop, never modify
  queue state. The flush task runs autonomously every 5s per T-2051.
- **No network access.** Pure local SQLite read of the durable FIFO.
- **No `AskUserQuestion`** — just run and report.
- **Pair with `termlink fleet doctor`** when pending stays high — the
  queue is the symptom, hub reachability is usually the cause.
- **The watch/notify/log/history forms stay at the CLI tier.**
  T-2083..T-2087 give long-running monitor / event / audit / retrospective
  surfaces. `/queue-status` is the one-shot daily-check verb.

## Common patterns

**Quick "is everything flowing?" check:**

```
/queue-status
```

**After a suspected hub blip:**

```
/queue-status                              # how deep?
termlink fleet doctor                      # is hub reachable?
/queue-status --json | jq '.pending'       # script-friendly
```

**Continuous monitor (CLI tier):**

```
termlink channel queue-status --watch 5 --notify /usr/local/bin/page-on-queue-pending.sh --log ~/.termlink/queue.log
```

The watch/notify/log forms are T-2083 / T-2084 / T-2085; the
retrospective read is T-2086's `channel queue-history`.

## Related

- T-2018 — arc-parallel-substrate ADR; this skill operationalizes #5
  read-side at the daily-verb tier.
- T-2051 — substrate primitive #5 RESILIENCE (offline queue + flush task).
- T-2049 — post-idempotency / dedupe (exactly-once across blips).
- T-2083 / T-2084 / T-2085 — `--watch --notify --log` CLI-tier forms.
- T-2086 — `channel queue-history` retrospective verb.
- T-2087 — `termlink_channel_queue_history` MCP parity.
- T-2092 / `/find-idle` — sibling daily-verb skill (substrate #2 read).
- T-2093 / `/claims` — sibling daily-verb skill (substrate #1 read).
- `docs/operations/substrate-offline-queue-recipe.md` — master recipe.
- PL-187 — verb-stack pattern rung 6 (ephemeral session skills).
