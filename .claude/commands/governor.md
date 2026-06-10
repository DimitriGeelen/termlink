# /governor — fleet backpressure snapshot (T-2095 skill-layer wrap)

Wraps `termlink fleet governor-status` (substrate primitive #10
BACKPRESSURE read-side, shipped under T-2048 / T-2060 / T-2062 /
T-2070). Answers "is the hub being rate-limited or at-capacity right
now?" — the operational health check that catches the silent class of
problems (refused connections, rate-limited posts, dedupe-absorbed
retries).

Read-only, no state mutation. Probes every hub in
`~/.termlink/hubs.toml` via the existing `hub.governor_status`
JSON-RPC path (Observe scope, no auth side-effects).

`/governor` is the **BACKPRESSURE-READ** daily verb completing the
substrate-read quad:

- **/find-idle** (T-2092) — DISPATCH (substrate #2)
- **/claims** (T-2093) — CLAIM-READ (substrate #1)
- **/queue-status** (T-2094) — RESILIENCE-READ (substrate #5)
- **/governor** (this skill) — BACKPRESSURE-READ (substrate #10)

Together the four cover the full substrate situational-awareness
surface. Watch/notify/log/history forms stay at the CLI tier
(T-2064..T-2069) because long-running loops sit awkwardly inside
slash-commands.

**Invocation:**

| Form | Action |
|------|--------|
| `/governor` | Walk every hub in `hubs.toml`, render per-hub block + fleet-wide roll-up |
| `/governor --only-pressured` | Filter to only hubs that need attention (T-2070) |
| `/governor --json` | Machine-readable envelope (passthrough to verb) |
| `/governor --only-pressured --json` | Filtered envelope (T-2071 MCP-parity shape) |

**What the verb reports** (per T-2048 + T-2049 + T-2110 schema):

- `connections_active` / `connections_max` — TCP connection budget
- `capacity_hits_total` — count of `HUB_AT_CAPACITY` (-32019) refusals
- `rate_buckets_active` / `rate_hits_total` / `max_rate_per_sec` — per-sender rate limiter
- `dedupe_entries_active` / `dedupe_hits_total` / `dedupe_ttl_ms` — post-idempotency absorption (T-2049)
- `cv_index_entries_active` / `cv_index_topics_active` / `cv_index_overflow_total` / `cv_index_cap_per_topic` — broadcast-with-replay (substrate #9) telemetry (T-2110). Pre-T-2110 hubs render as `n/a` (NOT `0`) — distinguish "no field" from "0 events".

**Operational reading:**

- `capacity_hits_total > 0` → a connection was refused. Investigate `TERMLINK_MAX_CONNECTIONS` (default 256).
- `rate_hits_total > 0` → an RPC was refused. Investigate `TERMLINK_RATE_LIMIT_PER_SEC` (default 1000).
- `dedupe_hits_total > 0` → spoke retries were absorbed before double-applying — this is **good**, exactly-once is working.
- `cv_overflow > 0` → **smoking-gun for producer mis-emitting `cv_key`** (e.g. timestamp instead of stable id) saturating the per-topic cap. Binary signal — ANY non-zero value is operator-actionable. Run `termlink channel cv-keys <topic>` to identify which topic. Fires `--only-pressured` (T-2118) and surfaces as per-event delta in `--watch --notify --log` (T-2119).
- All zeros + hubs reachable → steady-state healthy.

## Step 1: Pre-flight

Run:

```
termlink fleet governor-status --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
governor: `termlink` CLI not on PATH or substrate primitive #10
(BACKPRESSURE) not available in this build. Ensure you're on a version
with T-2048 + T-2062 shipped (run `termlink --version` to verify).
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Pass through verbatim — the
underlying verb validates and errors with usage on malformed input.

Examples:

| User typed | Command emitted |
|------------|-----------------|
| `/governor` | `termlink fleet governor-status` |
| `/governor --only-pressured` | `termlink fleet governor-status --only-pressured` |
| `/governor --json` | `termlink fleet governor-status --json` |
| `/governor --only-pressured --json` | passthrough |

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Render the result

For **default human-format** output:

- Pass the verb's stdout through verbatim (per-hub blocks +
  fleet-wide footer with `hubs_at_capacity` / `hubs_rate_limited`
  totals).
- If exit code is non-zero, surface stderr and stop.

For `--json` mode: pass the JSON envelope through verbatim. Do not
re-render — callers piping/parsing rely on the substrate verb's
schema.

## Step 5: Empty-result hint

For `--only-pressured` mode with zero pressured hubs (T-2070's
"affirmative confirmation, not silent success" path):

The verb itself prints `All hubs healthy (0/N pressured)` — just pass
that through. Append a follow-up only if useful:

```
Steady state. To leave a watch running:
  termlink fleet governor-status --watch 30 --include-pin-check
```

For default mode with all-zero counters across all hubs:

```
Fleet steady-state — no backpressure events recorded.

Counters are cumulative since each hub started, so all-zero across
all hubs means:
- No connection refusals (TERMLINK_MAX_CONNECTIONS=256 is sufficient)
- No rate-limit hits (TERMLINK_RATE_LIMIT_PER_SEC=1000 is sufficient)
- No dedupe absorptions (no spoke retries observed — fleet hasn't blipped)

If you expected to see activity:
- Were any hubs restarted recently? Counters reset on restart.
- Is hubs.toml current? termlink fleet verify
```

For default mode with at least one hub unreachable:

```
One or more hubs unreachable. The reachable hubs report their
counters; the unreachable ones surfaced as `UNREACHABLE` blocks.

Next diagnostic step:
  termlink fleet doctor                  # auth + cert pin diagnostic
  termlink fleet verify                  # TLS-fingerprint probe per hub
```

Never silent on empty.

## Step 6: Reference the audit trail

For human-format mode (NOT json), after the verb's output, append:

```
Forensic / retrospective:
- termlink fleet governor-status --watch 30                 # continuous monitor (T-2064)
- termlink fleet governor-status --watch 30 --notify <cmd>  # page on counter change (T-2065)
- termlink fleet governor-status --watch 30 --log <path>    # NDJSON audit trail (T-2066)
- termlink fleet governor-history --since 7 --hub <name>    # retrospective (T-2068)
- termlink_fleet_governor_history (MCP)                      # agent-callable retrospective (T-2069)
```

Skip this section in `--json` mode (machine output stays pure).

## Rules

- **Read-only by contract.** Never tune budgets, never restart hubs,
  never modify governor state.
- **Pure Observe-scope reads** — no auth side-effects on any hub.
- **No `AskUserQuestion`** — just run and report.
- **Pair with `/queue-status`** when investigating a blip — if your
  local queue is pending and a hub shows `rate_hits_total` growing,
  you've found the cause.
- **The watch/notify/log/history forms stay at the CLI tier.**
  T-2064..T-2069 give long-running monitor / event / audit / retrospective
  surfaces. `/governor` is the one-shot daily-check verb.

## Common patterns

**Steady-state health check:**

```
/governor                            # fleet-wide snapshot
```

**"Show me only what needs attention":**

```
/governor --only-pressured           # filter to pressured hubs (T-2070)
```

**Script-friendly fleet totals:**

```
/governor --json | jq '.summary.hubs_at_capacity'
```

**Continuous monitor with paging (CLI tier):**

```
termlink fleet governor-status --watch 30 --notify /usr/local/bin/page-on-cap.sh --log ~/.termlink/governor.log
```

**Page on cv_index overflow (catches producer mis-emitting cv_key, T-2119):**

```
termlink fleet governor-status --watch 30 --notify /usr/local/bin/page-on-cv-overflow.sh --log ~/.termlink/governor.log
```

The notify script gates on `[ "$TERMLINK_GOV_CV_OVERFLOW_DELTA" -gt 0 ]` then
pages — see `docs/operations/substrate-governor.md` § "Recipe — `--notify`
script template" for the full body.

The watch/notify/log forms are T-2064 / T-2065 / T-2066; the
retrospective read is T-2068's `fleet governor-history` verb.

## Related

- T-2018 — arc-parallel-substrate ADR; this skill operationalizes #10
  read-side at the daily-verb tier.
- T-2048 — substrate primitive #10 BACKPRESSURE (connection-cap +
  per-sender rate-limit + dedupe counters).
- T-2049 — post-idempotency / exactly-once (dedupe counter source).
- T-2060 — CLI: `hub status --governor` single-host form.
- T-2062 — fleet-wide aggregation (`fleet governor-status`).
- T-2064 / T-2065 / T-2066 — `--watch --notify --log` CLI-tier forms.
- T-2068 — `fleet governor-history` retrospective verb.
- T-2069 — `termlink_fleet_governor_history` MCP parity.
- T-2070 — `--only-pressured` presentation-level filter.
- T-2071 — MCP-parity for the only-pressured filter.
- T-2110 — cv_index telemetry (entries/topics/overflow/cap) surfaced via this envelope. Closes substrate §6 #9↔#10 cross-reference.
- T-2118 — `--only-pressured` predicate fires on `cv_index_overflow_total > 0`.
- T-2119 — watch/notify/log/history carry cv_overflow deltas end-to-end. `page-on-cv-overflow.sh` recipe in `docs/operations/substrate-governor.md`.
- T-2092 / `/find-idle` — sibling daily-verb skill (substrate #2 read).
- T-2093 / `/claims` — sibling daily-verb skill (substrate #1 read).
- T-2094 / `/queue-status` — sibling daily-verb skill (substrate #5 read).
- `docs/operations/substrate-governor.md` — master recipe.
- PL-187 — verb-stack pattern rung 6 (ephemeral session skills).
