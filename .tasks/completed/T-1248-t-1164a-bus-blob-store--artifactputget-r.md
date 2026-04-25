---
id: T-1248
name: "T-1164a Bus blob store + artifact.put/get RPC"
description: >
  Foundational blob store primitive for T-1164. Implements <bus-path>/artifacts/<sha256> content-addressed storage + artifact.put / artifact.get RPC methods. Unblocks T-1164b (sender) and T-1164c (receiver).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-1164, T-1155, bus, artifact]
components: [crates/termlink-bus/src/artifact_store.rs, crates/termlink-bus/src/error.rs, crates/termlink-bus/src/lib.rs, crates/termlink-hub/src/artifact.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/lib.rs, crates/termlink-hub/src/router.rs, crates/termlink-protocol/src/control.rs]
related_tasks: [T-1164, T-1155, T-1158]
created: 2026-04-25T11:43:45Z
last_update: 2026-04-25T11:53:33Z
date_finished: 2026-04-25T11:53:33Z
---

# T-1248: T-1164a Bus blob store + artifact.put/get RPC

## Context

Foundational sub-task of T-1164. Introduces a content-addressed blob store at `<bus-path>/artifacts/<sha256>` plus two RPC methods (`artifact.put`, `artifact.get`) so the channel.post artifact_ref slot already in the protocol (T-1160) has actual storage backing it. T-1249 (sender) and T-1250 (receiver) consume this surface; T-1251 wires GC into the existing retention engine (T-1158).

Design constraints from T-1164 audit:
- Content-addressed (sha256) → idempotent put, dedup by hash.
- Filesystem layout: `<bus-path>/artifacts/<first-2-hex>/<full-sha256>` (sharded, avoids 100k+ files in one dir).
- Inline threshold: payloads ≤64KB go directly in channel.post payload_b64; >64KB go via artifact.put + artifact_ref.
- No GC primitive in this task — T-1251 integrates with retention.
- Streaming put/get (chunked over JSON-RPC): respects existing MAX_PAYLOAD_SIZE; client-driven offset/total semantics like the legacy file.chunk path.

## Acceptance Criteria

### Agent
- [x] New module `crates/termlink-bus/src/artifact_store.rs` exposes `ArtifactStore { fn put, fn get, fn exists, fn path_for, fn put_streaming }`. Sharded layout `artifacts/<first-2-hex>/<full-sha256>`. Idempotent put (same bytes → same sha → no rewrite if exists). Evidence: 264 LOC + 8 unit tests.
- [x] Protocol method constants `ARTIFACT_PUT = "artifact.put"` and `ARTIFACT_GET = "artifact.get"` added to `crates/termlink-protocol/src/control.rs::method` mod (lines 123-138). Doc comments match the channel.* style. Test `artifact_method_constants_are_stable` asserts the strings.
- [x] Hub-side handlers in `crates/termlink-hub/src/artifact.rs` implement streaming put/get. `artifact.put { staging_id, offset, chunk_b64, is_final, expected_sha256? }` → `{ok, in_progress, sha256?, bytes_received|total_bytes}`. `artifact.get { sha256, offset?, max_bytes? }` → `{chunk_b64, bytes_returned, eof, total_bytes}`. Wired into router.rs match arms (line 91/94) and hub.capabilities reply (line 771/772).
- [x] Hub advertises new methods in `hub.capabilities` (T-1215 surface) — verified by grepping router.rs:771-772.
- [x] Unit tests in `termlink-bus`: round-trip, idempotent re-put, sha256 mismatch rejection on `expected_sha256`, sharded path layout, streaming chunked + offset-mismatch detection. 8/8 pass.
- [x] Hub-level handler tests in `termlink-hub::artifact`: 5MB chunked round-trip via the JSON-RPC handler signatures, get_unknown_returns_error, hash mismatch rejection. 4/4 pass.
- [x] `cargo build && cargo test -p termlink-bus -p termlink-hub -p termlink-protocol && cargo clippy --workspace -- -D warnings` all pass. Workspace lib tests: 751 passed, 0 failed (up from 738 baseline — delta = 13 new tests).
- [x] Fabric cards registered: `.fabric/components/crates-termlink-bus-src-artifact_store.yaml` + `crates-termlink-hub-src-artifact.yaml`. `fw fabric drift` reports 0 unregistered.

## Verification

cargo build
cargo test -p termlink-bus artifact
cargo test -p termlink-hub artifact
cargo test -p termlink-protocol
cargo clippy -- -D warnings
test -f /opt/termlink/crates/termlink-bus/src/artifact_store.rs
grep -q "ARTIFACT_PUT" /opt/termlink/crates/termlink-protocol/src/control.rs
grep -q "ARTIFACT_GET" /opt/termlink/crates/termlink-protocol/src/control.rs

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

### 2026-04-25T11:43:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1248-t-1164a-bus-blob-store--artifactputget-r.md
- **Context:** Initial task creation

### 2026-04-25T11:45:13Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-25T11:53:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
