---
id: T-1251
name: "T-1164d Legacy file.* event deprecation + inbox.rs cleanup + PL-011 closure"
description: >
  Mark legacy file.init/chunk/complete event-name path as deprecated, integrate blob GC with retention engine (T-1158), close PL-011 (send-file delivery confirmation) with structural-fix evidence pointing at T-1164. Depends on T-1164b + T-1164c.

status: work-completed
workflow_type: decommission
owner: agent
horizon: now
tags: [T-1164, T-1155, bus, artifact, PL-011]
components: [crates/termlink-bus/src/artifact_store.rs, crates/termlink-bus/src/error.rs, crates/termlink-bus/src/lib.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-hub/src/artifact.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/inbox.rs, crates/termlink-hub/src/lib.rs, crates/termlink-hub/src/router.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-protocol/src/control.rs]
related_tasks: [T-1164, T-1164b, T-1164c, T-1158]
created: 2026-04-25T11:43:54Z
last_update: 2026-04-25T13:30:48Z
date_finished: 2026-04-25T13:30:48Z
---

# T-1251: T-1164d Legacy file.* event deprecation + inbox.rs cleanup + PL-011 closure

## Context

Final pre-retirement step in T-1164. With T-1248/1249/1250 shipped, the new `channel.post {msg_type:artifact}` + `artifact.put/get` flow is the primary path. This task lays scaffolding for T-1166's eventual full retirement: deprecation warnings on legacy paths, blob GC primitives so the retention engine has a hook for unreferenced artifacts, and PL-011 closure now that `SendOutcome::Sent { sha256, channel_offset }` provides structural delivery proof. T-1166 still depends on the 60-day production observation gate before actual code removal.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-bus/src/artifact_store.rs` gains `pub fn sweep(&self, referenced: &HashSet<String>) -> io::Result<u64>` that walks `<root>/<2-hex>/<sha256>` files, deletes blobs whose sha is NOT in the referenced set, and returns the count pruned. Skips `.staging` directory. Empty-prefix-dir cleanup is best-effort (ignore errors).
- [x] Unit tests in artifact_store.rs: `sweep_removes_unreferenced_keeps_referenced` (puts 3 blobs, references 1, expects 2 pruned + 1 retained); `sweep_empty_set_prunes_all`; `sweep_skips_staging_dir` (creates a .staging entry, sweep doesn't touch it).
- [x] `crates/termlink-hub/src/inbox.rs` emits a one-shot `tracing::info!` deprecation log on the first legacy `file.*` event accepted into the inbox per process. Message: "T-1251: legacy file.* events received — sender should migrate to channel.post {msg_type:artifact}". Use `std::sync::OnceLock<()>` for warn-once.
- [x] `.context/project/learnings.yaml` PL-011 entry gains an `application:` closure note pointing to T-1164's SendOutcome::Sent structural-fix.
- [x] `cargo build && cargo test --workspace --lib && cargo clippy --workspace --tests -- -D warnings` all pass. Zero regressions (760 tests).

## Notes / Deferred

- **Hub-side blob GC scheduling:** `ArtifactStore::sweep` is the primitive; wiring it into a periodic sweep against current channel-log artifact_refs is deferred (depends on a hub-side iterator that walks all topics' envelopes, which T-1158's retention engine doesn't yet expose). T-1166 gates on production observation anyway — separate task there.

## Verification

cargo build
cargo test -p termlink-bus artifact_store
cargo clippy --workspace --tests -- -D warnings
grep -q "fn sweep" /opt/termlink/crates/termlink-bus/src/artifact_store.rs
grep -q "T-1251: legacy file.\* events received" /opt/termlink/crates/termlink-hub/src/inbox.rs
grep -q "Closed by T-1164" /opt/termlink/.context/project/learnings.yaml

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

### 2026-04-25T11:43:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1251-t-1164d-legacy-file-event-deprecation--i.md
- **Context:** Initial task creation

### 2026-04-25T13:27:54Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-25T13:30:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
