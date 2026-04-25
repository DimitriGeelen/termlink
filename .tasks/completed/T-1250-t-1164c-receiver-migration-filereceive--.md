---
id: T-1250
name: "T-1164c Receiver migration: file.receive → channel.subscribe artifact"
description: >
  Migrate all receivers to channel.subscribe + artifact download. Implements termlink-session::artifact receive helper used by CLI cmd_file_receive and MCP termlink_file_receive. Idempotent on sha256. Depends on T-1164a.

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: [T-1164, T-1155, bus, artifact]
components: [crates/termlink-bus/src/artifact_store.rs, crates/termlink-bus/src/error.rs, crates/termlink-bus/src/lib.rs, crates/termlink-cli/src/commands/file.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-hub/src/artifact.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/lib.rs, crates/termlink-hub/src/router.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-protocol/src/control.rs, crates/termlink-session/src/artifact.rs]
related_tasks: [T-1164, T-1164a, T-1155]
created: 2026-04-25T11:43:51Z
last_update: 2026-04-25T13:25:48Z
date_finished: 2026-04-25T13:25:48Z
---

# T-1250: T-1164c Receiver migration: file.receive → channel.subscribe artifact

## Context

Sub-task of T-1164. Adds `recv_artifacts_via_client` to `termlink-session::artifact` (alongside the T-1249 sender helper) plus `download_artifact_via_client` for chunked blob retrieval. Migrates `cmd_file_receive` (CLI) and `termlink_file_receive` (MCP) to consume `channel.subscribe(inbox:<self>)` filtered on `msg_type=artifact`, then download via `artifact.get` if the envelope carries `artifact_ref`, verify sha256, and write to disk. Reuses `inbox_channel::FallbackCtx` for capability gating + warn-once, mirroring the T-1249 sender pattern.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-session/src/artifact.rs` gains `recv_artifacts_via_client(client, host_port, target_self, since_offset, cache, ctx) -> io::Result<RecvOutcome>` that does `channel.subscribe(inbox:<target_self>, cursor=since_offset)`, filters `msg_type == "artifact"`, parses each manifest, and returns `Vec<RecvArtifact>` + `next_cursor`. Plus `download_artifact_via_client(client, sha256) -> io::Result<Vec<u8>>` that drives chunked `artifact.get` until eof and verifies the sha matches the requested key.
- [x] Capability gate: probes `channel.subscribe`. On absent, returns `RecvOutcome::LegacyOnly`. (Inline-only artifacts don't need `artifact.get`; chunked artifacts also need `artifact.get`, checked when downloading.)
- [x] CLI `cmd_file_receive` (`crates/termlink-cli/src/commands/file.rs`) tries the new path first when local hub is up. For each artifact envelope: if inline, use the payload bytes; else download via `artifact.get`. Verify sha256 against the artifact_ref/manifest. Write to `<output_dir>/<filename>`. Idempotent on sha256: skip if file already exists with matching hash. On `LegacyOnly`, falls through to existing event-stream reassembly.
- [x] MCP `termlink_file_receive` (`crates/termlink-mcp/src/tools.rs`) uses the same helpers; legacy fallback preserved.
- [x] Unit tests in `termlink-session::artifact`: receive-side fake hub serves channel.subscribe with one inline + one chunked artifact; helper parses both correctly; download_artifact_via_client retrieves chunked bytes; LegacyOnly when channel.subscribe absent from caps.
- [x] `cargo build && cargo test --workspace --lib && cargo clippy --workspace --tests -- -D warnings` all pass. Zero regressions.

## Limitations / Known issues

- **Inline path filename loss.** Sender's inline route (T-1249, payload ≤ 64KB) ships raw bytes as the channel payload — no manifest. Receiver synthesizes `received-<sha256[..16]>.bin` since the original filename is unrecoverable. Acceptable for migration; future improvement: wrap inline payloads in `{manifest, payload_b64}` JSON.
- **Migration window 2× wait.** When the local hub advertises `channel.subscribe` but a sender still uses legacy `file.*` events, the new-path loop consumes the full `--timeout` before falling through to legacy reassembly. Worst case wait = 2 × timeout. Acceptable for migration; documented in `try_recv_via_artifact` rustdoc.

## Verification

cargo build
cargo test -p termlink-session artifact
cargo clippy --workspace --tests -- -D warnings
grep -q "recv_artifacts_via_client" /opt/termlink/crates/termlink-session/src/artifact.rs
grep -q "download_artifact_via_client" /opt/termlink/crates/termlink-session/src/artifact.rs

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

### 2026-04-25T11:43:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1250-t-1164c-receiver-migration-filereceive--.md
- **Context:** Initial task creation

### 2026-04-25T12:09:54Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-25T13:25:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
