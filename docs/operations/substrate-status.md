# Substrate observability composition — SUBSTRATE-PULSE (cross-primitive rollup)

(SUBSTRATE-PULSE is the operator-facing name for the composition; it is
not an ADR §6 manifest primitive on its own — it composes #1, #2, #5,
and #10 into one situational view.)

T-2111..T-2117 close the SUBSTRATE-PULSE observability arc end-to-end: a
single CLI verb (`termlink substrate status`) + MCP companion that
composes the four substrate-read primitives into one situational
view, plus a `--watch`/`--notify`/`--log` event loop and a
retrospective `substrate history` walk over the audit trail.

This is the operator's "is my substrate healthy right now?" answer.
The four sub-primitives each answer one question; SUBSTRATE-PULSE
answers all four in one shot, in parallel, with graceful
degradation per section.

Read with `docs/operations/substrate-orchestrator-recipe.md` (T-2124)
for the AEF-integration walkthrough and per-primitive ops docs
(`substrate-claim-primitive.md`, `substrate-broadcast-with-replay.md`,
`substrate-governor.md`, `substrate-offline-queue-recipe.md`,
`substrate-post-idempotency.md`) for the underlying surfaces this
verb composes.

## What it composes

| Sub-read | Substrate primitive | Question answered |
|---|---|---|
| `agent.find_idle` (local hub) | #2 DISPATCH (T-2020/T-2045) | Who's free to take work? |
| `channel.claims_summary` per topic (local hub) | #1 CLAIM (T-2019/T-2042) | Any stuck claims? |
| `OfflineQueue::open` (local SQLite) | #5 RESILIENCE (T-2051) | Is my outbound queue draining? |
| `hub.governor_status` per hub (fleet) | #10 BACKPRESSURE (T-2048) | Any hub being refused? |

All four sub-reads dispatch via `tokio::join!`; total latency is
`max-of-four`, not `sum-of-four`. A failed sub-read renders as
`(<SECTION> unavailable: ...)` in human mode + `ok:false` in JSON;
the other three sections still render. Local-hub-down kills
DISPATCH + CLAIM, but RESILIENCE (local SQLite) and BACKPRESSURE
(fleet-wide RPC) still produce useful output.

## The wire shape

### One-shot CLI — `termlink substrate status`

```sh
termlink substrate status [--json] [--only-pressured] [--timeout SECS]
```

| Flag | Default | Meaning |
|---|---|---|
| `--json` | off | Emit merged JSON envelope; mutex with `--watch` |
| `--only-pressured` | off | Filter CLAIM to potentially-stuck topics + BACKPRESSURE to hubs needing attention; mirror of T-2070/T-2076 |
| `--timeout SECS` | 12 | Per-sub-read bound (BACKPRESSURE per-hub RPC, DISPATCH/CLAIM local-hub RPC); clamped 1..=120 |

Human-format output renders four labeled sections. The affirmative
empty-state is rendered explicitly (never silent):

```
═══ substrate status ═══

DISPATCH (substrate #2 — who's free to take work?):
  (no idle agents — see `agent find-idle` for diagnostic)

CLAIM (substrate #1 — any stuck claims?):
  work-queue  active=2 expired=0 oldest_age=4.2s
  ... (one row per topic — or "All topics healthy (0/N stuck)" under --only-pressured)

RESILIENCE (substrate #5 — queue draining?):
  pending=0 (steady-state)

BACKPRESSURE (substrate #10 — any hub pressured?):
  ring20-management  conn=12/256 cap_hits=0 rate_hits=0  (healthy)
  ... (per-hub row + fleet rollup)
```

JSON envelope:

```json
{
  "ok": true,
  "ts": "2026-06-10T17:28:13Z",
  "only_pressured": false,
  "dispatch":    {"ok": true, "data": { ... agent.find_idle shape ... }},
  "claim":       {"ok": true, "data": { ... claims_summary --all shape ... }},
  "resilience":  {"ok": true, "data": { ... queue-status shape ... }},
  "backpressure":{"ok": true, "data": { ... fleet governor-status shape ... }}
}
```

Each sub-section either `{ok:true, data}` (passthrough of its
underlying verb's `--json` shape) or `{ok:false, error}` (graceful
degradation). Top-level `ok` is `false` iff any sub-read failed.

### Continuous monitor — `--watch <SECS>`

```sh
termlink substrate status --watch 30
```

Loops every N seconds (clamped 5..=3600). Cycle 1 prints the full
baseline rollup. Cycle N>1 prints one change-line per rollup field
that changed since the prior cycle:

```
[2026-06-10T17:30:00Z] DISPATCH:    idle 0 → 2 agents (+2)
[2026-06-10T17:30:30Z] CLAIM:       stuck 0 → 1 topic (+1, oldest=work-queue 82s)
[2026-06-10T17:30:30Z] BACKPRESSURE: pressured 0 → 1 hub (ring20-management cap_hits 0 → 3)
```

Silent cycles render one `# no changes` footer line. SIGINT exits
cleanly with a final summary. Pattern parity with `fleet doctor
--watch` (T-1667) and `fleet governor-status --watch` (T-2064).

Mutex with `--json` (NDJSON-on-cleared-screen would be
unparseable) and `--only-pressured` (filter would silently drop
"moved out of pressure" transitions).

### Event hook — `--notify <CMD>`

```sh
termlink substrate status --watch 30 --notify /usr/local/bin/page-on-rollup-change.sh
```

Fires the command fire-and-forget per change event after the
baseline cycle. Per-event env vars passed:

| Env var | Meaning |
|---|---|
| `TERMLINK_SUBSTRATE_FIELD` | Which rollup field changed (`DISPATCH`, `CLAIM`, `RESILIENCE`, `BACKPRESSURE`) |
| `TERMLINK_SUBSTRATE_OLD` | Prior value (printable string, e.g. `idle=0`, `stuck=0`) |
| `TERMLINK_SUBSTRATE_NEW` | New value |
| `TERMLINK_SUBSTRATE_TS` | RFC3339 detection timestamp |

Hanging scripts do NOT block the loop (fire-and-forget). Spawn
failure (command-not-found) does NOT kill the watch (one stderr
line + continue). Requires `--watch`.

Common gate at the top of the notify script:

```sh
[ "$TERMLINK_SUBSTRATE_FIELD" = "BACKPRESSURE" ] || exit 0
# then page on-call ...
```

### Audit trail — `--log <PATH>`

```sh
termlink substrate status --watch 30 --log ~/.termlink/substrate.log
```

Appends one NDJSON line per change event (after baseline):

```json
{"ts":"2026-06-10T17:30:30Z","field":"CLAIM","old":"stuck=0","new":"stuck=1"}
```

Best-effort writes (parent dir auto-created; disk-full / permission
errors print one-line stderr warning + continue — watch never
crashes). Symmetric with `--notify`: when both flags are set, each
event lands in both surfaces from the same per-cycle event source.
Requires `--watch`. Mirror of T-1671's `rotation.log`, T-2066's
`governor.log`, T-2080's `find-idle.log`, T-2085's `queue.log`.

### Retrospective walk — `termlink substrate history`

```sh
termlink substrate history [--since DAYS] [--field FIELD] [--log PATH] [--json]
```

Walks the audit log (default `~/.termlink/substrate.log`) and
prints one line per matching event + per-field aggregate footer:

```
2026-06-10T17:30:30Z  CLAIM    stuck=0 → stuck=1
2026-06-10T17:32:10Z  CLAIM    stuck=1 → stuck=0
2026-06-10T17:45:00Z  BACKPRESSURE  pressured=0 → pressured=1

  CLAIM         2 events
  BACKPRESSURE  1 event
```

| Flag | Default | Meaning |
|---|---|---|
| `--since DAYS` | 7 | Window cutoff, clamped 1..=365 |
| `--field FIELD` | (all) | Exact match on `DISPATCH` \| `CLAIM` \| `RESILIENCE` \| `BACKPRESSURE` |
| `--log PATH` | `~/.termlink/substrate.log` | Override path |
| `--json` | off | Machine-readable envelope |

JSON shape:

```json
{
  "ok": true,
  "entries": [ {"ts":"...","field":"...","old":"...","new":"..."}, ... ],
  "summary": {
    "total": 3,
    "per_field": {"CLAIM": 2, "BACKPRESSURE": 1},
    "since_days": 7,
    "field_filter": null,
    "malformed_lines_skipped": 0,
    "log_path": "/root/.termlink/substrate.log"
  }
}
```

Read-only; no auth; no network; no log mutation. Missing log file →
operator hint pointing back at `substrate status --watch --log`.

### MCP parity — `termlink_substrate_status` / `termlink_substrate_history`

Subprocess-self pattern (T-1689 mirror): spawns own binary with
`substrate status --json` / `substrate history --json` under
`tokio::time::timeout` + `kill_on_drop(true)` + null stdin.

```python
# Agent investigating fleet health:
termlink_substrate_status(only_pressured=True, timeout_secs=12)
# Returns the same {ok, ts, only_pressured, dispatch, claim,
#                   resilience, backpressure, exit_code} envelope.

# Agent investigating recurring stuck-claim flap:
termlink_substrate_history(since_days=7, field="CLAIM")
# Returns {ok, entries, summary{...}}.
```

Read-only — no auth side-effects, no state mutation, no network
beyond the subprocess's own substrate reads. Use when an agent
needs the rollup without shelling out.

## Common patterns

### Cold-start fleet pulse (operator at a fresh terminal)

```sh
termlink substrate status
```

One command, four sections, parallel — answers "is anything wrong
right now?" in one round-trip.

Slash-command alias (claude-code session): `/substrate` (T-2096)
composes the same four sub-verbs at the skill tier with operator
hints layered on top.

### Page-on-rollup-change (operator standing watch)

```sh
termlink substrate status --watch 30 \
  --notify /usr/local/bin/page-on-rollup-change.sh \
  --log ~/.termlink/substrate.log
```

Real-time alerting + forensic audit trail in one process. The
notify script gates per-field; the log captures every event for
post-mortem.

Sample notify script:

```sh
#!/bin/sh
# /usr/local/bin/page-on-rollup-change.sh
# Page on backpressure transitions OR stuck-claim emergence.
case "$TERMLINK_SUBSTRATE_FIELD" in
  BACKPRESSURE) ;;
  CLAIM) echo "$TERMLINK_SUBSTRATE_NEW" | grep -q "stuck=" || exit 0 ;;
  *) exit 0 ;;
esac
exec /usr/local/bin/page-oncall "substrate $TERMLINK_SUBSTRATE_FIELD: $TERMLINK_SUBSTRATE_OLD → $TERMLINK_SUBSTRATE_NEW"
```

### Recurring-flap investigation (post-incident)

```sh
termlink substrate history --since 7 --field CLAIM
```

Walks the captured audit log to answer "how many stuck-claim flaps
in the past week?" without the watch terminal still attached. Pair
with `claims-history` (T-2074) for per-topic detail.

### Agent investigating cross-primitive health (MCP)

```python
status = termlink_substrate_status(only_pressured=True)
if not status["ok"] or any(not s["ok"] for s in (
        status["dispatch"], status["claim"],
        status["resilience"], status["backpressure"])):
    history = termlink_substrate_history(since_days=1)
    # ... reason about whether this is a fresh issue or a recurring flap
```

## What this does NOT do

- **Cross-hub rollup.** SUBSTRATE-PULSE is local-hub-scoped for
  DISPATCH + CLAIM and fleet-wide only for BACKPRESSURE (which is
  the per-hub primitive). G-060 — substrate state is per-hub by
  design; the rollup respects that.
- **Auto-heal.** SUBSTRATE-PULSE detects; it does not act. Pair
  `--notify` with operator-written heal scripts when you want
  automated response. Distinct from `fleet doctor --auto-heal`
  (T-1680/T-1683) which has its own targeted auto-heal for the
  cert/secret rotation classes only.
- **Replace per-primitive observability.** The four daily verbs
  (`agent find-idle`, `channel claims-summary`, `channel
  queue-status`, `fleet governor-status`) remain the authoritative
  surfaces for their respective primitives. SUBSTRATE-PULSE is the
  composition, not the source-of-truth — use the underlying verbs
  for deep diagnostics.
- **Mutate state.** Pure Observe-scope reads. No claim writes, no
  heartbeat emit, no log compaction.

## Related

- **Master integration recipe (T-2124):**
  [`substrate-orchestrator-recipe.md`](substrate-orchestrator-recipe.md)
  — end-to-end walkthrough composing this primitive with #1, #2,
  #3, #5, #9, #10 into the canonical work-stealing pattern.
- **Sibling per-primitive ops docs (the four sub-reads):**
  - [`substrate-claim-primitive.md`](substrate-claim-primitive.md) (#1 — CLAIM)
  - [`substrate-broadcast-with-replay.md`](substrate-broadcast-with-replay.md) (#9 — BROADCAST + cv_index)
  - [`substrate-governor.md`](substrate-governor.md) (#10 — BACKPRESSURE; the `governor.log` arc mirrors `substrate.log`)
  - [`substrate-offline-queue-recipe.md`](substrate-offline-queue-recipe.md) (#5 — RESILIENCE; the queue-status arc mirrors substrate-status)
- **Slash-command alias:** `/substrate` (T-2096) — same composition
  at the skill tier with operator hints.
- T-2018 ADR §6 — the substrate manifest this composes (#1 CLAIM, #2 DISPATCH, #5 RESILIENCE, #10 BACKPRESSURE)
- T-2111..T-2117 — substrate-status arc build chain (Slice 1
  one-shot CLI → Slice 2 `--watch` → Slice 3 `--notify` → Slice 4
  `--log` → Slice 5 `substrate history` CLI → Slice 6
  `termlink_substrate_status` MCP → Slice 7
  `termlink_substrate_history` MCP)
- T-2096 — `/substrate` slash-command skill (skill-tier alias)
- T-2124 — master orchestrator recipe (the doc that walks every
  primitive end-to-end)
- T-2127 — this doc (pattern-parity closure with the other five
  substrate primitive ops docs)
