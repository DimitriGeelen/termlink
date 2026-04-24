---
id: T-1164
name: "T-1155/7 Migrate file.send/receive → channel.post {type: artifact}"
description: >
  ~10 sites in file.rs, remote.rs, main.rs, tools.rs. Artifact becomes typed channel.post; chunked transfer becomes bus implementation detail. Replaces T-1017/T-1018 fix path (silent drop, stale chunks) with channel semantics.

status: captured
workflow_type: refactor
owner: agent
horizon: next
tags: [T-1155, bus, migration]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:15Z
last_update: 2026-04-22T04:52:49Z
date_finished: null
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
- [ ] Artifact storage: large payloads (>64KB) stored as blobs under `<bus-path>/artifacts/<sha256>`; `artifact_ref` in channel message carries the sha256; small payloads inline in channel.post
- [ ] `termlink file send <target> <path>` rewrites to: upload blob → `channel.post(topic="inbox:"+target, msg_type="artifact", payload=manifest, artifact_ref=sha256)` — no more manual chunking at CLI layer
- [ ] `termlink file receive` subscribes to `inbox:<self>`, filters `msg_type=="artifact"`, downloads blob, writes to disk; idempotent on sha256 (dedup)
- [ ] PL-011 resolved: `termlink file send` returns `{ok:true, offset:N, artifact_sha256: ...}` — caller can verify delivery by checking the channel log at `offset N` exists
- [ ] Legacy `file.send` / `file.receive` router methods remain as shims; `#[deprecated(note = "migrate to channel.post msg_type=artifact (T-1164)")]`
- [ ] Integration test: send a 5MB binary file via new path; receive; verify sha256 matches; verify the sender sees `ok:true + offset`; verify the receiver processes it exactly once even if subscribe() is called twice
- [ ] `cargo build && cargo test && cargo clippy -- -D warnings` pass workspace-wide
- [ ] Artifact blob cleanup: retention policy on `inbox:<session>` channel also triggers blob GC (blob without any channel message referencing it is deletable)
- [ ] PL-011 learning updated in `.context/project/learnings.yaml` noting the structural fix landed under T-1164

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
cargo test -p termlink-cli "file send"
cargo clippy -- -D warnings
grep -rn "file\.send\|file\.receive" crates/ | tee /tmp/T-1164-callsites.txt
grep -q "msg_type.*artifact" crates/termlink-cli/src/commands/file.rs

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

### 2026-04-20T14:12:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1164-t-11557-migrate-filesendreceive--channel.md
- **Context:** Initial task creation

### 2026-04-22T04:52:49Z — status-update [task-update-agent]
- **Change:** horizon: later → next
