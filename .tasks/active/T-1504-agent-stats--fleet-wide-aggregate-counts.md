---
id: T-1504
name: "agent stats — fleet-wide aggregate counts on chat-arc"
description: >
  New 'agent stats' verb: walks the chat-arc, aggregates posts in a window into 4 buckets — by msg_type, by peer, by project, by thread. Renders top-N rows per section in text mode, full counts in JSON. Pure helper change in channel.rs (new summarize_chat_arc_stats fn). Complements presence (per-peer activity), who (single-peer detail), timeline (chronological log). Operator question: 'what's the fleet been doing this week?'

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T22:18:53Z
last_update: 2026-05-04T22:18:53Z
date_finished: 2026-05-04T22:55:00Z
---

# T-1504: agent stats — fleet-wide aggregate counts on chat-arc

## Context

`agent presence`, `agent who`, `agent timeline`, `agent on-thread` all show different views of fleet activity, but operators frequently ask the synthesis question: "what kinds of work have been happening recently?" — a single-shot aggregate of post-counts grouped by msg_type, peer, project, and thread. This task adds `agent stats` as a pure single-fetch verb that walks the chat-arc, computes the four aggregates, and renders top-N rows per section. Pure helper change in channel.rs (`summarize_chat_arc_stats` fn) — reuses meta-exclusion convention from extract_recent_posts.

## Acceptance Criteria

### Agent
- [x] New AgentAction::Stats variant: `--window-secs` (default 86400 / 1d), `--top` (default 10), `--hub`, `--json`
- [x] New pure helper `summarize_chat_arc_stats(msgs, now_ms, window_ms) -> ChatArcStats` in channel.rs
- [x] ChatArcStats struct: `total: usize`, `by_msg_type: Vec<(String, usize)>`, `by_peer: Vec<(String, usize)>`, `by_project: Vec<(String, usize)>`, `by_thread: Vec<(String, usize)>` — all sorted desc by count
- [x] Meta-type exclusion (reaction/edit/redaction/topic_metadata/receipt) applied — same convention as extract_recent_posts
- [x] Window cutoff applied (only posts in `now - window_ms` ≤ ts ≤ `now`)
- [x] cmd_agent_stats: one fetch (1000 envelopes), one summary call, render text or JSON
- [x] Text mode: 4 sections (msg_type / peers / projects / threads) each capped at top-N, with "(N more)" footer when truncated
- [x] JSON envelope: {window_secs, total, top, by_msg_type, by_peer, by_project, by_thread}
- [x] main.rs propagates new value through AgentAction::Stats dispatch
- [x] New unit tests in channel.rs: (1) by_msg_type counts include only non-meta; (2) by_peer counts each sender; (3) by_thread excludes posts with no thread metadata; (4) sorted desc by count; (5) window cutoff respected
- [x] All existing channel.rs unit tests still pass
- [x] `cargo build --release -p termlink` clean
- [x] Live smoke: `agent stats --window-secs 86400` shows non-zero counts in at least msg_type and peer sections (real arc has activity)
- [x] Live smoke: `agent stats --json --window-secs 86400` returns parseable JSON with all 4 buckets

### Human
- [ ] [REVIEW] Verify `agent stats` is operator-readable as a synthesis view
  **Steps:**
  1. `target/release/termlink agent stats --window-secs 86400` (run from /opt/termlink)
  2. `target/release/termlink agent stats --window-secs 3600 --top 5`
  **Expected:** 4 clearly-labelled sections, top peers / projects / threads / msg_types visible at a glance.
  **If not:** suggest section ordering or layout improvements.

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release --bin termlink commands::channel::tests::stats 2>&1 | tail -3 | grep -qE "test result: ok"
target/release/termlink agent stats --help 2>&1 | grep -q -- "--window-secs"
out=$(target/release/termlink agent stats --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import json,sys; d=json.load(sys.stdin); assert 'by_msg_type' in d and 'by_peer' in d and 'by_project' in d and 'by_thread' in d; print('OK')" | grep -q OK

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

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
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

## Updates

### 2026-05-04T22:18:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1504-agent-stats--fleet-wide-aggregate-counts.md
- **Context:** Initial task creation

### 2026-05-04T22:55:00Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)
