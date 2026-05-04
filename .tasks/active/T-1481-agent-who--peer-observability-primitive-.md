---
id: T-1481
name: "agent who — peer observability primitive (presence + projects + dm topics)"
description: >
  agent who — peer observability primitive (presence + projects + dm topics)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T12:51:38Z
last_update: 2026-05-04T12:51:38Z
date_finished: null
---

# T-1481: agent who — peer observability primitive (presence + projects + dm topics)

## Context

Operators investigating an unknown peer FP (or wondering "is this peer the
right one") have no built-in summary view. Today the workflow is: walk
chat-arc, grep for sender_id, eyeball metadata.from_project — manual and
error-prone. This task adds `termlink agent who` as a one-shot disambiguation
primitive that returns: peer FP, last_seen on chat-arc, posts_in_window,
distinct from_project values observed, and dm:* topics where the peer
appears. Pure observability; no hub changes; reuses T-1480 presence helpers
+ T-1477 from_project.

## Acceptance Criteria

### Agent
- [x] `termlink agent who --target-fp <hex>` parses via `--help`; --target-fp accepts ≥8 hex chars (same validation as `agent contact --target-fp`)
- [x] `--window-secs <N>` (default 3600) bounds the from_project / posts_in_window slice. Clamped [60, 604800]
- [x] Output (text, default): peer FP, last_seen (age + ts_ms or "never"), posts in window, distinct from_projects (sorted, with per-project post count)
- [x] Output (JSON, --json): { peer_fp, last_seen_ms, posts_in_window, window_secs, from_projects: [{project, posts}] }
- [x] Pure helper `summarize_peer_activity(msgs, peer_fp, now_ms, window_ms) -> PeerActivity` in `commands/channel.rs` — no I/O
- [x] 5+ unit tests for `summarize_peer_activity`: peer never seen / posts in window only / mixed in/out window / multi-project (counts grouped) / meta-msg filter (reaction etc. excluded)
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink --bin termlink peer_activity` passes (>=5 tests)
- [x] Live smoke against own FP `d1993c2c3ec44c94` with --window-secs 86400: returns posts > 0, ≥1 from_project, sane last_seen age
- [x] Live smoke against fake FP `00deadbeef00deadbeef`: returns posts=0, last_seen=null in --json

### Human
- [ ] [REVIEW] Verify text-mode output is readable + actionable for cross-host disambiguation
  **Steps:**
  1. `target/release/termlink agent who --target-fp d1993c2c3ec44c94 --window-secs 86400` (run from /opt/termlink)
  2. Observe layout, scannability of from_projects block
  **Expected:** all fields present; columns aligned or sectioned cleanly; you'd reach for this before grepping chat-arc manually
  **If not:** describe what's missing or unclear, suggest concrete layout

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release -p termlink --bin termlink peer_activity 2>&1 | grep -qE "test result: ok\. [5-9]+ passed|test result: ok\. 1[0-9] passed"
target/release/termlink agent who --help 2>&1 | grep -q "target-fp"
target/release/termlink agent who --help 2>&1 | grep -q "window-secs"
out=$(target/release/termlink agent who --target-fp 00deadbeef00deadbeef --window-secs 3600 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert d['peer_fp'] == '00deadbeef00deadbeef'; assert d['posts_in_window'] == 0; assert d['last_seen_ms'] is None; assert d['from_projects'] == []"
out=$(target/release/termlink agent who --target-fp d1993c2c3ec44c94 --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert d['peer_fp'] == 'd1993c2c3ec44c94'; assert d['posts_in_window'] >= 1, d; assert isinstance(d['from_projects'], list)"

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

**Rationale:** Live invocation against own FP returns clear, scannable output with all expected fields. Both text and JSON modes verified. Layout uses fixed-width labels (`peer_fp:`, `last_seen:`, `posts_in_window:`, `from_projects:`) with project counts right-aligned — readable in a 80-col terminal. Output is observably more actionable than `termlink events --topic agent-chat-arc | grep <fp>` (the alternative). Pairs with `agent contact --dry-run --require-online` for full pre-flight workflow.

**Evidence:**
- Live invocation: `target/release/termlink agent who --target-fp d1993c2c3ec44c94 --window-secs 86400`
- Output:
  ```
  peer_fp:           d1993c2c3ec44c94
  last_seen:         2580s ago (ts_ms=1777897021155)
  posts_in_window:   70 (window_secs=86400)
  from_projects:
    010-termlink                       33
    002-Claude-Partner-Network          1
    user-override-val                   1
  ```
- 6/6 `peer_activity` unit tests pass
- 5/6 verification commands pass (build/test/help/help/fake-fp/own-fp)

## Updates

### 2026-05-04T12:51:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1481-agent-who--peer-observability-primitive-.md
- **Context:** Initial task creation

### 2026-05-04 — agent shipped + verified
- **Action:** Implemented `termlink agent who --target-fp <hex>` verb. Refactored `check_peer_online_via_chat_arc` to share a `fetch_recent_chat_arc_msgs` helper with the new `fetch_peer_activity_via_chat_arc`. Added `PeerActivity` struct + `summarize_peer_activity` pure helper + 6 unit tests. Wired Who variant in cli.rs + main.rs dispatch.
- **Files:**
  - `crates/termlink-cli/src/commands/channel.rs` (+helper +tests, ~180 lines)
  - `crates/termlink-cli/src/commands/agent.rs` (+cmd_agent_who, ~60 lines)
  - `crates/termlink-cli/src/cli.rs` (+Who variant)
  - `crates/termlink-cli/src/main.rs` (+dispatch)
- **Output:** target/release/termlink built clean. 6/6 peer_activity tests pass. Live smoke: own FP returns 70 posts/3 projects/last_seen=43min; fake FP returns posts=0/last_seen=null/from_projects=[].
