---
id: T-1225
name: "T-1220a: inbox_channel::list_with_fallback helper (T-1220 wedge a)"
description: >
  termlink-session helper that wraps capabilities probe + channel.subscribe(topic=inbox:<target>) + legacy inbox.list fallback + dedup-merge. Foundation for T-1220b/c/d migrations. ~100 LOC + tests. Per T-1220 GO inception: in-memory cursor (Q1 D), per-session-per-target cap cache (Q2 B), warn-once + flag-legacy fallback (Q3 B+C), dual-read transition (Q5 A).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-1155, bus, migration, T-1220, wedge-a]
components: [crates/termlink-session/src/inbox_channel.rs, crates/termlink-session/src/lib.rs]
related_tasks: [T-1220, T-1215, T-1163]
created: 2026-04-25T07:00:04Z
last_update: 2026-04-25T07:06:40Z
date_finished: 2026-04-25T07:06:40Z
---

# T-1225: T-1220a: inbox_channel::list_with_fallback helper (T-1220 wedge a)

## Context

Foundation wedge for T-1220 (GO). Provides a single async helper that
T-1220b/c/d (CLI local, CLI remote, MCP) call instead of `inbox.list` RPC.
The helper owns the capabilities-gated dispatch: read `inbox:<target>`
channel topic when peer supports `channel.subscribe`, fall back to legacy
`inbox.list` otherwise.

Channel mirror format (T-1163, `crates/termlink-hub/src/channel.rs:108`):
each `inbox::deposit` posts to `inbox:<target>` with
`msg_type=<file-event-topic>` and `payload={from, payload}`. The helper
reassembles `PendingTransfer`-shaped summaries from the message stream
(group by `transfer_id` extracted from each event payload, count chunks,
detect `file.complete`).

In-process cursor + warn-once tracker live on the helper struct
(per Q1 D + Q3 B/C of the inception). No on-disk persistence.

## Acceptance Criteria

### Agent
- [x] New module `crates/termlink-session/src/inbox_channel.rs` exposing
      `pub async fn list_with_fallback(addr: &TransportAddr, target: &str,
      cache: &HubCapabilitiesCache, ctx: &mut FallbackCtx) ->
      io::Result<Vec<InboxEntry>>`. `InboxEntry` mirrors the existing
      `inbox::list_pending` shape (transfer_id/filename/from/size/
      chunks_received/total_chunks/complete) so callers swap the call
      without touching downstream rendering.
- [x] `FallbackCtx` carries: in-memory per-target cursor (HashMap<String,
      u64>), warn-once dedup set (HashSet<(String, String)> keyed by
      (host_port, "channel.subscribe"|"inbox.list")), and a "peer flagged
      legacy-only" set. `pub fn new()` provides a fresh instance.
- [x] Dispatch logic:
        1. Probe capabilities via `hub_capabilities::probe(addr, cache)`.
        2. If `channel.subscribe` ∈ methods AND peer not flagged legacy:
           call `channel.subscribe(topic="inbox:<target>", cursor=<saved>)`,
           reassemble, advance cursor.
        3. Otherwise: log a warn-once line (per host_port + method) and
           call legacy `inbox.list`. On `method-not-found` from a
           supposedly-supported peer: flag legacy-only in ctx + warn-once.
- [x] Reassembly helper `fn fold_envelopes(messages: &[Value]) ->
      Vec<InboxEntry>`: walks msg_type ∈ {file.init, file.chunk,
      file.complete, file.error}, groups by transfer_id, counts chunks,
      sets `complete=true` on file.complete. file.error drops the entry.
- [x] Unit test `fold_envelopes_assembles_pending_transfer`: feeds a
      synthetic 3-message stream (init+chunk+complete) for one transfer
      → asserts one InboxEntry with chunks_received=1, total_chunks=1,
      complete=true.
- [x] Unit test `fold_envelopes_groups_by_transfer_id`: 2 interleaved
      transfers → 2 entries, correct chunk counts.
- [x] Unit test `fallback_ctx_warn_once_dedupes`: same (host, method)
      pair logged twice → second insert returns false.
- [x] Unit test `fold_envelopes_drops_errored_transfer`: init + file.error
      → empty result.
- [x] `cargo build -p termlink-session` clean (0 warnings).
- [x] `cargo test -p termlink-session inbox_channel` (≥4 passed) — all PASS.
- [x] No clippy regressions: `cargo clippy -p termlink-session -- -D warnings` clean.

## Verification

cargo build -p termlink-session 2>&1 | tail -5
cargo test -p termlink-session inbox_channel 2>&1 | tail -10
cargo clippy -p termlink-session -- -D warnings 2>&1 | tail -5
grep -q "pub async fn list_with_fallback" crates/termlink-session/src/inbox_channel.rs
grep -q "fold_envelopes" crates/termlink-session/src/inbox_channel.rs

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

### 2026-04-25T07:00:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1225-t-1220a-inboxchannellistwithfallback-hel.md
- **Context:** Initial task creation

### 2026-04-25T07:00:38Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-25T07:06:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
