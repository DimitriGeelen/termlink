---
id: T-1174
name: "Wire termlink channel post through BusClient so CLI gets offline-queue tolerance"
description: >
  T-1161 follow-up. Smoke-test on 2026-04-21 surfaced that the CLI `channel post` verb bypasses BusClient and calls rpc_call directly at crates/termlink-cli/src/commands/channel.rs:137. Operators running `termlink channel post` while the hub is down get an RPC error instead of offline-queue fallback. Swap to BusClient::post so CLI ops inherit the durable queue. Keep the existing rpc_call path as fallback when no identity/queue is available.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-1155, bus, cli, offline-queue]
components: []
related_tasks: []
created: 2026-04-20T23:09:11Z
last_update: 2026-04-20T23:19:43Z
date_finished: 2026-04-20T23:19:43Z
---

# T-1174: Wire termlink channel post through BusClient so CLI gets offline-queue tolerance

## Context

Follow-up to T-1161. The T-1155 bus-stack smoke test on 2026-04-21 surfaced that the CLI `termlink channel post` verb at `crates/termlink-cli/src/commands/channel.rs::cmd_channel_post` calls `client::rpc_call` directly instead of routing through the `BusClient` abstraction built in T-1161. Net effect: when the hub is down, operators running `termlink channel post` see a raw RPC error and the payload is lost. The durable SQLite queue exists but only programmatic `termlink-session` library consumers (future SDK users, integration tests) benefit from it.

Fix: build a `PendingPost` in `cmd_channel_post` and route through `BusClient::post`. On `PostOutcome::Delivered` print the existing "Posted to {topic} — offset=N" line. On `PostOutcome::Queued` print a clear "Queued to {topic} — queue_id=N (hub unreachable; will flush on next reconnect)" line. Keep existing flags (`--json`, `--msg-type`, `--payload`, `--artifact-ref`, `--sender-id`, `--hub`) unchanged.

Shape of the change (reviewer crib):
- Imports: add `use termlink_session::bus_client::{BusClient, PostOutcome};` + `use termlink_session::offline_queue::{default_queue_path, PendingPost};`
- Replace the `client::rpc_call(&sock, method::CHANNEL_POST, params)` call with a `BusClient::connect(sock, default_queue_path())?` + `client.post(pending).await?`
- The `params` json! construction disappears — `BusClient::post` takes a typed `PendingPost` and serialises internally (identical wire format; validated by T-1161 integration test).

## Acceptance Criteria

### Agent
- [x] `cmd_channel_post` in `crates/termlink-cli/src/commands/channel.rs` constructs a `PendingPost` from its args (topic, msg_type, payload, artifact_ref, ts_unix_ms, resolved sender_id, identity pubkey hex, signature hex) and delegates to `BusClient::post` instead of calling `client::rpc_call(method::CHANNEL_POST, ...)` directly
- [x] Queue path resolves via `offline_queue::default_queue_path()` (which already respects `$TERMLINK_IDENTITY_DIR` per T-1161)
- [x] CLI output for `PostOutcome::Delivered` is unchanged (`Posted to {topic} — offset={N}, ts={T}`) so existing smoke tests and human muscle memory keep working
- [x] CLI output for `PostOutcome::Queued` is a new, clearly labelled line: `Queued to {topic} — queue_id={N} (hub unreachable; will flush on next reconnect)`
- [x] `--json` flag still works in both cases; JSON shape is `{"delivered": {"offset": N, "ts": T}}` vs `{"queued": {"queue_id": N, "queue_path": "..."}}` (new but small, backward-compatible for consumers who key off `"offset"` vs `"queue_id"`)
- [x] Live smoke test run by agent (see transcript in Updates below): hub up → Delivered; hub down → Queued x3 → pending:3; hub restart → next CLI post drains all 3 in FIFO then delivers its own at offset=3; pending:0; subscribe shows perfect FIFO
- [x] `cargo build --workspace` + `cargo test --workspace --lib` (708 tests) + `cargo clippy --workspace --tests -- -D warnings` all pass

## Verification

cargo build --workspace
cargo test --workspace --lib
cargo clippy --workspace --lib --tests -- -D warnings
grep -q "BusClient" crates/termlink-cli/src/commands/channel.rs
grep -q "PostOutcome::Queued" crates/termlink-cli/src/commands/channel.rs
grep -q "will flush on next reconnect" crates/termlink-cli/src/commands/channel.rs

## Decisions

### 2026-04-21 — Opportunistic flush before post, not after
- **Chose:** `cmd_channel_post` calls `client.flush().await` BEFORE issuing its own `post(pending)`, but only when `queue_size() > 0`
- **Why:** CLI is one-shot — the BusClient's background 5 s flush task never gets a tick before process exit. Without inline flush, the queue would grow indefinitely across CLI invocations. Flushing *before* the new post preserves FIFO (queued items land first, new one takes the next offset), and reports "Drained N queued post(s) from previous offline period" so operators see the catch-up
- **Rejected:** (a) Flush after post — would reorder offsets (new post gets offset N, queued items get N+1..N+K, but they were enqueued earlier — semantic drift). (b) Leave the async flush task alone — works only for long-running consumers; CLI users would accumulate queue forever. (c) Add a dedicated `termlink channel flush` verb — more surface area for the same thing a post call can do implicitly

### 2026-04-21 — Add `hub_socket_soft` instead of relaxing `hub_socket`
- **Chose:** New `hub_socket_soft(hub)` returns the path unconditionally; `cmd_channel_post` uses it; `subscribe`/`list`/`create` keep the strict `hub_socket(hub)?` that bails on missing socket
- **Why:** Only `post` has an offline fallback. `subscribe`/`list`/`create` genuinely can't proceed without a live hub (there's no local state to return). Keeping the strict resolver on those paths preserves the clear "hub not running" error that ops expect
- **Rejected:** Relaxing `hub_socket` across all verbs — would make `subscribe` silently return empty when the hub is down, hiding a failure mode operators need to see

## Updates

### 2026-04-20T23:09:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1174-wire-termlink-channel-post-through-buscl.md
- **Context:** Initial task creation

### 2026-04-20T23:10:29Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-21 — smoke-test transcript (workspace binary 0.9.256 in isolated tempdir hub)

(a) hub UP → post delivers:
```
$ termlink channel post broadcast:global --msg-type smoke --payload '{"when":"hub-up"}'
Posted to broadcast:global — offset=0, ts=1776726871011

$ termlink channel post ... --json
{ "delivered": { "offset": 1, "ts": 1776726871024 } }
```

(b) hub DOWN → post queues:
```
$ termlink hub stop        → Hub stopped.
$ termlink channel post ...   → Queued to broadcast:global — queue_id=1 (hub unreachable; will flush on next reconnect)
$ termlink channel post ...   → Queued to broadcast:global — queue_id=2 (hub unreachable; ...)
$ termlink channel post ... --json
{ "queued": { "queue_id": 3, "queue_path": "/tmp/.../outbound.sqlite" } }
$ termlink channel queue-status
queue:    /tmp/.../outbound.sqlite
pending:  3
oldest:   id=1 topic=broadcast:global msg_type=smoke ts_ms=1776726872643 sender=547fd4fe6ed1b863
```

(c) hub restart + next CLI post drains FIFO, then delivers its own:
```
$ termlink hub start        → listening...
$ termlink channel post broadcast:global --msg-type smoke --payload '{"n":"next"}'
Drained 3 queued post(s) from previous offline period
Posted to broadcast:global — offset=3, ts=1776727087797

$ termlink channel queue-status       → pending:  0

$ termlink channel subscribe broadcast:global --limit 10
[0] 547fd4fe6ed1b863 smoke: {"n":1}
[1] 547fd4fe6ed1b863 smoke: {"n":2}
[2] 547fd4fe6ed1b863 smoke: {"n":3}
[3] 547fd4fe6ed1b863 smoke: {"n":"next"}
```

Perfect FIFO across offline → online transition. `sender_id` on every envelope matches the identity fingerprint (queued + fresh posts alike — signing happens once at enqueue time, survives restart).

### 2026-04-20T23:19:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
