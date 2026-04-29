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
you when `--targets` is empty. Per-target fan-out (`--targets a,b,c`)
still uses `event.broadcast` until T-1166 cuts the router method — at
which point the CLI will need a separate replacement (planned: parallel
emit_to calls). Most callers don't use `--targets`.

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

On the hub host, the audit log records every method dispatch:

```bash
# Count legacy calls in the last 24h
fw metrics api-usage --last-Nd 1

# Or grep the audit log directly:
jq -r 'select(.method | test("^(event.broadcast|inbox\\.|file\\.send|file\\.receive)$")) | .method' \
  /var/lib/termlink/rpc-audit.jsonl | sort | uniq -c | sort -rn
```

For client-side hunting (your own session is one of the named callers in
the report), grep your codebase:

```bash
git grep -nE 'event\.broadcast|event_broadcast|inbox\.(list|status|clear)|file\.send|file\.receive' \
  -- 'crates/' 'lib/' 'skills/' '*.py' '*.rs' '*.ts'
```

Exclude protocol constants, deprecation shims, and test fixtures from
your hit count.

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
