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
last_update: 2026-04-11T20:34:06Z
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
- [ ] Verified claim: `router.rs:56` transparent forwarding behaviour covers all non-hub-local methods through `forward_to_target`. Cite line ranges in the Decisions section.
- [ ] Scope enforcement gap assessed: either (a) prove via code-read that the forwarder does honor `hub.auth` scope, or (b) file a concerns.yaml entry and implement a minimal scope check before forwarding sensitive methods.
- [ ] Added integration test in `crates/termlink-hub/src/router.rs` (or `server.rs`) that binds the hub on a loopback TCP address, starts a local session, connects a client via `TransportAddr::Tcp`, calls `hub.auth`, then calls `termlink.ping` with `params.target = session_id`, and asserts the response contains the session's `display_name` — proving end-to-end cross-host forwarding.
- [ ] `cargo test -p termlink-hub --lib` passes on the new test.
- [ ] `cargo build --workspace` clean.
- [ ] docs/reports/T-923-hub-forwarder-discovery.md written with: the discovery (forwarder already exists), where it lives, the scope assessment outcome, and the path it enables for T-924.

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

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-11T20:33:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-923-hub-sessionforward-rpc--forward-a-call-t.md
- **Context:** Initial task creation

### 2026-04-11T20:34:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
