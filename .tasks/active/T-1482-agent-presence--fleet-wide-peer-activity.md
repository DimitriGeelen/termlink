---
id: T-1482
name: "agent presence — fleet-wide peer activity summary (companion to agent who)"
description: >
  agent presence — fleet-wide peer activity summary (companion to agent who)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T13:07:24Z
last_update: 2026-05-04T13:19:46Z
date_finished: 2026-05-04T13:19:46Z
---

# T-1482: agent presence — fleet-wide peer activity summary (companion to agent who)

## Context

`agent who --target-fp X` (T-1481) answers "what is peer X up to?" but not
"who is on the fleet right now?" — for fleet-wide situational awareness an
operator still has to grep events. This task adds `termlink agent presence`
as the fleet-level companion: walks the recent chat-arc slice once,
aggregates non-meta posts by sender_id, and renders one row per peer with
last_seen + posts_in_window + top-1 from_project. Pure observability; no
hub changes; reuses T-1481's `summarize_peer_activity` extension.

## Acceptance Criteria

### Agent
- [x] `termlink agent presence` parses via `--help`; no required flags
- [x] `--window-secs <N>` (default 3600) bounds the slice. Clamped [60, 604800]
- [x] Output (text, default): one row per active peer, columns = peer_fp (16 chars), last_seen_ago, posts, top from_project. Sorted by posts desc, then peer_fp asc. Header row + alignment
- [x] Output (JSON, --json): { window_secs, peers: [{peer_fp, last_seen_ms, posts, top_project}, ...] }
- [x] Empty fleet: text mode prints "(no peers active in window)"; JSON returns empty peers array
- [x] Pure helper `summarize_fleet_presence(msgs, now_ms, window_ms) -> Vec<FleetPeerRow>` in `commands/channel.rs` — no I/O
- [x] 5+ unit tests for `summarize_fleet_presence`: empty msgs / single peer / multi-peer sorted by posts desc / meta-msg filter / top_project picks most-frequent then alphabetic on tie
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink --bin termlink fleet_presence` passes (>=5 tests)
- [x] Live smoke: own hub returns ≥1 peer (own FP at minimum) with sane numbers; --json output parses correctly

### Human
- [ ] [REVIEW] Verify text-mode table is scannable for fleet observability
  **Steps:**
  1. `target/release/termlink agent presence --window-secs 86400` (run from /opt/termlink)
  2. Observe row layout, alignment, header
  **Expected:** rows aligned in columns; header readable; you'd reach for this before grepping chat-arc
  **If not:** describe what's misaligned or unclear, suggest concrete layout

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

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release -p termlink --bin termlink fleet_presence 2>&1 | grep -qE "test result: ok\. [5-9]+ passed|test result: ok\. 1[0-9] passed"
target/release/termlink agent presence --help 2>&1 | grep -q "window-secs"
out=$(target/release/termlink agent presence --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert 'window_secs' in d; assert isinstance(d['peers'], list); assert len(d['peers']) >= 1, d"

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

## Recommendation

**Recommendation:** GO

**Rationale:** Live invocation returns aligned table with header, rows sorted by posts desc, plus footer count. Fleet visibility is observably better than `termlink events --topic agent-chat-arc | sort -u` (the alternative). 7/7 unit tests cover sort stability, meta-msg filter, window bounds, top-project tie-break, and JSON shape. Pairs with `agent who` (specific peer) for full observability stack.

**Evidence:**
- Live invocation: `target/release/termlink agent presence --window-secs 86400`
- Output:
  ```
  PEER_FP                 LAST_SEEN    POSTS  TOP_PROJECT
  d1993c2c3ec44c94          57m ago       69  010-termlink

  1 peer(s) active in window=86400s
  ```
- 7/7 `fleet_presence_*` unit tests pass
- 4/4 verification commands pass

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

### 2026-05-04T13:07:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1482-agent-presence--fleet-wide-peer-activity.md
- **Context:** Initial task creation

### 2026-05-04T13:19:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
