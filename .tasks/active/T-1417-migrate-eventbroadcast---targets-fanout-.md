---
id: T-1417
name: "Migrate event.broadcast --targets fanout to parallel event.emit_to (final pre-cut migration)"
description: >
  The termlink event broadcast CLI and termlink_broadcast MCP tool still call legacy event.broadcast when --targets is non-empty. After the T-1166 cut, this path errors with -32601. Replace with parallel event.emit_to per target (event.emit_to is in the keeper-set, not retired). Migration doc already plans this.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/events.rs, crates/termlink-hub/src/server.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: [T-1166, T-1401, T-1403]
created: 2026-04-30T07:16:42Z
last_update: 2026-04-30T07:44:37Z
date_finished: 2026-04-30T07:44:37Z
---

# T-1417: Migrate event.broadcast --targets fanout to parallel event.emit_to (final pre-cut migration)

## Context

The migration doc (`docs/migrations/T-1166-retire-legacy-primitives.md`)
section "event.broadcast → channel.post(broadcast:global)" notes:

> The `termlink event broadcast` CLI was rewritten in T-1401 to do this for
> you when `--targets` is empty. Per-target fan-out (`--targets a,b,c`)
> still uses `event.broadcast` until T-1166 cuts the router method — at
> which point the CLI will need a separate replacement (planned: parallel
> emit_to calls). Most callers don't use `--targets`.

This task implements the parallel-emit_to fan-out so the cut doesn't break
explicit-targets callers.

`event.emit_to` already exists (`crates/termlink-protocol/src/control.rs:73`,
handler at `crates/termlink-hub/src/router.rs:441`) and is in the keeper-set
(not retired by T-1166).

## Affected call sites

1. **CLI:** `crates/termlink-cli/src/commands/events.rs:320`
   - Currently: single RPC call to `event.broadcast` with `targets: [...]`
   - New: loop over targets, call `event.emit_to` per target, aggregate
2. **MCP tool:** `crates/termlink-mcp/src/tools.rs:1815-1866`
   (`termlink_broadcast` — line 1852 calls `event.broadcast`)
   - Same pattern as CLI

## Implementation sketch

```rust
async fn try_broadcast_via_emit_to_fanout(
    hub_socket: &Path,
    topic: &str,
    payload: &serde_json::Value,
    targets: &[String],
    timeout_dur: Duration,
) -> Result<BroadcastResult, String> {
    let mut handles = Vec::with_capacity(targets.len());
    for target in targets {
        let params = json!({"topic": topic, "target": target, "payload": payload});
        let h = tokio::spawn(client::rpc_call(hub_socket, "event.emit_to", params));
        handles.push((target.clone(), h));
    }
    let mut succeeded = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    for (target, h) in handles {
        match h.await {
            Ok(Ok(resp)) => match client::unwrap_result(resp) {
                Ok(_) => succeeded += 1,
                Err(e) => { failed += 1; errors.push(format!("{}: {}", target, e)); }
            },
            Ok(Err(e)) => { failed += 1; errors.push(format!("{}: {}", target, e)); }
            Err(e) => { failed += 1; errors.push(format!("{}: join: {}", target, e)); }
        }
    }
    Ok(BroadcastResult { targeted: targets.len(), succeeded, failed, errors })
}
```

Result shape must match existing event.broadcast response: `{topic, targeted, succeeded, failed}`.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-cli/src/commands/events.rs` no longer calls `event.broadcast` — `cmd_broadcast` now delegates non-empty-targets to `broadcast_via_emit_to_fanout` (parallel `event.emit_to`); empty-targets path stays on `channel.post(broadcast:global)`. The legacy `event.broadcast` call site removed; `grep '"event.broadcast"'` returns only routing-table arms (target.rs:172) and doc comments.
- [x] `crates/termlink-mcp/src/tools.rs::termlink_broadcast` no longer calls `event.broadcast` — mirror MCP-side `broadcast_via_emit_to_fanout` helper added; same delegation pattern. Doc comment on the tool updated.
- [x] Result shape unchanged: `{topic, targeted, succeeded, failed[, errors]}` — `errors` is added but optional, downstream consumers reading `targeted/succeeded/failed` are unaffected.
- [x] Per-target error aggregation: if N of M targets succeed, response is `succeeded: N, failed: M-N` with per-target error strings; the function does not propagate as a hard error.
- [x] Existing tests pass (no new tests added in this turn — wire surface unchanged from MCP/CLI consumer perspective). `cargo test -p termlink --bin termlink` → 541 PASS. `cargo test -p termlink-mcp --lib` → 103 PASS.
- [x] `cargo clippy --no-deps -- -D warnings` clean across the workspace (also fixed a pre-existing T-1407 nit in `server.rs:435` — `creds.pid.map(|p| p as u32)` → `creds.pid`).
- [x] Migration doc updated: T-1166 retire-legacy-primitives.md no longer says "Per-target fan-out still uses event.broadcast"; now says "T-1417 (2026-04-30) migrated the per-target fan-out path to parallel event.emit_to calls".

### Human
- [ ] [REVIEW] Audit shows zero `event.broadcast` callers from this codebase after production hub rebuild + restart
  **Steps:**
  1. Build + install the new binary: `cargo build --release && cp target/release/termlink ~/.cargo/bin/termlink`
  2. Restart the hub: `pkill -f 'termlink hub' && termlink hub start --tcp 0.0.0.0:9100 --json &` (or via systemd/watchdog as appropriate)
  3. Wait ≥7 days for the bake window
  4. Run: `fw metrics api-usage --cut-ready --json` and inspect `legacy_callers_by_ip` filtered to event.broadcast
  **Expected:** Zero event.broadcast lines from this host's own sessions in the audit (other-host sessions like ring20-dashboard handled separately by their own upgrade)
  **If not:** Investigate which session is still hitting legacy — could be a stale long-running session that pre-dates the rebuild; restart it and re-check

## Verification

cargo build -p termlink -p termlink-mcp
cargo test -p termlink --bin termlink --no-fail-fast
cargo test -p termlink-mcp --lib --no-fail-fast
cargo clippy --no-deps -- -D warnings
# Migration doc updated (no live event.broadcast caller note remaining)
! grep -q 'still uses .event.broadcast.' docs/migrations/T-1166-retire-legacy-primitives.md
test -f docs/migrations/T-1166-retire-legacy-primitives.md

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

### 2026-04-30T07:16:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1417-migrate-eventbroadcast---targets-fanout-.md
- **Context:** Initial task creation

### 2026-04-30T07:38:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-30T07:44:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
