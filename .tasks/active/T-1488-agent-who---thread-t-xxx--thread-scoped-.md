---
id: T-1488
name: "agent who --thread <T-XXX> — thread-scoped peer activity filter"
description: >
  agent who --thread <T-XXX> — thread-scoped peer activity filter

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T15:50:55Z
last_update: 2026-05-04T16:00:12Z
date_finished: 2026-05-04T16:00:12Z
---

# T-1488: agent who --thread <T-XXX> — thread-scoped peer activity filter

## Context

T-1481 / T-1483 ship `agent who` for per-peer activity. T-1429 Phase-2
gave us `metadata._thread=<T-XXX>` for canonical task-id routing. The
symmetric question — "what's peer X been doing on T-1485?" — currently
requires `agent who | grep` or `events --topic agent-chat-arc | jq`.
This task adds `--thread <T-XXX>` to scope the activity summary: only
posts whose `metadata._thread == <T-XXX>` count toward
posts_in_window / from_projects. Mirrors the T-1484 filter pattern for
fleet presence. Pure helper extension, signature change with all
callers updated.

## Acceptance Criteria

### Agent
- [x] `--thread <name>` flag added to `agent who` (clap parses via `--help`)
- [x] When set: only posts with `metadata._thread == <name>` count toward posts_in_window / from_projects (last_seen still walks whole peer history for "really how active is this peer at all")
- [x] When unset: behavior identical to T-1481 baseline (regression-safe — JSON envelope omits `filter_thread`)
- [x] `summarize_peer_activity` helper extended with `filter_thread: Option<&str>` parameter (signature change with all callers updated)
- [x] JSON envelope echoes `filter_thread` field when filter is set (omitted otherwise to stay backward-compatible)
- [x] Text mode: header line `# filter_thread=<name>` printed when filter is set; existing layout otherwise unchanged
- [x] 4+ unit tests: filter-matches / filter-excludes-untagged / no-match-returns-zero / last-seen-independent-of-filter
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink --bin termlink peer_activity` passes (10/10 — 6 baseline + 4 new)
- [x] Live smoke: `agent who --target-fp d1993c2c3ec44c94 --thread T-1487 --window-secs 86400` returns 0 posts_in_window (chat-arc has no T-1487-tagged posts — expected) but last_seen stays populated

### Human
- [ ] [REVIEW] Verify thread-filter output makes sense
  **Steps:**
  1. `target/release/termlink agent who --target-fp d1993c2c3ec44c94 --window-secs 86400` (run from /opt/termlink) — note posts_in_window
  2. `target/release/termlink agent who --target-fp d1993c2c3ec44c94 --thread T-1487 --window-secs 86400`
  3. Compare
  **Expected:** filtered count ≤ unfiltered count; from_projects narrows to projects that posted on that thread; output is operator-readable
  **If not:** describe what's missing or unclear, suggest concrete improvement

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release -p termlink --bin termlink peer_activity 2>&1 | grep -qE "test result: ok\. (1[0-9]|[2-9][0-9])"
target/release/termlink agent who --help 2>&1 | grep -q -- "--thread"
out=$(target/release/termlink agent who --target-fp d1993c2c3ec44c94 --thread T-1487 --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert d.get('filter_thread') == 'T-1487', d; assert isinstance(d.get('posts_in_window'), int)"
out=$(target/release/termlink agent who --target-fp d1993c2c3ec44c94 --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert 'filter_thread' not in d, 'unset filter must not appear in JSON'"

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

**Rationale:** Closes the symmetric-to-T-1484 thread-scoping gap on agent who. Pure helper extension with 4 new unit tests covering the key edge cases including the design choice that `last_seen` stays filter-independent (validated live: peer reads as "alive, but not on this thread"). Backward-compatible — `filter_thread` field absent from JSON when flag is unset, so existing T-1481 consumers see no shape change.

**Evidence:**
- Live invocations:
  - Unfiltered: `agent who --target-fp d1993c2c3ec44c94 --window-secs 86400` → 62 posts_in_window, 35 distinct from_projects (top: 010-termlink/33)
  - Filtered: `agent who ... --thread T-1487` → header line `# filter_thread=T-1487`, 0 posts_in_window, but last_seen still `2548s ago` (filter-independent design validated)
  - Filtered JSON: `filter_thread: "T-1487"` echoed in envelope; from_projects empty
  - Unfiltered JSON: no `filter_thread` field (backward-compatible)
- Unit tests: 10/10 `peer_activity_*` pass (6 baseline + 4 thread-filter tests)
- Verification: 5/5 commands pass

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

### 2026-05-04T15:50:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1488-agent-who---thread-t-xxx--thread-scoped-.md
- **Context:** Initial task creation

### 2026-05-04T16:00:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-06-13T13:41:08Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `target/release/termlink agent who --target-fp d1993c2c3ec44c94 --window-secs 86400`; `target/release/termlink agent who --target-fp d1993c2c3ec44c94 --thread T-1487 --window-secs 86400`
- **Result:** exit=0,0; ok — thread filter narrows posts 19→0; filter_thread header shown
- **Output:**
  ```
  $ target/release/termlink agent who --target-fp d1993c2c3ec44c94 --window-secs 86400
  peer_fp:           d1993c2c3ec44c94
  last_seen:         1428s ago (ts_ms=1781356621735)
  posts_in_window:   19 (window_secs=86400)
  from_projects:
    010-termlink                       18
    100-Video-riper-and-translation-app      1
  [exit=0]
  $ target/release/termlink agent who --target-fp d1993c2c3ec44c94 --thread T-1487 --window-secs 86400
  # filter_thread=T-1487
  peer_fp:           d1993c2c3ec44c94
  last_seen:         1428s ago (ts_ms=1781356621735)
  posts_in_window:   0 (window_secs=86400)
  from_projects:     (none observed in window)
  [exit=0]
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm.
