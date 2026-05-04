# Migration Guide — Retire legacy `event.broadcast`, `inbox.*`, `file.*`

**Status:** scheduled — gated on T-1166 entry-gate (bake window).
**Tracking task:** [T-1166](../../.tasks/) (in this repo, `.tasks/active/T-1166-*.md` until it lands).
**Predecessor migrations that already shipped:** T-1162 (broadcast→channel mirror),
T-1163 (inbox→channel mirror), T-1164 (file→channel artifacts), T-1400
(doctor inbox.status migration), T-1401 (broadcast→channel.post wrapper).

This document tells consumers of the TermLink JSON-RPC API what changes
when legacy primitives are removed, and how to migrate ahead of the cut.

## Audience

Anyone whose code calls TermLink hub RPC methods directly. Specifically:

- ring20-management, ring20-dashboard (TermLink fleet hubs)
- ntb-atc-plugin (NTB ATC integration)
- 999-Agentic-Engineering-Framework (`framework-agent` session)
- skills-manager
- Any operator script or ad-hoc shell that runs `termlink event broadcast`,
  `termlink inbox *`, or `termlink file send/receive`

If your code only calls the high-level CLI or MCP tools (not raw RPC), you
likely don't need to do anything — the wrappers already point at the
channel-based replacements. Verify by reading the per-method recipe below.

## What's Removed

The following hub RPC methods are removed in the next protocol version:

| Removed RPC | Replacement |
|---|---|
| `event.broadcast` | `channel.post(topic="broadcast:global", msg_type=<original>)` |
| `inbox.list` | `channel.list(prefix="inbox:")` |
| `inbox.status` | `channel.list(prefix="inbox:")` (per-target counts in result) |
| `inbox.clear` | `channel.clear(topic="inbox:<target>")` |
| `file.send` (legacy) | `channel.post(topic="file:<target>", msg_type="file.init"/"file.chunk"/"file.complete")` |
| `file.receive` (legacy) | `channel.subscribe(topic="file:<target>")` and reassemble |

The same wire-shape is already produced by the hub-side mirror shims (T-1162,
T-1163, T-1164), so subscribers see identical envelopes whether they came
from a legacy call or a direct channel.post. The cut is on the **producer**
side: callers must stop hitting the legacy method names.

## Migration Recipe

### `event.broadcast` → `channel.post(broadcast:global)`

Legacy call (removed):

```json
{
  "method": "event.broadcast",
  "params": {
    "topic": "deploy.start",
    "payload": {"version": "1.2.3"},
    "from": "tl-mysession"
  }
}
```

Replacement (signed channel.post):

```json
{
  "method": "channel.post",
  "params": {
    "topic": "broadcast:global",
    "msg_type": "deploy.start",
    "payload_b64": "<base64(JSON.stringify(payload))>",
    "ts": <unix_ms>,
    "sender_id": "<your-identity-fingerprint>",
    "sender_pubkey_hex": "<32-byte ed25519 pubkey hex>",
    "signature_hex": "<64-byte ed25519 signature hex>",
    "metadata": {"from": "tl-mysession"}
  }
}
```

The signature is over `canonical_sign_bytes(topic, msg_type, payload, artifact_ref, ts)`
— see `termlink_protocol::control::channel::canonical_sign_bytes` for the
exact byte sequence. The `metadata.from` field is NOT part of the signed
bytes (routing-only per T-1287 / trusted-mesh threat model).

The `termlink event broadcast` CLI was rewritten in T-1401 to do this for
you when `--targets` is empty. **T-1417 (2026-04-30) migrated the
per-target fan-out path** (`--targets a,b,c`) to parallel `event.emit_to`
calls, so the CLI no longer hits `event.broadcast` for any input. The
same migration was applied to the `termlink_broadcast` MCP tool. Result
shape preserved: `{topic, targeted, succeeded, failed[, errors]}`.

### `inbox.list` / `inbox.status` → `channel.list(prefix="inbox:")`

Legacy call:

```json
{"method": "inbox.list", "params": {}}
{"method": "inbox.status", "params": {}}
```

Replacement:

```json
{"method": "channel.list", "params": {"prefix": "inbox:"}}
```

Result shape:

```json
{
  "topics": [
    {"name": "inbox:tl-alice", "count": 3, "retention": {"kind": "messages", "value": 1000}},
    {"name": "inbox:tl-bob",   "count": 0, "retention": {"kind": "messages", "value": 1000}}
  ]
}
```

To replicate `inbox.status` (per-target counts), aggregate the `count`
field per topic — each `inbox:<target>` topic's count IS the pending
message count for that target.

`fw doctor` and the `termlink_doctor` MCP tool were migrated in T-1400.
The CLI does this transparently; only direct RPC callers need to update.

### `inbox.clear` → `channel.clear`

```json
{"method": "channel.clear", "params": {"topic": "inbox:<target>"}}
```

Topic-scoped, not target-scoped. To clear inbox for multiple targets,
issue one `channel.clear` per `inbox:<target>` topic.

### `file.send` / `file.receive` → `file.init` / `file.chunk` / `file.complete` envelopes

The legacy `file.send` / `file.receive` RPCs have already been replaced
in the CLI and MCP wrappers (T-1164). The replacement is a sequence of
channel.post envelopes on `file:<target>`:

1. `msg_type = "file.init"` — header, with `name`, `size`, `sha256`
2. `msg_type = "file.chunk"` — body, with `seq`, `data_b64`
3. `msg_type = "file.complete"` — trailer, with `sha256` (final check)

Receivers subscribe to `file:<target>` and reassemble. Use the
`termlink file send` / `termlink file receive` CLI commands or the
`termlink_file_send` / `termlink_file_receive` MCP tools — both are
already wired to the channel protocol.

## Capability Handshake Change

After T-1166 lands, the hub's `hub.capabilities` response includes:

```json
{
  "methods": [...],
  "hub_version": "...",
  "protocol_version": 1,
  "features": {"legacy_primitives": false}
}
```

The `features.legacy_primitives` flag is currently advertised as `true`
(T-1405 shipped 2026-04-29, before T-1166 lands). Downstream consumers
should wire startup checks NOW against the existing `true` value — when
the cut happens, the value flips to `false` and the failure path trips
automatically.
Clients that depend on legacy methods should check this key on connect
and fail fast with a clear error pointing at this guide instead of
discovering the method removal at first-call time.

Pre-T-1166 clients that don't check the capability will receive
`-32601 method_not_found` from the hub on legacy calls — same as any
other unknown method. Existing error-handling paths apply.

## Timeline

The cut is gated on `fw metrics api-usage --last-60d` showing
≤1.0% legacy traffic over the 60-day window. Status check:

```bash
fw metrics api-usage
```

When the trend reads `Status: PASS` at 60d, T-1166 promotes from
`captured` to `started-work` and the operator runs the cut procedure
below. Source cleanup (deleting `handle_event_broadcast`, the inbox
handlers, and the 6 client-side fallback paths that were allowlisted
in T-1406) is a separate follow-up task because the flag-off behavior
is already test-proven; that work carries no risk and can land at the
operator's convenience.

Downstream consumers should aim to land their migrations
**before** the gate passes, so the cut itself is a no-op for them.

## Operator Cut Procedure

T-1411 staged the cut so it is a single-character source change. The
procedure on the hub host:

1. **Verify the bake gate has passed:**
   ```bash
   .agentic-framework/bin/fw metrics api-usage    # 60d window must show PASS
   ```
2. **Confirm Tier-2 authorization** has been recorded (the cut is not
   self-delegated by an agent — the human must explicitly approve it).
3. **Pre-verify the OFF path passes CI** (T-1413):
   ```bash
   cargo test -p termlink-hub --lib --features legacy_primitives_disabled
   # expect: test result: ok. <N> passed; 0 failed
   ```
   This runs the same suite with `LEGACY_PRIMITIVES_ENABLED=false` baked in
   at compile time, including `cut_path::*` tests that exercise the
   capabilities response, the methods-array filter, and the route-level
   rejection. If this is green, the cut works; if red, fix before flipping.
4. **Edit the const expression** in `crates/termlink-hub/src/router.rs`:
   ```rust
   // Either: hardcode false directly:
   pub(crate) const LEGACY_PRIMITIVES_ENABLED: bool = false;
   // Or (equivalent): build with the feature on:
   //   cargo build --release -p termlink --features termlink-hub/legacy_primitives_disabled
   ```
5. **Build and install:**
   ```bash
   cd /opt/termlink
   cargo build --release -p termlink
   cp -f target/release/termlink /root/.cargo/bin/termlink
   sudo systemctl restart termlink-hub.service
   ```
6. **Verify capabilities reflects the cut:**
   ```bash
   python3 -c "import socket, json; \
     s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM); \
     s.connect('/var/lib/termlink/hub.sock'); \
     s.sendall(b'{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"hub.capabilities\",\"params\":{}}\n'); \
     d = json.loads(s.recv(8192).decode().split('\n')[0]); \
     print(json.dumps(d['result']['features'], indent=2))"
   # expect: {"legacy_primitives": false}
   ```
7. **Smoke-test rejection:** call any retired method (e.g.
   `event.broadcast`) — the response must be JSON-RPC error code
   `-32601` with a message naming the migration target.
8. **Commit + push the source change** with a `T-1166: cut — flip
   LEGACY_PRIMITIVES_ENABLED` commit message.

After the cut:

- The hub keeps serving every other method exactly as before.
- Legacy method names are filtered out of `hub.capabilities.methods[]`,
  so consumers that registered T-1405 startup checks will fail-fast on
  next start.
- Open a follow-up task to delete the now-dead handler functions
  (`handle_event_broadcast`, `handle_inbox_*`) and the 6 client-side
  fallback paths that T-1406 currently allowlists. The T-1406
  ALLOWLIST shrinks to zero in that follow-up.

## Diagnostic — am I still calling legacy methods?

The fastest answer (T-1459/T-1460/T-1461/T-1467, since 2026-05-04) is one
command. Per-hub top-callers come from the hub binary if it includes T-1460;
otherwise (the entire 0.9.0 fleet) the CLI derives them from `by_method` on
the client side (T-1467) — no hub upgrade needed for this view:

```bash
termlink fleet doctor --legacy-usage --legacy-window-days 1
```

Read the verdict line:

| Verdict              | Meaning                                                                  | Operator action |
|----------------------|--------------------------------------------------------------------------|-----------------|
| `CUT-READY`          | All hubs report zero legacy traffic.                                     | Safe to flip the cut flag. |
| `CUT-READY-DECAYING` | Residue exists but no live caller in the last 5 minutes.                 | May cut now (residue is historical) or wait for the audit window to roll. |
| `WAIT`               | At least one hub had a legacy call in the last 5 minutes.                | Live caller — DO NOT cut. Use top-callers below to identify it. |
| `UNCERTAIN`          | Some hubs are pre-T-1432 or have no audit yet. Cut-readiness unknown.    | Upgrade or wait for traffic. |

The same command shows top callers — both per-hub and a fleet-wide
aggregate — so the operator sees who is producing the residue without
SSHing each host:

```
WITH TRAFFIC:
  local-test: 579 legacy invocation(s) — last call 1h ago (decay residue)
    └─ 579× addr:192.168.10.121
  ring20-management: 579 legacy invocation(s) — last call 1h ago (decay residue)
    └─ 579× addr:192.168.10.121
  workstation-107-public: 579 legacy invocation(s) — last call 1h ago (decay residue)
    └─ 579× addr:192.168.10.121
  Top callers (fleet-wide):
    1737× addr:192.168.10.121
```

The `addr:<ip>` form means the audit log carried a `peer_addr` for that
caller (post-T-1409 fleet); `<label>` would mean `from` was set
(post-T-1427 caller); `pid:<n>` is a Unix-socket caller without a
`from` field. The IP-only normalization strips ephemeral source ports
so 100 reconnects from the same host do not show as 100 distinct
callers.

### Decay-rate sampling (T-1462 / T-1463, since 2026-05-04)

Point-in-time counts only answer "is there residue right now?" To answer
"is the residue actually clearing?" capture a snapshot today and diff
against a prior one. The CLI does this without any hub upgrade; the
existing `legacy_summary` block carries everything `--diff` needs.

```bash
# Daily capture (cron-friendly: still prints human-readable verdict to
# stderr while saving the JSON to disk for tomorrow's diff).
mkdir -p /var/lib/termlink/snapshots
termlink fleet doctor --legacy-usage \
  --save-snapshot /var/lib/termlink/snapshots/$(date +%F).json

# Tomorrow: compare against yesterday's snapshot.
termlink fleet doctor --legacy-usage \
  --diff /var/lib/termlink/snapshots/$(date -d yesterday +%F).json
```

The diff block surfaces: fleet `total_legacy` delta with arrow (↑/↓/→),
average rate (calls/min) over the elapsed interval, per-hub deltas
(NEW / VANISHED / explicit ±N), and per-caller deltas from
`top_callers_fleet`. Zero-information rows are suppressed so a flat
fleet shows nothing under the per-hub/caller subsections rather than a
wall of noise.

Read the rate as the cut-readiness yardstick:

| Rate sign | Meaning |
|-----------|---------|
| Negative (decay) | Residue is clearing on its own. Cut may be safe even from `CUT-READY-DECAYING` if extrapolated time-to-zero is acceptable. |
| Zero (flat) | Audit log is in a steady state — the rolling window is dropping calls at the same rate new ones arrive. Investigate top callers. |
| Positive (growing) | Live caller still polling somewhere. `WAIT` verdict expected; do not cut. |

**Rate interpretation caveat (audit-log window roll-off).** The rate is
computed naïvely as `total_fleet_delta / elapsed_minutes`. The hub's
audit log is a rolling window (currently 60–90d depending on hub
retention config). When the elapsed interval between two snapshots
crosses a meaningful fraction of that window, some of the apparent
"decay" is just calls aging out of the window, not callers stopping.

For migration tracking, prefer **short intervals** to keep this effect
negligible:

| Interval     | Rolloff contribution to rate | Recommended use |
|--------------|------------------------------|-----------------|
| ≤ 1 day      | < 2% of window — negligible. | Daily cron, primary cut-readiness signal. |
| 1–7 days     | 2–10% — present but small.   | Trend confirmation; treat decay rate as a lower bound on real migration progress. |
| > 7 days     | > 10% — material.            | For long-term context only; do not infer caller behaviour from a multi-week rate. Re-snapshot recently. |

When in doubt, take a fresh snapshot today and diff against yesterday.
The 1d interval keeps roll-off out of the picture and the rate becomes
a clean signal of caller activity.

### Cron/CI integration (T-1465, since 2026-05-04)

For automated pipelines, parsing the JSON to gate on the verdict is
overkill. `--exit-code-on-verdict` maps the verdict to a process exit
code so a shell script (or CI step) can branch on it directly:

| Verdict              | Exit code | Cron interpretation |
|----------------------|-----------|---------------------|
| `CUT-READY`          | 0         | Safe — proceed with cut. |
| `CUT-READY-DECAYING` | 0         | Acceptable — residue is historical, no live callers. |
| `WAIT`               | 10        | Live caller present — retry later. |
| `UNCERTAIN`          | 11        | Hub upgrade or audit-window age-out needed. |

Connectivity failures keep their existing non-zero exit (precedence over
verdict mapping) — a hub that didn't connect means the verdict was built
from incomplete data, so the script doesn't blindly gate on a partial
view of the fleet.

Canonical use:

```bash
# Daily cron — only proceed with downstream actions if cut-readiness ok.
termlink fleet doctor --legacy-usage --exit-code-on-verdict
case $? in
  0)  echo "Cut-readiness OK — proceeding with downstream cut steps." ;;
  10) echo "Live legacy caller present — sleeping until next run." ;;
  11) echo "Verdict UNCERTAIN — operator action needed." ; exit 1 ;;
  *)  echo "Connectivity failure — fleet sweep incomplete." ; exit 2 ;;
esac
```

For older fleets (or to inspect the JSON shape):

```bash
# Count legacy calls in the last 24h
fw metrics api-usage --last-Nd 1

# Read the JSON shape (T-1414 attribution split):
fw metrics api-usage --last-Nd 7 --json | jq '{
  legacy, legacy_attributable, legacy_unattributable_pre_t1409,
  callers: .legacy_callers_by_ip
}'

# Or grep the audit log directly:
jq -r 'select(.method | test("^(event.broadcast|inbox\\.|file\\.send|file\\.receive)$")) | .method' \
  /var/lib/termlink/rpc-audit.jsonl | sort | uniq -c | sort -rn
```

**Read the attribution split correctly (T-1414).** Pre-T-1409-deploy lines
on TCP callers carry no `peer_addr`/`peer_pid`/`from` — they appear as
"(unknown)" in the legacy_callers list. The bake-decision number is
`legacy_attributable`, not `legacy`. The unattributable backlog ages out of
the rolling window naturally (60d after T-1409 deployed).

**Distinguish live from stale callers (T-1419, since 2026-04-30).** Each
row in `legacy_callers`, `legacy_callers_by_pid`, and `legacy_callers_by_ip`
now carries `last_seen_ts_ms` (int) and `last_seen_iso` (UTC string). If a
peer's `last_seen_iso` is older than your most recent restart of that
peer's polling agent, the count is stale rolling-window residue — the
migration succeeded; the count will drop to zero as the window rolls.
This is the primary signal for verifying a post-deploy cut-blocker
clearance (T-1418 uses it).

For client-side hunting (your own session is one of the named callers in
the report), grep your codebase:

```bash
git grep -nE 'event\.broadcast|event_broadcast|inbox\.(list|status|clear)|file\.send|file\.receive' \
  -- 'crates/' 'lib/' 'skills/' '*.py' '*.rs' '*.ts'
```

Exclude protocol constants, deprecation shims, and test fixtures from
your hit count.

### Identifying the caller behind a peer_addr

If you ran the T-1460-aware `fleet doctor --legacy-usage` above, you
already see `addr:<ip>` directly under each hub and at the fleet-wide
top-callers line. Skip ahead to the TLS-fingerprint rollover note
below if you need to identify the role behind a renumbered IP.

The `legacy_callers_by_ip` rollup names the source IP. If the IP is also
running termlink-hub, its TLS fingerprint is the most stable identifier
(persistent across container re-numbers under T-985 / T-1028 persist-if-present):

```bash
# Get the TLS fingerprint of the suspect peer's hub
python3 -c "
import socket, ssl, hashlib
raw = socket.create_connection(('SUSPECT_IP', 9100), timeout=3)
ctx = ssl.create_default_context()
ctx.check_hostname = False
ctx.verify_mode = ssl.CERT_NONE
ss = ctx.wrap_socket(raw, server_hostname='SUSPECT_IP')
fp = hashlib.sha256(ss.getpeercert(binary_form=True)).hexdigest()
print('sha256:' + fp)
"

# Cross-reference against ~/.termlink/known_hubs to identify the role
grep '^[0-9].*sha256:53de15ec' ~/.termlink/known_hubs
```

If the fingerprint matches a known hub at a different IP, the container has
been re-numbered. Look up the role by fingerprint, not by IP.

For the dashboard agent specifically (canonical T-1166 holdout pattern):
the dashboard polls `inbox.status` on a ~60s cadence. Migration is to
upgrade its `termlink-cli` to a build containing T-1235 (the
`inbox_channel::status_with_fallback` dual-read shim). Once the shim lands,
polls switch to `channel.list(prefix="inbox:")` and legacy traffic from
that caller drops to zero within one polling interval. **No hub-side change
is required for the migration itself.**

## Roll-Forward Checklist (consumer-side)

For each consumer project:

1. **Audit:** grep your code for the legacy method names. List remaining
   call-sites.
2. **Migrate:** swap each call-site to the channel-based replacement
   shown above. The hub-side wire shape is identical to what the legacy
   mirror shims already produce, so subscribers don't need to change.
3. **Test:** run your test suite. Hit a hub running the new code locally
   to confirm the cut works.
4. **Audit-log diff:** before and after running your test suite, snapshot
   `rpc-audit.jsonl` line count and `grep` for the migrated method names.
   The replacement runs should show channel.* methods only.
5. **Capability check:** add a startup check that fails fast if the hub
   advertises `features.legacy_primitives = false` and you still have a
   legacy call-site you missed. (Available now — see T-1405. While the
   cut hasn't landed the value is `true`; your failure path will trip
   automatically when it flips.)
6. **Bake:** run the migrated code in production for ≥7 days before the
   T-1166 cut, so any edge case surfaces while the legacy path is still
   available as a fallback.

## Roll-Back

T-1411 made the cut reversible until the source-cleanup follow-up
lands. The flag-flip itself can be undone by setting
`LEGACY_PRIMITIVES_ENABLED` back to `true`, rebuilding, and restarting
the hub — capabilities flips back, the (still-present) handler functions
serve again, and any caller still hitting a legacy method works as
before.

Once the source-cleanup follow-up ships (handler functions deleted,
client-side fallbacks removed), there is no roll-back: the methods
are gone. Roll-forward only at that point. If you discover a missed
call-site after the cleanup, the failure mode is `method_not_found`
which surfaces immediately at first-call time, not a silent data drop.
Fix the call-site and ship.

The recommendation is to leave the cut in flag-off state for at least
one bake cycle (≥7 days) before shipping the source-cleanup, so a
genuinely-broken consumer can be discovered and either fixed or
temporarily un-cut without a code surgery.

## References

- T-1155 (decommission strategy umbrella)
- T-1162 (event.broadcast → channel mirror)
- T-1163 (inbox.* → channel mirror)
- T-1164 (file.* → channel artifacts)
- T-1300 / T-1301 (topic-lint warnings on legacy calls)
- T-1304 (`<runtime_dir>/rpc-audit.jsonl` telemetry surface)
- T-1311 (`fw metrics api-usage` agent)
- T-1400 (doctor inbox.status migration)
- T-1401 (broadcast → channel.post wrapper, CLI)
- T-1403 (broadcast → channel.post wrapper, MCP — sibling miss)
- T-1405 (`features.legacy_primitives` capability flag — pre-staged)
- T-1406 (regression-guard test forbidding new in-repo direct callers)
- T-1407 (rpc-audit `peer_pid` for Unix-socket callers)
- T-1408 (`fw metrics api-usage` peer_pid breakdown — anonymous-caller forensics)
- T-1409 (rpc-audit `peer_addr` for TCP callers — closes the network-side blind spot)
- T-1410 (api-usage agent — IP rollup, ports stripped)
- T-1411 (hub-side flag-gated rejection — single-const cut, this section's `## Operator Cut Procedure`)
- T-1413 (cargo-feature-driven const + OFF-path test suite for CI verification of the cut)
