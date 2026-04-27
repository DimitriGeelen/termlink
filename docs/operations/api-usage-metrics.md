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
{"ts":1714234567890,"method":"event.broadcast"}
```

- `ts` — UNIX milliseconds (UTC)
- `method` — the JSON-RPC `method` field as the client sent it (no
  normalization)

The hub records every parseable request — including auth attempts and
permission-denied calls. The audit log answers "what was asked of the
hub", not "what succeeded". This is intentional: a flood of
`event.broadcast` calls that all fail auth still indicates a stale
caller that hasn't migrated.

## Reading the report

```sh
fw metrics api-usage --last-Nd 60
```

```
== fw metrics api-usage ==
  Audit file: /var/lib/termlink/rpc-audit.jsonl
  Window:     last 60 days
  Total RPCs: 12345

  Top 10 methods:
       9876   80.0%  channel.post
       1234   10.0%  event.subscribe
        ...

  Legacy primitives: 12 (0.10% of total)
  Gate threshold:    1.00%  →  PASS
```

Exit code: `0` if legacy ≤ gate, `1` otherwise. The gate makes this
suitable as a CI check before authorising T-1166 retirement.

### Flags

| Flag | Default | Purpose |
|------|---------|---------|
| `--last-Nd N` | `60` | Time window in days |
| `--runtime-dir PATH` | `$TERMLINK_RUNTIME_DIR` or `/var/lib/termlink` | Hub runtime directory |
| `--gate-pct N` | `1.0` | Threshold below which legacy share passes |

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
