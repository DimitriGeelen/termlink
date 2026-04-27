# API-usage telemetry (T-1304)

> **Quick start:** start a hub on a path you control, generate some traffic,
> then run `fw metrics api-usage --last-Nd 7 --runtime-dir <runtime_dir>`.

The hub appends one JSON line per parseable RPC dispatch to
`<runtime_dir>/rpc-audit.jsonl`. `fw metrics api-usage` reads the file,
filters by time window, tallies per-method counts, and reports the
percentage of legacy primitives — used as the **T-1166 entry gate**
(retire legacy `event.broadcast` + `inbox.*` + `file.*` primitives once
their share drops below 1%).

## Where data lives

| File | Purpose |
|------|---------|
| `<runtime_dir>/rpc-audit.jsonl` | Append-only audit log, one line per RPC |

`<runtime_dir>` is whatever the hub was started with — typically
`/var/lib/termlink/` (production) or `/tmp/termlink-0/` (legacy default).
Override at runtime with `TERMLINK_RUNTIME_DIR` or `--runtime-dir`.

## Line format

```json
{"ts":1714234567890,"method":"event.broadcast","from":"framework-agent"}
```

- `ts` — UNIX milliseconds (UTC)
- `method` — the JSON-RPC `method` field as the client sent it (no
  normalization)
- `from` — *(T-1309, optional)* caller display_name when the request
  carries a top-level `from` parameter. Omitted when the caller did not
  supply it (legacy clients, methods that don't carry the field). Surfaces
  as `(unknown)` in the caller breakdown.

The hub records every parseable request — including auth attempts and
permission-denied calls. The audit log answers "what was asked of the
hub", not "what succeeded". This is intentional: a flood of
`event.broadcast` calls that all fail auth still indicates a stale
caller that hasn't migrated.

## Reading the report

### Trend mode (default — incremental feedback)

```sh
fw metrics api-usage
```

```
== fw metrics api-usage ==
  Audit file: /var/lib/termlink/rpc-audit.jsonl
  Mode:       trend

    Window     Total    Legacy   Legacy %  Status
  --------  --------  --------  ---------  ------
      1d         812         0      0.00%  PASS
      7d        5904         3      0.05%  PASS
     30d       18203        12      0.07%  PASS
     60d       33421        29      0.09%  PASS

  Top 10 methods (last 60d):
       28000   83.8%  channel.post
        4500   13.5%  event.subscribe
          29    0.1%  event.broadcast ←legacy
        ...

  Legacy callers (last 60d):
          23  event.broadcast       framework-agent
           4  inbox.list            ring20-mgmt
           2  event.broadcast       (unknown)

  Gate threshold: 1.00% (over 60-day window — T-1166)
```

The **Legacy callers** breakdown (T-1309) names the caller's display_name
for every legacy invocation in the window. This is the *who* dimension —
operators driving migration use it to know which session to migrate next.
`(unknown)` covers entries from clients that didn't supply `from`
(legacy clients pre-T-1309, or methods that don't carry the field).

**Why trend mode is the default.** Don't wait 60 days to find out a
migration didn't land. Watch the 1d / 7d trajectory toward zero from
day one. If `event.broadcast` was supposed to drop after a deploy and
the 1d row stays high, you can investigate within hours.

Exit code: `0` if the **60d** row passes, `1` otherwise — so existing
CI usage is unchanged when invoked with no flag.

### Single-window CI mode (T-1166 gate)

```sh
fw metrics api-usage --last-Nd 60
```

Original behavior — single window, single gate verdict. Use this in
CI scripts that explicitly want the canonical T-1166 check. Exit `0`
if legacy ≤ gate-pct in the window, `1` otherwise.

### Flags

| Flag | Default | Purpose |
|------|---------|---------|
| `--last-Nd N` | (trend mode) | Single window in days. Omit for trend mode. |
| `--runtime-dir PATH` | `$TERMLINK_RUNTIME_DIR` or `/var/lib/termlink` | Hub runtime directory |
| `--gate-pct N` | `1.0` | Threshold below which legacy share passes |
| `--json` | (off) | Machine-readable output for dashboards (T-1312) |

### JSON output (`--json`)

Stable shape — once shipped, downstream integrations depend on it.

```json
{
  "audit_file": "/var/lib/termlink/rpc-audit.jsonl",
  "mode": "trend",
  "gate_pct": 1.0,
  "malformed_lines": 0,
  "windows": [
    {"days": 1, "total": 812, "legacy": 0, "legacy_pct": 0.0, "passing": true},
    {"days": 7, "total": 5904, "legacy": 3, "legacy_pct": 0.0508, "passing": true},
    {"days": 30, "total": 18203, "legacy": 12, "legacy_pct": 0.0659, "passing": true},
    {"days": 60, "total": 33421, "legacy": 29, "legacy_pct": 0.0868, "passing": true}
  ],
  "top_methods": [
    {"method": "channel.post", "count": 28000, "pct": 83.8, "is_legacy": false}
  ],
  "legacy_callers": [
    {"method": "event.broadcast", "from": "framework-agent", "count": 23},
    {"method": "inbox.list", "from": "ring20-mgmt", "count": 4},
    {"method": "event.broadcast", "from": "(unknown)", "count": 2}
  ],
  "gate": {"window_days": 60, "passing": true}
}
```

`mode` is `"trend"` when `--last-Nd` is omitted, `"single-window"` otherwise
(in which case `windows` contains exactly one entry for the requested
`--last-Nd N`). Exit code semantics are preserved across both human and JSON
paths: `0` if the canonical gate (60d in trend mode, the chosen window in
single-window mode) is passing, `1` otherwise. When the audit file is
missing, JSON mode emits `{"error": "audit file not found", "audit_file":
"..."}` to stdout and exits `1`.

### When to look

- **Day 1 after migrating a caller:** check the 1d row. New legacy
  traffic should drop sharply; if it didn't, that caller's deploy
  didn't land or another caller is still using the old API.
- **Week after a migration:** 7d row tells you whether the drop is
  steady or just a low-traffic blip.
- **Before authorizing T-1166 retirement:** all four windows should
  read PASS. The 60d row is the canonical gate; the others give
  early warning if a caller regresses.

## Legacy primitives tracked by the gate

Per T-1166 § Decommission scope:

- `event.broadcast`
- `inbox.list`, `inbox.status`, `inbox.clear`
- `file.send`, `file.receive` (and any `file.send.*` / `file.receive.*` chunked variants)

If you add or rename a method on the migration ladder, update the
`LEGACY` set in `.agentic-framework/agents/metrics/api-usage.sh`.

## Retention

v1 ships **no automatic rotation or pruning**. Operationally:

- Disk pressure is bounded by RPC volume; ~125 bytes/line at fleet
  steady state (~10 RPC/s) ≈ 100 MB/day. Most hubs see 1-2 orders of
  magnitude less.
- Operator-cron should delete `rpc-audit.jsonl` (or rename + truncate)
  once it ages past your retention window. Recommended: keep ≥90d so
  the 60-day default window has headroom.
- A future `rpc_audit::rotate()` based on size + age is a follow-up
  task once a real hub shows pressure.

To check current size:

```sh
ls -lh $TERMLINK_RUNTIME_DIR/rpc-audit.jsonl
```

To prune in place (truncate, keep file):

```sh
: > $TERMLINK_RUNTIME_DIR/rpc-audit.jsonl
```

To archive then truncate (preserves history out-of-band):

```sh
gzip -c $TERMLINK_RUNTIME_DIR/rpc-audit.jsonl > /var/log/termlink/rpc-audit-$(date +%F).jsonl.gz
: > $TERMLINK_RUNTIME_DIR/rpc-audit.jsonl
```

## Real-time deprecation log (T-1311)

The audit log is **retrospective** — operators run `fw metrics api-usage`
to see counts after the fact. T-1311 adds the **real-time** complement:
a `tracing::warn!` at the hub the moment a legacy primitive is dispatched.

Output shape:

```
WARN deprecated method=event.broadcast from=framework-agent T-1166: schedule retirement once legacy <1% over 60d
```

Filter for it in journalctl:

```sh
journalctl -u termlink-hub | grep "T-1166"
```

Or live-tail via `journalctl -fu termlink-hub | grep --line-buffered T-1166`
to watch deprecated calls happen as a migration is rolled out.

**Rate-limited** to one log per `(method, from)` pair per 5 minutes — a
chatty long-running caller spamming `inbox.list` every second floods the
log only on the first call, then again 5 minutes later if it has not
been migrated. `from` shows `(unknown)` for callers that did not
populate the field (legacy clients, methods that don't carry it).

The list of legacy methods is the same set the audit-tally script flags
as `←legacy` in the `fw metrics api-usage` output: `event.broadcast`,
`inbox.{list,status,clear}`, `file.{send,receive}` (and chunked
`file.send.*` / `file.receive.*` variants).

## Hot-path safety

`record()` opens, appends, closes the file synchronously per RPC. Write
errors (full disk, EACCES, missing parent) are logged at debug and
swallowed — the audit never fails an RPC. If the audit file is missing
at hub start the next call creates it.

## Performance notes

A synchronous append per RPC is acceptable at observed traffic rates
(<100 RPC/s/hub).

**Skip-list (T-1307).** Long-poll subscriber methods (`event.poll`,
`event.collect`) are excluded from the audit log. Without the skip,
a single `termlink event collect --timeout 1` invocation would fire
~13,000 audit appends per second, dominating disk I/O and obscuring
the user-meaningful API surface that the T-1166 entry gate cares
about. The skip-list lives in `crates/termlink-hub/src/rpc_audit.rs`
as a `const &[&str]`. Adding/removing entries is the right adjustment
when new transport-plumbing methods land — keep `event.broadcast`,
`event.emit_to`, `channel.post`, `inbox.*`, `file.*` recorded since
those are real API surface.

If you see latency regressions or `tracing` debug output mentioning
rpc_audit at scale, the next iteration should batch via
`tokio::sync::mpsc` + a background writer.

## Implementation

| Component | File |
|-----------|------|
| Audit module | `crates/termlink-hub/src/rpc_audit.rs` |
| Dispatch wire-in | `crates/termlink-hub/src/server.rs` (per-request `record()`) |
| Init at bootstrap | `crates/termlink-hub/src/server.rs` alongside `topic_lint::init` |
| Tally script | `.agentic-framework/agents/metrics/api-usage.sh` |
| `fw` routing | `.agentic-framework/bin/fw` (under `metrics` subcommand) |
