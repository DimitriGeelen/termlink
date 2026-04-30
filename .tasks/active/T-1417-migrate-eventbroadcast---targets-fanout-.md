---
id: T-1417
name: "Migrate event.broadcast --targets fanout to parallel event.emit_to (final pre-cut migration)"
description: >
  The termlink event broadcast CLI and termlink_broadcast MCP tool still call legacy event.broadcast when --targets is non-empty. After the T-1166 cut, this path errors with -32601. Replace with parallel event.emit_to per target (event.emit_to is in the keeper-set, not retired). Migration doc already plans this.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: [T-1166, T-1401, T-1403]
created: 2026-04-30T07:16:42Z
last_update: 2026-04-30T07:16:42Z
date_finished: null
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
- [ ] `crates/termlink-cli/src/commands/events.rs` no longer calls `event.broadcast` — replaced with `event.emit_to` fan-out (or `channel.post(broadcast:global)` for empty-targets case, already in place)
- [ ] `crates/termlink-mcp/src/tools.rs::termlink_broadcast` no longer calls `event.broadcast` — same fan-out pattern
- [ ] Result shape unchanged: `{topic, targeted, succeeded, failed}` (downstream consumers depend on this)
- [ ] Per-target error aggregation: if 2 of 3 targets succeed and 1 fails, response is `succeeded: 2, failed: 1` (not a hard error)
- [ ] Tests added covering: empty targets (uses channel.post), 1 target (succeeds), N targets (all succeed), N targets (some fail)
- [ ] `cargo test -p termlink-cli && cargo test -p termlink-mcp` PASS
- [ ] `cargo clippy -p termlink-cli -p termlink-mcp -- -D warnings` clean
- [ ] Migration doc updated: remove "Per-target fan-out still uses event.broadcast" note; add "T-1417 migrated --targets to parallel event.emit_to"
- [ ] Audit shows zero `event.broadcast` callers from this codebase after rebuild + restart

## Verification

cargo test -p termlink-cli --lib
cargo test -p termlink-mcp --lib
cargo clippy -p termlink-cli -p termlink-mcp -- -D warnings
# After rebuild + hub restart, attributable event.broadcast traffic from local sessions drops to 0
# (verified by api-usage --cut-ready --json on the local hub)
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
