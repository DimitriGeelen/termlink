---
id: T-1249
name: "T-1164b Sender migration: file.send → channel.post artifact"
description: >
  Migrate all senders to artifact-store + channel.post. Implements termlink-session::artifact send helper used by CLI cmd_file_send, remote.rs (6 call sites), and MCP termlink_file_send. Depends on T-1164a.

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: [T-1164, T-1155, bus, artifact]
components: []
related_tasks: [T-1164, T-1164a, T-1155]
created: 2026-04-25T11:43:48Z
last_update: 2026-04-25T11:55:11Z
date_finished: null
---

# T-1249: T-1164b Sender migration: file.send → channel.post artifact

## Context

Sub-task of T-1164. Builds the `termlink-session::artifact` helper module that uploads payload bytes via the T-1248 `artifact.put` RPC and posts a `channel.post {msg_type: "artifact", artifact_ref: sha256}` envelope to the recipient's `inbox:<target>` topic. Then migrates each sender call site (CLI cmd_file_send, remote.rs file-send arm, MCP termlink_file_send) to use the helper, gated on `hub.capabilities` advertising `artifact.put` (legacy 3-phase event-emit path retained as fallback for older hubs — see T-1235 capability-cache pattern).

PL-011 closes structurally with this work: the helper returns `{ok: true, sha256, channel_offset}` so the caller can prove delivery by reading the channel log at the returned offset.

## Acceptance Criteria

### Agent
- [ ] New module `crates/termlink-session/src/artifact.rs` exposes `pub async fn send_artifact(client, hub_addr, target, payload, manifest, cache, ctx) -> Result<SendOutcome>`. Internally: chunked `artifact.put` (256KB chunks) → `channel.post` to topic `inbox:<target>` with `msg_type=artifact, artifact_ref=sha256`. Returns `{sha256, channel_offset, total_bytes}`.
- [ ] Capability gate: probes `HubCapabilitiesCache::shared_cache()` for `artifact.put`. On unsupported hub, returns `LegacyOnly` outcome — caller must fall back to the existing event-emit path. (Same `FallbackCtx` warn-once pattern as T-1235.)
- [ ] Helper handles small (≤64KB) payloads via inline `channel.post payload_b64` (skipping artifact.put entirely) per T-1164 design.
- [ ] CLI sender `cmd_file_send` (`crates/termlink-cli/src/commands/file.rs`) uses the helper. On `LegacyOnly`, falls through to existing 3-phase event-emit. JSON output gains `artifact_sha256` + `channel_offset` fields when the new path is used.
- [ ] CLI remote variants (`crates/termlink-cli/src/commands/remote.rs` ~6 call sites for `termlink_file_send`-like paths) use the helper. Same fallback semantics.
- [ ] MCP `termlink_file_send` tool (`crates/termlink-mcp/src/tools.rs`) uses the helper. JSON envelope preserves `{ok, hub, result}` shape; `result` includes `artifact_sha256` + `channel_offset` on the new path.
- [ ] Unit tests in `termlink-session::artifact`: small payload inline path; large payload chunked path; capability-cache miss → LegacyOnly; integration against a fake hub that answers `artifact.put` + `channel.post`.
- [ ] CLI integration test: `termlink file send` against a hub that advertises `artifact.put` succeeds; verify the channel log contains the artifact envelope with the right sha256.
- [ ] `cargo build && cargo test --workspace --lib && cargo clippy -- -D warnings` pass workspace-wide. No regressions in pre-existing tests.

## Verification

cargo build
cargo test -p termlink-session artifact
cargo test -p termlink artifact
cargo clippy --workspace -- -D warnings
test -f /opt/termlink/crates/termlink-session/src/artifact.rs
grep -q "send_artifact" /opt/termlink/crates/termlink-cli/src/commands/file.rs

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

### 2026-04-25T11:43:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1249-t-1164b-sender-migration-filesend--chann.md
- **Context:** Initial task creation

### 2026-04-25T11:55:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
