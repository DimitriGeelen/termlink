---
id: T-1285
name: "T-243 dep: Audit + harden channel.* ordered durable delivery"
description: >
  Verify channel.* guarantees ordered durable delivery under: (a) concurrent multi-publisher writes, (b) hub restart, (c) subscriber reconnect mid-stream. This is Agent A's NO-GO check from T-243 inception and Agent C's crash gap. If gaps found, fix before any dialog.heartbeat or metadata-extension work proceeds — append-only log model collapses without it. Foundation for all other T-243 child tasks.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-243, channel, reliability]
components: []
related_tasks: []
created: 2026-04-26T09:31:53Z
last_update: 2026-04-26T09:42:02Z
date_finished: null
---

# T-1285: T-243 dep: Audit + harden channel.* ordered durable delivery

## Context

Audit foundation for T-243 multi-turn dialog primitive. Agent A's NO-GO trigger from the T-243 inception: *"discover that TermLink channel retention silently drops events under hub restart or that event ordering breaks with multiple simultaneous publishers."*

Code surveyed: `crates/termlink-bus/src/{lib,log,meta}.rs`, `crates/termlink-hub/src/channel.rs`.

## Audit Findings (2026-04-26)

### ✅ Ordering under concurrent writers — SAFE

`LogAppender::append` holds `tokio::sync::Mutex<File>` over the entire write; `Bus::post` then calls `Meta::record_append` which uses a SQLite transaction (`SELECT next_offset` → `UPDATE` → `INSERT INTO records`) under `std::sync::Mutex<Connection>`.

**Subtlety:** byte_pos is assigned inside the appender mutex, offset is assigned inside the meta mutex. The two are separate critical sections, so byte_pos order in the log file CAN differ from offset order in SQLite.

**Read-side correctness:** `Bus::subscribe` orders by offset (`ORDER BY offset`) and reads bytes positionally — subscribers see logical (offset) order regardless of byte order. Correct.

**Invariant to document:** "Never read the log file in byte order without consulting SQLite." A future helper that streams the raw log expecting offset order would be wrong.

### ❌ Subscriber gap detection on retention sweep — MISSING (must-fix for T-1286)

`Bus::subscribe(topic, cursor)` returns records with `offset >= cursor`. If `Bus::sweep` ran between the subscriber's last read and reconnect, records with `offset < oldest_remaining` are silently gone from SQLite. The subscriber gets the records still present and **no signal** that anything was missed.

**Why this blocks T-1286:** dialog.heartbeat will rely on subscribers reconnecting after disconnect (mid-conversation). If a heartbeat (or any event) was swept while the subscriber was disconnected, the conversation drops a turn silently. Conversation state diverges between peers without anyone noticing.

**Fix shape:** subscribe should detect `cursor < min(records.offset)` and return a typed gap-marker as the first iter item, OR Bus should expose `oldest_offset(topic)` so subscribers can compare and decide. Tiny structural change; large reliability win.

**Prior art note:** T-009 GO rationale on the older EventBus said *"add gap detection to EventBus (cursor < oldest_seq → warning/event)"* — same gap, different bus. The T-1155 bus is a clean reimplementation that re-introduced the omission.

### ⚠️ Durability — fsync missing (should-fix, not blocking)

`LogAppender::append` calls `guard.flush()` — that's stdio flush, not `fsync(2)`. Bytes hit OS page cache; on power loss / kernel panic before the OS pages get written they vanish. SQLite uses default DELETE journal mode (durable but no PRAGMA explicitly set).

**Effect on T-243 use cases:**
- Heartbeats: small frequent writes; losing the last ~5s of heartbeats on power loss is acceptable
- Conversation turns: losing a turn on power loss is harder — the consumer assumes "B replied, I read it" but the file lost it. Less bad than gap-without-detection because at least sender knows it sent and consumer hadn't read yet, but still a hole.

**Fix shape:** add per-topic durability flag (`Retention::Forever` could imply durable, or add a separate flag), call `file.sync_data()` (POSIX fdatasync) on append for durable topics. Independently testable.

### ⚠️ Crash recovery — orphan bytes possible (should-fix, low blast radius)

If hub crashes between successful `appender.append()` and SQLite `record_append` commit:
- Bytes are in the log file (and possibly fsync'd by OS)
- SQLite has no record pointing at those bytes
- On restart, `next_offset` is unchanged; next post writes bytes at end-of-file (past orphans) with the same logical offset — correct.
- Storage leaks; no log compaction tool.

Fixing this needs either (a) write-ahead the SQLite metadata before the file write, or (b) a recovery scan on `Bus::open` that detects orphaned bytes. Not urgent for T-243.

## Acceptance Criteria

### Agent
- [x] Surveyed `termlink-bus` Bus/LogAppender/Meta + `termlink-hub` channel.rs
- [x] Verified ordering safety under concurrent writers (offset assignment is monotonic via SQLite transaction)
- [x] Identified missing subscriber gap-detection on retention sweep — must-fix before T-1286 (dialog.heartbeat) lands
- [x] Identified missing fsync — should-fix, not blocking
- [x] Audit findings documented in task file
- [x] Implement subscriber gap detection — `Bus::oldest_offset(topic) -> Option<Offset>` exposed (lib.rs); SQLite `MIN(offset)` query in Meta::oldest_offset (meta.rs)
- [x] Add test: post 5, sweep to 2, oldest_offset reports 3 → cursor=1 detects gap (oldest_offset_reflects_sweep_drops). Plus oldest_offset_unknown_topic_errors for the error path.
- [x] cargo test passes for `termlink-bus` — 27/27 tests pass

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
cargo test -p termlink-bus --lib 2>&1 | tail -20 | grep -E "test result: ok|test result: FAILED|passed"

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

### 2026-04-26T09:31:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1285-t-243-dep-audit--harden-channel-ordered-.md
- **Context:** Initial task creation

### 2026-04-26T09:39:39Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
