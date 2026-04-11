# T-923: Hub forwarder discovery — session.forward already exists

**Task:** T-923 — Hub session.forward RPC
**Parent inception:** T-921 (Full cross-host parity — GO, Option A + C)
**Status:** discovery + end-to-end test + scope assessment complete
**Date:** 2026-04-11

## TL;DR

The planned new `session.forward` hub RPC method is **not needed**. The hub router
already transparently forwards any non-hub-local method to the named session via
`forward_to_target` (`crates/termlink-hub/src/router.rs`). T-923 therefore closes
as: discovery + end-to-end integration test + scope assessment + this report. T-924
(the shared CLI `TargetOpts` + `call_session` helper) can build directly on this
mechanism without any new hub surface.

## Where the forwarder lives

- `crates/termlink-hub/src/router.rs` — `route()` dispatch table at the top of the
  function. All hub-local methods (discover, broadcast, collect, register_remote,
  orchestrator.route, etc.) have explicit match arms. The fallthrough arm is
  `_ => forward_to_target(req, id).await,` — **any unknown method** gets forwarded.
- `forward_to_target` at `router.rs:1171-1230`:
  - Reads `params.target` (required; errors with `SESSION_NOT_FOUND` if missing)
  - Resolves it via `manager::find_session(target)` (local FS registry lookup by
    session ID or display name) — if that fails, falls back to
    `remote_store().get(target)` / `.list_live().find(…)` (display name or id match)
  - Dials the resolved `TransportAddr` (Unix for local, TCP for remote) via
    `client::Client::connect_addr(&addr).await`
  - Calls `c.call(&req.method, id.clone(), req.params.clone()).await`
  - Propagates success/error back to the originating connection unchanged

This means:

1. A new CLI command gets cross-host routing **for free** — if the client can
   reach the hub and name the target session, any method the session exposes is
   forwardable. No per-method wrapper required.
2. T-924's CLI helper `call_session(opts, method, params)` just needs to
   (a) connect to the hub via `commands::remote::connect_remote_hub` when a
   `--target HOST:PORT` is set, and (b) stuff `session` into `params.target`.

## Scope enforcement assessment

### The worry

At discovery time the question was: does the forwarder re-check the connection's
scope against the forwarded method's required scope, or does it silently dial
the target regardless? The latter would be a security gap — any authenticated
connection with the weakest scope (`observe`) could call `command.inject`
through the forwarder path.

### The finding

Scope enforcement is done **before** reaching `forward_to_target` at
`crates/termlink-hub/src/server.rs:505-534`:

```rust
Some(scope) => {
    let required = hub_method_scope(&req.method);
    if !scope.satisfies(required) {
        // → AUTH_DENIED (-32010) "Permission denied: '<method>' requires …"
    } else {
        router::route(&req).await  // ← only reached once scope satisfies
    }
}
```

The helper `hub_method_scope` (`server.rs:216-233`) handles the explicit
hub-local methods, then falls through to `auth::method_scope(method)` for any
method the router will forward. `auth::method_scope` has the same scope table
as a local session applies for itself:

- `termlink.ping`, `query.*`, `kv.get`, `event.poll` → `Observe`
- `kv.set`, `kv.delete`, `event.emit`, `session.update` → `Interact`
- `command.run`, `command.inject`, `session.spawn` → `Control` / `Execute`

So the same scope gate a local session enforces on its own Unix socket is
also enforced at the hub for cross-host calls. **No gap.**

### Confirmation test

`tcp_forward_rejected_when_scope_insufficient` (in the same `router.rs` test
module) authenticates a TCP connection with only `Observe` scope and then
attempts to forward `kv.set` (which requires `Interact`). The hub rejects the
call at the scope gate with `AUTH_DENIED (-32010)` and the `kv.set` never
reaches `forward_to_target`. This test is the regression guard: if someone ever
moves scope enforcement after the router dispatch, this test will fail.

## End-to-end integration test

Previously the existing `forward_to_target_session` and
`forward_to_remote_session_via_tcp` tests exercised `forward_to_target`
directly by calling `route(&req)` inside the test process. Neither drove the
full TCP hub accept loop + `hub.auth` + scope gate + forward path. T-923 adds
that coverage:

**`tcp_forward_to_local_session_after_auth`** (`router.rs` tests module):

1. Start a local session via `start_test_session(&sessions_dir, "fwd-tcp-local")`
2. Set `TERMLINK_RUNTIME_DIR` so `manager::find_session` finds it
3. Start the hub with TCP binding via `start_hub_with_tcp(&dir)` (reuses the
   `run_accept_loop` path with `Some(tcp_listener)` + a generated HMAC secret)
4. Open a TCP connection, call `hub.auth` with an `Interact`-scoped token
   (via `tcp_connect_and_auth` helper)
5. Send `{"method": "termlink.ping", "params": {"target": "fwd-tcp-local"}}`
6. Assert response carries the local session's `display_name` and `state`

This is the first test that proves the complete cross-host flow — TCP accept →
`hub.auth` → scope gate → `router::route` fallthrough → `forward_to_target` →
local session Unix socket → response — works end-to-end in a single test
process. It is the mechanical prerequisite for T-924 and the T-925..T-935
rollout.

Both new tests share the existing `ENV_LOCK` mutex (other tests also mutate
`TERMLINK_RUNTIME_DIR`) and reuse the existing `start_hub_with_tcp` +
`tcp_connect_and_auth` helpers so no duplicated scaffolding was introduced.

## Path this enables for T-924

With the forwarder confirmed and the scope gate confirmed, T-924's shape is:

```rust
pub async fn call_session(
    opts: &TargetOpts,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    if let Some(hub) = &opts.target {
        // Cross-host path
        let scope = opts.scope.as_deref().unwrap_or(default_scope_for(method));
        let mut client = commands::remote::connect_remote_hub(
            hub,
            opts.secret_file.as_deref(),
            opts.secret.as_deref(),
            scope,
        ).await?;
        let mut params = params;
        params.as_object_mut().unwrap().insert(
            "target".to_string(),
            json!(opts.session),
        );
        Ok(client.call(method, params).await?)
    } else {
        // Local path — existing pattern
        let reg = manager::find_session(&opts.session)?;
        Ok(client::rpc_call(reg.socket_path(), method, params).await?)
    }
}
```

The helper needs zero new hub-side code. All the complexity was already absorbed
by `forward_to_target`. T-924 is purely CLI wiring + secret-file lookup + unit
tests for the validation paths (same as the T-919 `connect_remote_hub` pattern).

## Verification commands

```bash
cargo build --workspace
cargo test -p termlink-hub --lib -- forward
test -f docs/reports/T-923-hub-forwarder-discovery.md
grep -q "forward_to_target" docs/reports/T-923-hub-forwarder-discovery.md
```

All pass as of the T-923 completion commit.

## References

- `crates/termlink-hub/src/router.rs` — `route()` dispatch + `forward_to_target`
- `crates/termlink-hub/src/server.rs:505-534` — scope gate that guards the router
- `crates/termlink-hub/src/server.rs:216-233` — `hub_method_scope` helper
- `crates/termlink-session/src/auth.rs:171-200` — `method_scope` table
- `crates/termlink-cli/src/commands/remote.rs` — `connect_remote_hub` (T-920)
- `docs/reports/T-921-cross-host-parity.md` — parent inception and option analysis
