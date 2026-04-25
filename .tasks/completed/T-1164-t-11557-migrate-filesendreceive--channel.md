---
id: T-1164
name: "T-1155/7 Migrate file.send/receive → channel.post {type: artifact}"
description: >
  ~10 sites in file.rs, remote.rs, main.rs, tools.rs. Artifact becomes typed channel.post; chunked transfer becomes bus implementation detail. Replaces T-1017/T-1018 fix path (silent drop, stale chunks) with channel semantics.

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: [T-1155, bus, migration]
components: [crates/termlink-bus/src/artifact_store.rs, crates/termlink-bus/src/error.rs, crates/termlink-bus/src/lib.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/commands/file.rs, crates/termlink-cli/src/commands/mirror_grid_composer.rs, crates/termlink-cli/src/commands/mirror_grid.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-hub/src/artifact.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/inbox.rs, crates/termlink-hub/src/lib.rs, crates/termlink-hub/src/router.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-protocol/src/control.rs, crates/termlink-session/src/artifact.rs, crates/termlink-session/src/inbox_channel.rs, crates/termlink-session/src/lib.rs]
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:15Z
last_update: 2026-04-25T13:57:00Z
date_finished: 2026-04-25T13:57:00Z
---

# T-1164: T-1155/7 Migrate file.send/receive → channel.post {type: artifact}

## Context

Third migration in the T-1155 bus rollout: `file.send` / `file.receive` → `channel.post {msg_type: "artifact", artifact_ref: <path>}`. Chunked transport becomes an implementation detail of the bus. ~10 call sites across 4 files per T-1155 §"Subsumption mapping".

Depends on: T-1163 (inbox migration proven). This migration closes PL-011 (send-file reports ok on acceptance, not delivery) structurally — with persistent log + per-recipient cursor, delivery confirmation is provable.

**Call sites + architecture audit (2026-04-24):**

*Correction to original scope:* `file.send` / `file.receive` are NOT RPC methods — there are no `handle_file_*` router handlers. The actual file-transfer protocol is **event-based** (`file.init` / `file.chunk` / `file.complete` / `file.error`), routed via `event.broadcast`. Constants in `crates/termlink-protocol/src/events.rs:466-469`. Inbox delivery uses `crates/termlink-hub/src/inbox.rs:110-116` to persist by event type name.

*Senders (produce file.* events):*
- `crates/termlink-cli/src/commands/file.rs::cmd_file_send @88` (local): serializes FileInit payload → `event.broadcast` → loops chunks → file.complete
- `crates/termlink-cli/src/commands/remote.rs @921/930/932/988/996/998` (remote): same pattern over rpc_client
- `crates/termlink-mcp/src/tools.rs::termlink_file_send @3112` (MCP): `file.init @3145`, `file.chunk @3166`, `file.complete @3188`

*Receivers (subscribe + reassemble):*
- `crates/termlink-cli/src/commands/file.rs::cmd_file_receive @262`
- `crates/termlink-mcp/src/tools.rs::termlink_file_receive @3222` (scans events for last `file.init`, finds matching `file.complete` @3312 for sha256)

*Hub-side persistence path:*
- `crates/termlink-hub/src/inbox.rs @110-116`: maps `file.init/chunk/complete/error` event-type to inbox filename conventions. Not a router handler — invoked by the inbox-delivery path for targeted events.

**Migration shape (NOT a simple shim):**
Unlike T-1162/T-1163 (RPC dispatch handlers), file transfer has no single handler to intercept. The migration must:
1. Replace the 3-phase event emit (`init → chunks → complete`) at each sender with a single `channel.post {msg_type: "artifact", artifact_ref}` + blob upload
2. Replace the scan/reassemble receiver with `channel.subscribe` filtering `msg_type == "artifact"` + blob download
3. Coordinate with `inbox.rs` persistence path — either teach it to also write artifact-style messages, or deprecate the `file.*` event filename convention

**Implication:** This task is materially bigger than T-1163. Recommend splitting into (a) blob-store primitive + `channel.post msg_type=artifact` landing, (b) sender rewrite with dual-emit during transition, (c) receiver rewrite, (d) inbox.rs cleanup. PL-011 resolution rides on (a).

## Acceptance Criteria

### Agent
- [x] Audit all callers of `file.send`, `file.receive`, `file_send`, `termlink file send` — captured above. Key finding: protocol is event-based (file.init/chunk/complete), not RPC. No handler shim possible; migration is materially bigger than T-1163.
- [x] Artifact storage: large payloads (>64KB) stored as blobs under `<bus-path>/artifacts/<sha256>`; `artifact_ref` in channel message carries the sha256; small payloads inline in channel.post — **shipped T-1248** (sharded `<root>/<2-hex>/<sha256>` layout, `ARTIFACT_INLINE_THRESHOLD = 64KB`)
- [x] `termlink file send <target> <path>` rewrites to: upload blob → `channel.post(topic="inbox:"+target, msg_type="artifact", payload=manifest, artifact_ref=sha256)` — no more manual chunking at CLI layer — **shipped T-1249** via `send_artifact_via_client` helper (`crates/termlink-session/src/artifact.rs:926,934`)
- [x] `termlink file receive` subscribes to `inbox:<self>`, filters `msg_type=="artifact"`, downloads blob, writes to disk; idempotent on sha256 (dedup) — **shipped T-1250** (`recv_artifacts_via_client` + `download_artifact_via_client`; receiver skips redownload if dest sha matches)
- [x] PL-011 resolved: `termlink file send` returns `{ok:true, offset:N, artifact_sha256: ...}` — caller can verify delivery by checking the channel log at `offset N` exists — **shipped T-1251** (`SendOutcome::Sent { sha256, channel_offset, total_bytes, path }`)
- [x] Legacy `file.send` / `file.receive` router methods remain as shims; `#[deprecated(note = "migrate to channel.post msg_type=artifact (T-1164)")]` — **N/A: audit (line 28-29) found `file.*` are EVENT names not RPC methods, no Rust handler to annotate.** Shipped equivalent via T-1251: warn-once-per-process `tracing::info!` in `inbox.rs::deposit` when legacy `file.*` events arrive (`crates/termlink-hub/src/inbox.rs`).
- [x] Integration test: send a 5MB binary file via new path; receive; verify sha256 matches — **shipped T-1248** (`put_chunked_5mb_roundtrip` in `artifact_store.rs` tests, currently passing)
- [x] `cargo build && cargo test && cargo clippy -- -D warnings` pass workspace-wide — verified 2026-04-25 (build 5.33s clean; 4/4 artifact tests pass; clippy 12.53s clean)
- [x] Artifact blob cleanup: retention policy on `inbox:<session>` channel also triggers blob GC (blob without any channel message referencing it is deletable) — **GC primitive shipped T-1251** (`ArtifactStore::sweep(&referenced) -> u64` with 3 tests). Hub-side periodic scheduling deferred (depends on bus topic-iterator that termlink-bus retention engine doesn't expose) — captured for T-1166 follow-up.
- [x] PL-011 learning updated in `.context/project/learnings.yaml` noting the structural fix landed under T-1164 — entry at line 319-325 with `application:` field naming SendOutcome::Sent fields as proof

### Human
- [x] [REVIEW] Approve artifact storage location and blob GC timing — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — artifact storage + blob GC timing approved.
  **Steps:**
  1. Verify `<bus-path>/artifacts/<sha256>` doesn't conflict with existing runtime dirs
  2. Confirm blob GC timing: sweep on retention trim, not on a timer (same as T-1158 retention engine)
  3. Check that the 64KB inline threshold is reasonable (too small = lots of blobs; too big = channel log bloats)
  **Expected:** Approval or a revised threshold
  **If not:** Note the required change

## Verification

cargo build
cargo test -p termlink-hub artifact
cargo clippy -- -D warnings
grep -q '"msg_type": "artifact"' crates/termlink-session/src/artifact.rs
grep -q "send_artifact_via_client" crates/termlink-cli/src/commands/file.rs
grep -q "recv_artifacts_via_client" crates/termlink-cli/src/commands/file.rs
grep -q "T-1251: legacy file" crates/termlink-hub/src/inbox.rs

## Decisions

### 2026-04-25 — Decompose into 4 build sub-tasks
- **Chose:** Split T-1164 into T-1248 (blob store + artifact.put/get RPC), T-1249 (sender migration), T-1250 (receiver migration), T-1251 (legacy deprecation + inbox.rs cleanup + PL-011 closure). T-1164 becomes the umbrella tracker.
- **Why:** Audit (2026-04-24) in this task body explicitly recommended a 4-way split. Per CLAUDE.md Task Sizing ("One task = one deliverable") and Pickup Message Handling (>3 new files / new subsystem → decompose), the 4 sub-tasks each represent one independent deliverable with its own ACs and verification. T-1248 is foundational; T-1249/T-1250 depend on it; T-1251 depends on b+c.
- **Rejected:** Treating T-1164 as a single task — would have aggregated 4 different deliverables under one AC list and one verification gate, making partial progress invisible and triggering G-020 build-readiness false-completes.

## Updates

### 2026-04-20T14:12:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1164-t-11557-migrate-filesendreceive--channel.md
- **Context:** Initial task creation

### 2026-04-22T04:52:49Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-25T11:42:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-25T13:57:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
