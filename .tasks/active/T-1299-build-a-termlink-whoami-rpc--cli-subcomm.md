---
id: T-1299
name: "Build A: termlink whoami RPC + CLI subcommand"
description: >
  Per T-1297 GO: read-only RPC that returns the calling session's identity card (id, display_name, roles, tags, cwd, pid, hub_address). Disambiguator chain: TERMLINK_SESSION_ID env var (primary, set by termlink register) → source-PID tree-walk fallback → ambiguous-result hint with candidates list. Pure exposure of existing session registry — no new data model. Estimate: ½ dev-day. Reversible: additive RPC. Forward-compat: older binaries return Method-not-found cleanly. Evidence: docs/reports/T-1297-termlink-agent-routing-discipline.md § Spike 2.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [termlink, routing, whoami, T-1297-child, hub-rpc]
components: []
related_tasks: [T-1297]
created: 2026-04-26T21:19:36Z
last_update: 2026-04-26T21:20:09Z
date_finished: null
---

# T-1299: Build A: termlink whoami RPC + CLI subcommand

## Context

Per T-1297 GO. Adds a hub RPC that lets a caller ask "who am I on this hub?" — closing the originator-confusion gap that produced the 5 misroutes documented in T-1297 Spike 1. Pure exposure of the existing session registry; no new data model.

Spike 2 evidence: `docs/reports/T-1297-termlink-agent-routing-discipline.md` § Spike 2 — 71% of current sessions share their cwd with another, so cwd-only lookup is insufficient. Disambiguator chain:
1. `TERMLINK_SESSION_ID` env var (primary; injected by `termlink register` into the shell it spawns)
2. Source-PID tree walk (fallback for processes that lost the env var)
3. Ambiguous-result hint with candidates list (tertiary)

## Acceptance Criteria

### Agent
- [x] `termlink-protocol`: `session.whoami` method constant added (`crates/termlink-protocol/src/control.rs`)
- [x] `termlink-hub`: `handle_whoami` resolves by `session_id` or `display_name` hint; returns identity card OR `{ ok: true, ambiguous: true, candidates: [...] }` when no hint OR `{ ok: false, found: false, hint: "..." }` when hint is unknown. Wired into `route()` and `hub_method_scope()` (Observe scope) and the `hub.capabilities` listing
- [x] `termlink-cli`: `termlink whoami` subcommand — reads `$TERMLINK_SESSION_ID` env var when no `--session`/`--name` flag is passed; prints human-readable card (default) or JSON (`--json`); ambiguous-list path prints all candidates with the hint to set the env var
- [x] Unit tests cover: id-hint match, name-hint match, no-hint candidate list, unknown-hint not-found-with-hint (4 new tests in `crates/termlink-hub/src/router.rs::tests`)
- [x] Workspace `cargo clippy --all-targets -- -D warnings` clean
- [x] All existing unit + integration tests still pass (termlink-protocol 100, termlink-hub 235, termlink CLI 225 unit + 172 integration, all green)
- [ ] **Deferred to a follow-up build (T-13xx):** (a) `termlink register` env-var injection so spawned shells inherit `TERMLINK_SESSION_ID` automatically — currently the operator sets it manually by copying from `termlink list --json`; (b) source-PID tree-walk fallback when no env var is set (requires threading `peer_pid` through hub's connection-handler → router::route plumbing); (c) cross-host forward-compat hint when calling `session.whoami` against a hub that pre-dates this build (current CLI uses local file lookup, not RPC, so this only matters for `termlink remote call ... session.whoami` which is out of MVP scope).

### Human
- [ ] [RUBBER-STAMP] Smoke test on the local-test hub from different sessions
  **Steps:**
  1. From inside one of the registered sessions on .107: `termlink whoami`
  2. From outside any session (fresh shell): `termlink whoami`
  3. From a session with multiple cwd siblings (e.g. one of the three /opt/termlink sessions): `termlink whoami` — should pick correct one via env var
  **Expected:** Each call returns the right identity card or a clear "ambiguous, set TERMLINK_SESSION_ID" hint with candidates listed.
  **If not:** Capture the failing payload and the actual session list (`termlink list`) — mismatch indicates either env-injection regression or PID-walk bug.

## Verification

# All commands run from the repo root.
cargo build --workspace 2>&1 | tail -5 | grep -qE "Finished|Compiling"
cargo test -p termlink-protocol --lib --quiet 2>&1 | tail -3 | grep -qE "test result: ok"
cargo test -p termlink-hub --lib whoami --quiet 2>&1 | tail -3 | grep -qE "test result: ok"
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -3 | grep -qE "Finished|Checking"
target/debug/termlink whoami --json 2>&1 | grep -qE "\"ok\":\\s*true"

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

### 2026-04-26T21:19:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1299-build-a-termlink-whoami-rpc--cli-subcomm.md
- **Context:** Initial task creation

### 2026-04-26T21:20:09Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-26T22:30Z — MVP shipped [agent]
- **Protocol:** `SESSION_WHOAMI = "session.whoami"` constant added (Tier-A, opaque payload).
- **Hub handler:** `handle_whoami` in `crates/termlink-hub/src/router.rs` — resolves by `session_id`/`display_name` hint via existing `manager::find_session()`; returns identity card or candidate list. Observe-scope. Listed in `hub.capabilities`.
- **CLI:** `termlink whoami` reads `$TERMLINK_SESSION_ID` when no `--session`/`--name` is passed. Smoke-tested all four paths on local hub:
  - `--session tl-ismotg7j` → resolved to framework-agent ✓
  - `--name email-archive --json` → JSON identity card ✓
  - `--session bogus` → exit 1 with helpful hint ✓
  - `TERMLINK_SESSION_ID=tl-pljpkait termlink whoami` → email-archive ✓
- **Tests:** 4 new unit tests in `crates/termlink-hub/src/router.rs::tests` — all green. Total hub: 235 passing. termlink CLI: 225 unit + 172 integration. termlink-protocol: 100. Workspace clippy clean.
- **Descoped (3 deferred items, captured as final unchecked AC):** (a) `termlink register` env-var injection — currently the operator sets `$TERMLINK_SESSION_ID` manually by reading the id from `termlink list --json`; (b) source-PID tree-walk fallback — requires plumbing `peer_pid` from `PeerCredentials` through the JSON-RPC dispatch (invasive, single-purpose, deferred); (c) cross-host forward-compat hint — current CLI uses local file lookup, so the RPC method-not-found path only matters for `termlink remote call ... session.whoami`. None of these block the routing-discipline value: agents can call `whoami` today by reading their session id from `termlink list` once.
- **Operator AC:** smoke test from a few sessions + verify the candidate list is helpful in the cwd-collision case.
