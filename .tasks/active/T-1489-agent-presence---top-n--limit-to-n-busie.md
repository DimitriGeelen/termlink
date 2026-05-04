---
id: T-1489
name: "agent presence --top N — limit to N busiest peers"
description: >
  agent presence --top N — limit to N busiest peers

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T16:06:40Z
last_update: 2026-05-04T16:12:54Z
date_finished: 2026-05-04T16:12:54Z
---

# T-1489: agent presence --top N — limit to N busiest peers

## Context

T-1482's `agent presence` returns all active peers sorted by posts desc.
On a busy fleet (10+ peers), an operator usually only cares about the
top few. This task adds `--top N` to limit the output to N busiest peers
post-sort — pure render-time slice, no helper change. JSON envelope
echoes the field so callers can confirm what was applied.

## Acceptance Criteria

### Agent
- [x] `--top N` flag added to `agent presence` (clap parses via `--help`)
- [x] When set: output is truncated to N rows post-sort (text + JSON paths both)
- [x] When unset: behavior identical to T-1482/T-1484/T-1486 baseline (JSON omits `top`/`total_peers` fields)
- [x] Clamped to [1, 1000] (clamp() applied via Option::map)
- [x] Text mode footer shows "(N of M)" when truncation occurred (when display_rows.len() < total_peers); shows just "N" otherwise
- [x] JSON envelope echoes `top` field when set + `total_peers` field so callers know the original count
- [x] Composes with `--filter-project`, `--window-secs`, `--watch` (passed through to render_presence_text)
- [x] `cargo build --release -p termlink` clean
- [x] Live smoke: `--top 1 --window-secs 86400` returns 1 row; `--top 100` against fleet of 1 returns same 1 row (no error)

### Human
- [ ] [REVIEW] Verify the truncation footer reads naturally
  **Steps:**
  1. `target/release/termlink agent presence --top 1 --window-secs 86400` (run from /opt/termlink)
  **Expected:** output ends with footer showing both the truncation and the original total
  **If not:** suggest a clearer phrasing

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent presence --help 2>&1 | grep -q -- "--top"
out=$(target/release/termlink agent presence --top 1 --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert d.get('top') == 1, d; assert isinstance(d.get('total_peers'), int)"
out=$(target/release/termlink agent presence --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert 'top' not in d, 'unset top must not appear'"
out=$(target/release/termlink agent presence --top 1 --window-secs 86400 2>&1); echo "$out" | grep -qE "PEER_FP"

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

**Rationale:** Small operator-polish flag. Composes cleanly with all existing presence flags (filter-project, window-secs, watch) without touching helpers. Backward-compatible JSON envelope — `top` and `total_peers` fields appear only when flag is set. Truncation footer reports both shown and total counts so operator never loses sight of the fleet size.

**Evidence:**
- Live: `agent presence --top 1` → 1-row table, footer `1 peer(s) active in window=86400s` (1==total, no truncation marker)
- Live: `agent presence --top 100` against 1-peer fleet → same 1 row (clamp pass-through)
- Live JSON `--top 1`: `{"top": 1, "total_peers": 1, "peers": [...], "window_secs": 86400}`
- Live JSON no-top: no `top`/`total_peers` keys (backward-compatible)
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

### 2026-05-04T16:06:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1489-agent-presence---top-n--limit-to-n-busie.md
- **Context:** Initial task creation

### 2026-05-04T16:12:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
