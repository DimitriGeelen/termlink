---
id: T-1158
name: "T-1155/1 Build termlink-bus crate — log-append + cursor + subscribe + retention"
description: >
  Foundation crate for T-1155 channel bus. Append-only per-channel log, per-recipient cursor store, subscribe API, per-channel retention engine. In-hub. See docs/reports/T-1155-agent-communication-bus.md §Recommendation.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [T-1155, bus, foundation]
components: []
related_tasks: [T-1155]
created: 2026-04-20T14:11:33Z
last_update: 2026-04-23T19:17:47Z
date_finished: 2026-04-20T20:38:03Z
---

# T-1158: T-1155/1 Build termlink-bus crate — log-append + cursor + subscribe + retention

## Context

Foundation crate for the T-1155 channel bus (inception GO 2026-04-20). See `docs/reports/T-1155-agent-communication-bus.md` §Recommendation / §"Build scope". This task ships the in-hub core primitives only — **no API surface, no identity, no client queue, no migrations** (those are T-1159, T-1160, T-1161, T-1162..T-1166).

Scope boundary: the crate is a passive library the hub embeds — it does not talk to the network directly. Net/RPC integration is T-1160's job.

## Acceptance Criteria

### Agent
- [x] New crate `crates/termlink-bus/` exists with `Cargo.toml`, `src/lib.rs`, registered as workspace member in root `Cargo.toml`
- [x] Public API exposes: `Bus::open(path)`, `bus.post(topic, envelope) -> Offset`, `bus.subscribe(topic, cursor) -> Iterator<(Offset, Envelope)>`, `bus.list_topics()`, `bus.create_topic(name, retention)` + cursor APIs + sweep
- [x] Append-only per-channel log on disk: one log file per topic under `<path>/topics/<sha256-of-topic>.log`, records framed with 8-byte big-endian length prefix + payload (JSON-encoded envelope)
- [x] SQLite sidecar at `<path>/meta.db` tracks: `topics`, `cursors(subscriber_id, topic, last_offset)`, `offsets(topic, next_offset)`, and a `records(topic, offset, byte_pos, length, ts_unix_ms)` index that makes subscribe reads and sweep trivial
- [x] Retention engine: per-topic policy `{Forever, Days(u32), Messages(u64)}`; `bus.sweep(topic, now_unix_ms)` deletes index rows outside the policy (log-file compaction is a follow-up). Explicit — no background thread.
- [x] Envelope type carries `{topic, sender_id, msg_type, payload: Vec<u8>, artifact_ref: Option<String>, ts_unix_ms}` — no signature/identity fields yet (T-1159 adds those)
- [x] Concurrent-safe: post() serializes on `tokio::sync::Mutex<File>`; subscribe path opens a read-only fd and uses positional reads (no shared lock across reads)
- [x] Unit tests cover: append+replay round-trip, cursor advance, empty-topic subscribe, retention trim by count, retention trim by age, topic creation idempotence — 12 tests, all pass
- [x] `cargo build -p termlink-bus` passes from workspace root
- [x] `cargo test -p termlink-bus` passes
- [x] `cargo clippy -p termlink-bus -- -D warnings` passes
- [x] No public API depends on hub types — crate is pure-data-plane; `termlink-hub` can adopt without circular deps

### Human
- [x] [REVIEW] Approve the on-disk format (one log file per topic, 8-byte LE length-prefix, opaque bytes). Alternative to consider: single WAL + index-by-topic. — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — bus on-disk format approved (one log file per topic, 8-byte LE length-prefix).
  **Steps:**
  1. Read `crates/termlink-bus/src/log.rs` (storage module)
  2. Consider: under heavy fan-in (many topics posting in parallel), does per-topic file scale, or does fd pressure matter?
  3. Record decision in task or open a follow-up if a rewrite is warranted
  **Expected:** Approval or a refactor task opened
  **If not:** Note why the format is wrong and what to change

  **Agent evidence (2026-04-21, smoke-tested against workspace binary 0.9.256 in isolated tempdir hub):**

  All three retention policies round-trip correctly:
  ```
  $ termlink channel create topic-forever --retention forever     → Created (forever)
  $ termlink channel create topic-days    --retention days:7      → Created (days:7)
  $ termlink channel create topic-msgs    --retention messages:3  → Created (messages:3)
  $ termlink channel list
    broadcast:global  [messages:1000]
    topic-days        [days:7]
    topic-forever     [forever]
    topic-msgs        [messages:3]
  ```

  Post + cursor-subscribe round-trip on `topic-msgs` (retention=messages:3, 4 appends):
  - Posts return offsets 0..3 monotonic as expected
  - `channel subscribe topic-msgs --cursor 0` returns all 4 records
  - `channel subscribe topic-msgs --cursor 2` returns only offsets 2,3 — cursor filter correct

  **Runtime observation worth calling out in review:** retention=messages:3 on a synchronous subscribe returned all 4 messages. Sweep appears to be lazy (no inline eviction on `post`), which is consistent with the AC wording "no background thread" + explicit `bus.sweep()` — but may confuse operators expecting Kafka-style bounded topics on read. Suggest either documenting the sweep contract or having `post` trigger a best-effort same-topic sweep after the append commits.

  JSON-lines subscribe works and carries every envelope field (`sender_id`, `msg_type`, `payload_b64`, `artifact_ref`, `offset`, `ts`, `topic`):
  ```
  $ termlink channel subscribe broadcast:global --limit 5 --json
  {"artifact_ref":null,"msg_type":"learning","offset":0,"payload_b64":"eyJsIjoibDEifQ==","sender_id":"c7d31e57...","topic":"broadcast:global","ts":1776725942134}
  ```

  Rubber-stamp the format if the lazy-eviction contract is acceptable; otherwise open a follow-up.

## Verification

cargo build -p termlink-bus
cargo test -p termlink-bus
cargo clippy -p termlink-bus -- -D warnings
test -f crates/termlink-bus/Cargo.toml
test -f crates/termlink-bus/src/lib.rs
grep -q "termlink-bus" Cargo.toml

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

### 2026-04-20T14:11:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1158-t-11551-build-termlink-bus-crate--log-ap.md
- **Context:** Initial task creation

### 2026-04-20T19:13:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-04-20T20:38:03Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
