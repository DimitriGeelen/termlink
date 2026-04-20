---
id: T-1164
name: "T-1155/7 Migrate file.send/receive → channel.post {type: artifact}"
description: >
  ~10 sites in file.rs, remote.rs, main.rs, tools.rs. Artifact becomes typed channel.post; chunked transfer becomes bus implementation detail. Replaces T-1017/T-1018 fix path (silent drop, stale chunks) with channel semantics.

status: captured
workflow_type: refactor
owner: agent
horizon: later
tags: [T-1155, bus, migration]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:15Z
last_update: 2026-04-20T14:12:15Z
date_finished: null
---

# T-1164: T-1155/7 Migrate file.send/receive → channel.post {type: artifact}

## Context

Third migration in the T-1155 bus rollout: `file.send` / `file.receive` → `channel.post {msg_type: "artifact", artifact_ref: <path>}`. Chunked transport becomes an implementation detail of the bus. ~10 call sites across 4 files per T-1155 §"Subsumption mapping".

Depends on: T-1163 (inbox migration proven). This migration closes PL-011 (send-file reports ok on acceptance, not delivery) structurally — with persistent log + per-recipient cursor, delivery confirmation is provable.

## Acceptance Criteria

### Agent
- [ ] Audit all callers of `file.send`, `file.receive`, `file_send`, `termlink file send` — grep produces exhaustive call-site list; add to task file
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
- [ ] [REVIEW] Approve artifact storage location and blob GC timing
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
