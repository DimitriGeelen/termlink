---
id: T-1501
name: "agent recent / on-thread / timeline --grep — content substring filter"
description: >
  Add --grep <pattern> filter to extract_recent_posts: case-insensitive substring match against post content. AND-composes with existing peer/thread/project/msg-type filters. Threaded through cmd_agent_recent + cmd_agent_on_thread + cmd_agent_timeline. Pure helper change with new unit tests. Operator can grep the chat-arc for any phrase or task-id mention.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T21:46:00Z
last_update: 2026-05-04T22:08:00Z
date_finished: 2026-05-04T22:08:00Z
---

# T-1501: agent recent / on-thread / timeline --grep — content substring filter

## Context

The arc reading verbs (T-1492 recent, T-1493 on-thread, T-1500 timeline) all walk `extract_recent_posts` with optional peer/thread/project/msg-type filters. Missing primitive: content search. Operator wants "find every post containing 'T-1438'" or "find posts mentioning 'kasten'". This task adds `filter_grep: Option<&str>` to `extract_recent_posts` (case-insensitive substring match against post content), AND-composed with the 4 existing filters, plumbed through to all 3 reading verbs as `--grep <pattern>`.

## Acceptance Criteria

### Agent
- [x] `extract_recent_posts` gains `filter_grep: Option<&str>` (9th param) — when Some, posts whose content does not match (case-insensitive substring) are skipped
- [x] AND-composes with the 4 existing filters (peer / thread / project / msg-type)
- [x] Match against the (possibly truncated) content field — same field rendered to operator
- [x] Empty pattern treated same as None (no filter applied — defensive)
- [x] `--grep <pattern>` flag added to Recent variant
- [x] `--grep <pattern>` flag added to OnThread variant
- [x] `--grep <pattern>` flag added to Timeline variant
- [x] main.rs propagates new value through all 3 dispatches
- [x] cmd_agent_recent / cmd_agent_on_thread / cmd_agent_timeline pass filter to all callsites (one-shot json + one-shot text + watch loop)
- [x] Text-mode header includes `grep=<pattern>` suffix when set (for all 3 verbs, both one-shot and watch)
- [x] JSON envelope includes `filter_grep: "..."` when set (omitted when unset)
- [x] New unit tests in channel.rs: (1) case-insensitive match; (2) lowercase pattern matches uppercase content; (3) None keeps all (regression); (4) empty-string pattern treated as None; (5) AND-composes with filter_peer_fp
- [x] All existing extract_recent_posts unit tests pass (signature change additive — None preserves prior behavior)
- [x] `cargo build --release -p termlink` clean
- [x] Live smoke: `agent timeline --window-secs 86400 --grep T-1438` returns only posts containing T-1438 in content (or "no posts found" if none in window)

### Human
- [ ] [REVIEW] Verify --grep filtering output is operator-readable
  **Steps:**
  1. `target/release/termlink agent timeline --window-secs 86400 --grep T-1438` (run from /opt/termlink)
  2. `target/release/termlink agent timeline --window-secs 86400 --grep agent --n 10`
  3. `target/release/termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 86400 --grep T-1500`
  **Expected:** Only posts whose content matches the pattern shown; header shows `grep=<pattern>` suffix; JSON envelope includes `filter_grep` when --json
  **If not:** suggest header layout / regex support / case-sensitive flag worth adding

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release --bin termlink commands::channel::tests::recent_posts 2>&1 | tail -3 | grep -qE "test result: ok"
target/release/termlink agent recent --help 2>&1 | grep -q -- "--grep"
target/release/termlink agent on-thread --help 2>&1 | grep -q -- "--grep"
target/release/termlink agent timeline --help 2>&1 | grep -q -- "--grep"
out=$(target/release/termlink agent timeline --window-secs 86400 --n 50 --grep T-1438 --json 2>&1); echo "$out" | grep -qE '"filter_grep":"T-1438"'

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

### 2026-05-04T21:46:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1501-agent-recent--on-thread--timeline---grep.md
- **Context:** Initial task creation

### 2026-05-04T22:08:00Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)
