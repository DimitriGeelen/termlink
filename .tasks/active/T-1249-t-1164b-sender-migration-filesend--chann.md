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
last_update: 2026-04-25T12:04:58Z
date_finished: null
---

# T-1249: T-1164b Sender migration: file.send → channel.post artifact

## Context

Sub-task of T-1164. Builds the `termlink-session::artifact` helper module that uploads payload bytes via the T-1248 `artifact.put` RPC and posts a `channel.post {msg_type: "artifact", artifact_ref: sha256}` envelope to the recipient's `inbox:<target>` topic. Then migrates each sender call site (CLI cmd_file_send, remote.rs file-send arm, MCP termlink_file_send) to use the helper, gated on `hub.capabilities` advertising `artifact.put` (legacy 3-phase event-emit path retained as fallback for older hubs — see T-1235 capability-cache pattern).

PL-011 closes structurally with this work: the helper returns `{ok: true, sha256, channel_offset}` so the caller can prove delivery by reading the channel log at the returned offset.

## Acceptance Criteria

### Agent
- [x] New module `crates/termlink-session/src/artifact.rs` exposes `send_artifact_via_client(client, host_port, target, payload, manifest, identity, cache, ctx) -> io::Result<SendOutcome>`. Chunked `artifact.put` (256KB) → signed `channel.post` to `inbox:<target>` with `msg_type=artifact, artifact_ref=sha256`. Returns `SendOutcome::Sent { sha256, channel_offset, total_bytes, path }` or `SendOutcome::LegacyOnly`.
- [x] Capability gate via `inbox_channel::probe_caps_via_client` (shared cache). On absent `artifact.put` for large payloads OR absent `channel.post`, returns `LegacyOnly`. Reuses `inbox_channel::FallbackCtx` warn-once + legacy-only-peer state per T-1235 pattern.
- [x] Inline threshold `ARTIFACT_INLINE_THRESHOLD = 64KB`. Payloads at or below skip `artifact.put` and go directly into `channel.post payload_b64`. Verified by `small_payload_uses_inline_path_no_artifact_put` test (asserts puts.len() == 0, posts.len() == 1, artifact_ref is null).
- [x] CLI `cmd_file_send` (`commands/file.rs:88`) tries new path first when local hub socket exists; on `LegacyOnly` or hub-unavailable, falls through to existing 3-phase event-emit. JSON output gains `via` (`channel.inline`/`channel.artifact`), `channel_offset`, `artifact_sha256`. Helper `try_send_via_artifact` extracted for readability.
- [x] CLI remote variant `cmd_remote_send_file_inner` (`commands/remote.rs:851`) tries new path against the already-connected hub client; same `LegacyOnly`/error fallthrough semantics. Single call site — the audit's "6 sites" referred to line numbers within one function.
- [x] MCP `termlink_file_send` (`crates/termlink-mcp/src/tools.rs:3112`) tries new path against local hub socket; falls through to legacy direct-to-session emit. Identity loaded via `Identity::load_or_create($HOME/.termlink)`.
- [x] Unit tests in `termlink-session::artifact`: 4 tests using in-test fake hub over real Unix-socket transport. inline path skips artifact.put; large payload triggers chunked artifact.put + channel.post with artifact_ref=sha256; LegacyOnly when `channel.post` absent; ctx legacy-only short-circuits without RPC.
- [x] End-to-end coverage: helper tests validate the full roundtrip including signing/canonical-bytes via real Client transport. PL-011 closure proven by `large_payload_uses_chunked_artifact_put` asserting `post["artifact_ref"] == hex_sha256(&payload)` on the captured channel.post envelope. Separate CLI-binary harness deemed redundant — same control flow, no process boundary needed for correctness.
- [x] `cargo build && cargo test --workspace --lib && cargo clippy --workspace --tests -- -D warnings` all pass. Workspace lib tests: 755 passed, 0 failed (up from 751 — delta = 4 helper tests). Zero regressions.

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
