---
id: T-1480
name: "agent contact --require-online — Q3 fail-fast flag (T-1425 Phase-2 backlog)"
description: >
  agent contact --require-online — Q3 fail-fast flag (T-1425 Phase-2 backlog)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-04T12:31:51Z
last_update: 2026-05-18T17:34:03Z
date_finished: 2026-05-04T12:50:10Z
---

# T-1480: agent contact --require-online — Q3 fail-fast flag (T-1425 Phase-2 backlog)

## Context

T-1425 Q3 deferred Phase-2 flag: `agent contact <name> --require-online` should
fail-fast when the peer hasn't been seen on the canonical liveness arc within
a recent window. Q3 chose default queue + opt-in fail-fast; the default verb
shipped fire-and-forget; this task lands the opt-in. Presence signal is
`agent-chat-arc` heartbeat activity (T-1455 cron pattern) — peer FP appearing
as `sender_id` on any non-meta msg in the last N seconds means "online enough
to receive". No new RPC; reuses `channel.subscribe` to walk the topic.

## Acceptance Criteria

### Agent
- [x] `--require-online` flag added to `agent contact` (clap parses via `--help`)
- [x] `--online-window-secs <N>` flag (default 300, clamped to [10, 86400])
- [x] Pre-flight: when `--require-online` is set, query `agent-chat-arc` for peer FP activity in last <N> seconds via channel.subscribe (last 500 msgs scanned). If `posts_in_window == 0` → exit code 9, error names peer FP, last_seen (or "never"), and the configured window
- [x] Live success path: peer recently active → no behavior change (proceeds to dm post)
- [x] Dry-run + `--require-online`: presence check still runs; result attached to preview JSON as `online_check: { online, last_seen_ms, posts_in_window, window_secs }`. Dry-run exits 0 either way (preview is non-failing)
- [x] Pure helper `evaluate_presence(msgs, peer_fp, now_ms, window_ms) -> PresenceCheck` in `commands/channel.rs` — no I/O
- [x] 5+ unit tests for `evaluate_presence`: peer never seen / only outside window / only inside / mixed (last_seen_ms == max ts) / meta-msg filter (reaction/topic_metadata excluded)
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink --bin termlink presence` passes (>=5 tests)
- [x] Live smoke against local hub: `--require-online` against own FP succeeds (just posted heartbeat would count); against an obviously-fake fp `00deadbeef00deadbeef` exits 9
- [x] `--dry-run --require-online` against fake FP prints preview JSON with `online_check.online=false` and exit 0
- [x] Offline error wording names the FP, the configured window, and the last_seen state (moved from Human AC per PL-169 — operator-actionability is operationalizable as `names {fp, window, last_seen} + offers recovery path`, all mechanically verifiable).
  **Evidence (2026-05-18T11:00Z):** `target/release/termlink agent contact --target-fp 00deadbeef00deadbeef --message x --require-online --online-window-secs 60` exited code 9, stderr was `error: peer fp=00deadbeef00deadbeef not online — last seen: never (no posts on agent-chat-arc), window=60s. Re-run without --require-online to queue the post (chat-arc is offset-durable), or wait for the peer's next heartbeat.` All four criteria present: exit 9 ✓, FP named ✓, window=60s named ✓, "last seen: never" named ✓; bonus: two recovery paths offered (drop flag to queue / wait for heartbeat) — operator-actionable beyond the spec minimum.

### Human
<!-- All criteria are mechanically verifiable and have been moved to ### Agent
     per PL-169. Original [REVIEW] AC was sound at filing but the success
     criterion ("names FP + window + last_seen") is pure string-match. -->
- (none — all moved to ### Agent per PL-169)

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release -p termlink --bin termlink presence 2>&1 | grep -qE "test result: ok\. [5-9]+ passed|test result: ok\. 1[0-9] passed"
target/release/termlink agent contact --help 2>&1 | grep -q "require-online"
target/release/termlink agent contact --help 2>&1 | grep -q "online-window-secs"
out=$(target/release/termlink agent contact --target-fp 00deadbeef00deadbeef --message x --require-online --online-window-secs 60 --dry-run --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert d['dry_run'] is True; assert d['online_check']['online'] is False, d"

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Recommendation

**Recommendation:** GO

**Rationale:** Mechanical step from the Human [REVIEW] AC ran live; output hits all three Expected criteria (exit 9, FP named, "last seen: never" + window). Wording also includes the operator's recovery path (queue alternative, peer-heartbeat hint), exceeding minimum-actionable.

**Evidence:**
- Live invocation: `target/release/termlink agent contact --target-fp 00deadbeef00deadbeef --message x --require-online --online-window-secs 60`
- Captured stderr: `error: peer fp=00deadbeef00deadbeef not online — last seen: never (no posts on agent-chat-arc), window=60s. Re-run without --require-online to queue the post (chat-arc is offset-durable), or wait for the peer's next heartbeat.`
- Captured exit code: 9
- Verification block: 5/5 PASS (this run)
- Unit tests: 7/7 PASS (presence_*)

## Updates

### 2026-05-04T12:31:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1480-agent-contact---require-online--q3-fail-.md
- **Context:** Initial task creation

### 2026-05-04 — agent shipped + verified
- **Action:** Implemented `--require-online` + `--online-window-secs` flags on `agent contact`. Added `PresenceCheck` struct + `evaluate_presence` pure helper + `check_peer_online_via_chat_arc` async probe in `commands/channel.rs`. 7 unit tests added (presence_*). All Agent ACs verified live against local hub. Verification block: 5/5 PASS.
- **Files:**
  - `crates/termlink-cli/src/commands/channel.rs` (+helper +tests, ~140 lines)
  - `crates/termlink-cli/src/commands/agent.rs` (+pre-flight + dry-run integration)
  - `crates/termlink-cli/src/cli.rs` (+2 flags on Contact variant)
  - `crates/termlink-cli/src/main.rs` (+dispatch)
- **Output:** target/release/termlink built clean. 7/7 presence tests pass. Live smoke: fake FP exits 9 with operator-actionable message; dry-run + fake FP shows online=false in preview.

### 2026-05-04T12:50:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-05-18T17:33:48Z — status-update [task-update-agent]
- **Change:** owner: human → agent
- **Reason:** Operator authorized via conversation option 3: same as T-1481. PL-169 applies.
