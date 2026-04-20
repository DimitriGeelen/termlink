---
id: T-1173
name: "T-1162 follow-up: audit event.collect/event.poll kind-filter callers (no migration needed)"
description: >
  Audit receiver-side callers of event.collect/event.poll that filter by 'kind' (payload discriminator). Audit outcome: no such callers exist in the termlink repo — consumers filter by the RPC 'topic' param, not by a payload 'kind' field. The task premise is invalidated. Finding recorded; no code change required. The hub-side T-1162 dual-write shim already exposes every event.broadcast on channel:broadcast:global for any future consumer that wants the new surface.

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: [T-1155, bus, migration, audit]
components: [crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/router.rs]
related_tasks: [T-1162, T-1166]
created: 2026-04-20T22:28:50Z
last_update: 2026-04-20T22:40:46Z
date_finished: 2026-04-20T22:40:46Z
---

# T-1173: T-1162 follow-up — audit event.collect/event.poll kind-filter callers (no migration needed)

## Context

This task was created as Phase-2 receiver-side follow-up to T-1162 (hub-side dual-write shim). The original expectation was that agents poll `event.collect`/`event.poll` and filter the returned envelopes by a `kind` payload field, so migrating them to `channel.subscribe(broadcast:global)` + client-side `msg_type` filter would be a natural next wedge.

**Audit outcome (2026-04-21):** No `kind`-filter callers exist in the repo.

**Evidence:**
- `rg 'event\.(collect|poll)' crates/` → 31 matches across 16 files (CLI, MCP, hub, session, tests)
- `rg '"kind"\s*[:=]|kind:\s*"' crates/` → only matches are **retention policy kind** (`{"kind":"forever|days|messages"}`) in channel.rs, tools.rs, and commands/channel.rs. Zero matches are payload-discriminator filters on event polling.
- All real `event.collect`/`event.poll` consumers (`dispatch.rs:416`, `file.rs:340`, `events.rs:26`, `remote.rs:1079`, MCP tools) filter by the **RPC `topic` param**, which already routes to per-session event rings; they never inspect a payload `kind` field.

**Conclusion:** There is nothing to migrate. The T-1162 shim stands on its own:
- Producers keep calling `event.broadcast` (unchanged).
- Hub dual-writes every payload to `channel:broadcast:global` (shipped in T-1162).
- Any *new* consumer that prefers the bus surface can call `channel.subscribe("broadcast:global")` directly — no legacy code to rewrite.

If a future caller emerges (e.g., from another repo / agent implementation) that does polyfill a `kind` filter client-side, a fresh migration task should be opened at that time. Speculative migration of code that doesn't exist is anti-pattern.

## Acceptance Criteria

### Agent
- [x] Audit `event.collect`/`event.poll` call sites across the workspace (completed — see Context above)
- [x] Grep workspace for payload `kind`-field filter patterns (completed — only retention-policy `kind` found, never payload filter)
- [x] Confirm the T-1162 shim is self-sufficient — new consumers can use `channel.subscribe("broadcast:global")` directly without any producer or receiver rewrite (shipped in T-1162, verified by `BROADCAST_GLOBAL_TOPIC` constant + auto-creation at `init_bus` + unit tests in channel.rs)
- [x] Record the finding in Context + Decisions so future agents don't re-open this task speculatively
- [x] `cargo build --workspace` passes (no code change; sanity check workspace still compiles)

## Verification

cargo build --workspace
grep -q "no such callers exist" /opt/termlink/.tasks/active/T-1173-t-1162-follow-up-migrate-eventcollecteve.md
bash -c 'out=$(grep -rEn "event\[\"kind\"\]|payload\[\"kind\"\]|\.kind\s*==" /opt/termlink/crates --include="*.rs" || true); [ -z "$out" ]'

## Decisions

### 2026-04-21 — Close as premise-invalidated rather than build speculative migration

- **Chose:** Audit and close — no code changes.
- **Why:** Workspace grep shows zero `kind`-filter callers; all consumers use `topic` (RPC param) routing. Rewriting non-existent callers is speculative work; the T-1162 shim already exposes the new surface for any *future* consumer that wants it.
- **Rejected:**
  - **Rewrite dispatch.rs / file.rs `event.collect` callers anyway.** These filter by topic, not kind — rewriting them to `channel.subscribe` would change the semantic (per-session ring → shared broadcast topic) and risk dispatch coordination correctness (T-916 territory). Not a migration, a redesign. Out of scope for this wedge.
  - **Add end-to-end integration test: `event.broadcast` RPC → `channel.subscribe` RPC observes same payload.** Would require converting `channel::BUS` from `OnceLock<Bus>` to `Mutex<Option<Bus>>` or similar so tests can re-initialise per-test — a non-trivial invasive change just to add a test. The existing unit test in `channel.rs::mirror_event_broadcast_lands_envelope_in_broadcast_global` proves the mirror code path; router-level verification is covered by the human smoke-test on T-1162.

## Updates

### 2026-04-20T22:28:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1173-t-1162-follow-up-migrate-eventcollecteve.md
- **Context:** Initial task creation as T-1162 receiver-side follow-up

### 2026-04-20T22:37:32Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-04-20T22:37:32Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-21 — audit-complete
- **Action:** Grepped workspace for kind-filter event callers
- **Finding:** Zero matches. Task premise invalidated.
- **Outcome:** Rewrote scope to "audit only"; no migration code produced.

### 2026-04-20T22:40:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
