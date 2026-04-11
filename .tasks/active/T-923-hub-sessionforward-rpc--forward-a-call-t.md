---
id: T-923
name: "Hub session.forward RPC — forward a call to a local session socket"
description: >
  T-921 Spike 3 picked routing option γ (hub-as-forwarder): add one new hub JSON-RPC method 'session.forward' that accepts { target, method, params } and translates it to client::rpc_call against the target session's local unix socket on the hub's host. Scope check requires 'interact' or higher. Tests: round-trip termlink.ping through the forwarder. Prerequisite for T-924 (the CLI --target helper) and T-925..T-935 (per-command rollout).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-11T20:33:52Z
last_update: 2026-04-11T20:51:54Z
date_finished: null
---

# T-923: Hub session.forward RPC — forward a call to a local session socket

## Context

T-921 Spike 3 picked routing option γ (hub-as-forwarder) and planned this task as "add a new `session.forward` hub RPC method". On investigation the mechanism is already present: `crates/termlink-hub/src/router.rs:56` has `_ => forward_to_target(req, id).await,` — **any** unknown RPC method falls through to `forward_to_target` (defined at `router.rs:1171`), which reads `params.target`, resolves it via `manager::find_session` (local FS) or `remote_store` (remote entries), dials that address, and forwards the request transparently. Existing test `forward_to_target_session` at `router.rs:1371` exercises the local-forwarding path.

So T-923 does **not** need to add a new wrapper method. What it needs to verify and close:

1. The forwarder flow works end-to-end when the hub is bound on TCP (T-920 shipped `run_with_tcp`) — i.e. a remote client can `hub.auth` → `termlink.ping` with `params.target = S-xxx` → response returns. The existing test exercises only the local-FS path; no test uses the TCP hub.
2. The forwarder honors the token scope obtained via `hub.auth`. A worry: `forward_to_target` at 1171-1230 does not re-check scope before dialing the session. If true, this is a security gap (any client with `observe` scope can call `command.inject` through the forwarder). This must either be confirmed safe or filed as a gap + fixed.
3. The discovery is documented so future sessions (and T-924) do not re-invent `session.forward`.

Linked: T-921 (inception, closed GO), T-924 (CLI TargetOpts helper — unblocked by this task).

## Acceptance Criteria

### Agent
- [x] Verified claim: `router.rs:56` transparent forwarding behaviour covers all non-hub-local methods through `forward_to_target`. Cite line ranges in the Decisions section.
- [x] Scope enforcement gap assessed: either (a) prove via code-read that the forwarder does honor `hub.auth` scope, or (b) file a concerns.yaml entry and implement a minimal scope check before forwarding sensitive methods.
- [x] Added integration test in `crates/termlink-hub/src/router.rs` (or `server.rs`) that binds the hub on a loopback TCP address, starts a local session, connects a client via `TransportAddr::Tcp`, calls `hub.auth`, then calls `termlink.ping` with `params.target = session_id`, and asserts the response contains the session's `display_name` — proving end-to-end cross-host forwarding.
- [x] `cargo test -p termlink-hub --lib` passes on the new test.
- [x] `cargo build --workspace` clean.
- [x] docs/reports/T-923-hub-forwarder-discovery.md written with: the discovery (forwarder already exists), where it lives, the scope assessment outcome, and the path it enables for T-924.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo build --workspace
cargo test -p termlink-hub --lib -- forward
test -f docs/reports/T-923-hub-forwarder-discovery.md
grep -q "forward_to_target" docs/reports/T-923-hub-forwarder-discovery.md

## Decisions

### 2026-04-11 — No new hub RPC method; close as discovery + test + doc

- **Chose:** Skip implementing a new `session.forward` hub RPC method. The hub
  router already transparently forwards any non-hub-local method to the named
  session through `forward_to_target`. Close T-923 with (a) an end-to-end
  integration test that drives TCP hub + `hub.auth` + forwarding + local session
  in one test process and (b) a discovery report so T-924 and future sessions
  do not re-invent the mechanism.
- **Why:** Building a wrapper would have been duplicate code on top of the
  existing fallthrough dispatch. The forwarder has existed since T-182/T-920 —
  the T-921 inception planned T-923 without noticing it. Code-read evidence:
  - `crates/termlink-hub/src/router.rs` — `route()` match: hub-local arms
    (discover, broadcast, collect, register_remote, orchestrator.route, …) +
    fallthrough `_ => forward_to_target(req, id).await`.
  - `crates/termlink-hub/src/router.rs:1171-1230` — `forward_to_target` reads
    `params.target`, resolves via `manager::find_session` → fallback to
    `remote_store`, dials the address via `client::Client::connect_addr`, and
    proxies the call+response transparently.
- **Rejected:** Adding a wrapper method `session.forward` that takes
  `{target, method, params}`. It would have been a strictly-weaker alias for
  the transparent fallthrough and a second surface to keep in sync with
  `auth::method_scope` / `hub_method_scope`.

### 2026-04-11 — Scope enforcement is in the server accept loop, not the forwarder

- **Chose:** Close the scope-gap AC as "verified safe via code-read + regression
  test". Do NOT add per-method scope checks inside `forward_to_target`.
- **Why:** `crates/termlink-hub/src/server.rs:505-534` enforces
  `scope.satisfies(hub_method_scope(&req.method))` BEFORE calling
  `router::route(&req)`. `hub_method_scope` (server.rs:216-233) falls through
  to `auth::method_scope(method)` for any method the router will forward, so
  forwarded calls are gated by the same scope table a local session would
  enforce on its own unix socket. The new
  `tcp_forward_rejected_when_scope_insufficient` test authenticates with
  Observe scope and attempts `kv.set` (requires Interact); the hub returns
  AUTH_DENIED (-32010) without reaching `forward_to_target`. This is the
  regression guard — if someone moves scope enforcement after the router
  dispatch, the test fails.
- **Rejected:** Duplicating scope checks inside `forward_to_target`. Would have
  been redundant, would have made the scope table live in two places, and
  would have masked the real invariant (scope gate is the accept loop's
  responsibility).

## Updates

### 2026-04-11T20:33:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-923-hub-sessionforward-rpc--forward-a-call-t.md
- **Context:** Initial task creation

### 2026-04-11T20:34:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
