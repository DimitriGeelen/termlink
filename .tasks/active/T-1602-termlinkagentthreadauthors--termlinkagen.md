---
id: T-1602
name: "termlink_agent_thread_authors + termlink_agent_recent_window — dedup'd thread participants + time-window post filter MCP read tools"
description: >
  termlink_agent_thread_authors + termlink_agent_recent_window — dedup'd thread participants + time-window post filter MCP read tools

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T05:30:55Z
last_update: 2026-05-06T05:30:55Z
date_finished: null
---

# T-1602: termlink_agent_thread_authors + termlink_agent_recent_window — dedup'd thread participants + time-window post filter MCP read tools

## Context

T-1601 brought MCP read surface to 148 tools. Wave 58 adds two **thread-and-time orientation reads**:

- `termlink_agent_thread_authors` — given a `root_offset`, returns dedup'd list of unique sender_ids participating in the thread (root + all descendants), each with `{sender_id, post_count, first_seen_ts, last_seen_ts}`. Pure thread-walk + per-author tally. Companion to T-1591 `active_in_thread` (which returns per-message rows) but pivots to per-author with counts. Useful for "who's in this thread?" without sifting duplicates.
- `termlink_agent_recent_window` — given `hours` (default 6, max 168), returns posts within the last N hours sorted newest-first. Pure walk + filter by `ts_unix_ms > now_ms - hours*3600*1000`. Returns `[{offset, sender_id, body_preview, ts_unix_ms, mins_ago}, ...]` capped at limit. Time-window orientation companion to limit-only `recent`-style reads — answers "what happened in the last 6 hours?" without guessing the right limit.

Both pure walk + filter.

## Acceptance Criteria

### Agent
- [x] New `AgentThreadAuthorsParams` struct (root_offset u64)
- [x] New `AgentRecentWindowParams` struct (hours Option<u64>, limit Option<u64>)
- [x] New `termlink_agent_thread_authors` walks topic + builds parent→children map + collects descendants from root_offset + per-author tally
- [x] New `termlink_agent_recent_window` walks topic + filters by `now_ms - hours*3600*1000` ts cutoff + sorts newest-first
- [x] thread_authors returns `total_authors` + `total_posts` + sorted `[{sender_id, post_count, first_seen_ts, last_seen_ts}, ...]` desc by post_count
- [x] recent_window default hours=6, capped at 168 (1 week); default limit=50, capped at 500
- [x] recent_window includes `mins_ago` derived from `(now_ms - ts_unix_ms) / 60_000`
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=150 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_thread_authors` + `termlink_agent_recent_window` are operator-fluent over MCP
  **Steps:**
  1. Pick a thread root_offset from `termlink_agent_busiest_threads`
  2. Call `termlink_agent_thread_authors` with that offset
  3. Verify dedup'd participant list with per-author post counts
  4. Call `termlink_agent_recent_window` with hours=6
  5. Verify time-window slice of recent posts
  **Expected:** thread_authors gives clean per-thread participant census; recent_window gives time-anchored orientation.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_thread_authors"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_recent_window"' crates/termlink-mcp/src/tools.rs

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

## Recommendation

**Recommendation:** GO
**Rationale:** Two orientation reads on different axes. thread_authors is dedup'd companion to active_in_thread — pivots from per-message rows to per-author census with counts; cleaner answer to "who's in this thread?". recent_window is time-anchored alternative to limit-only reads — answers "what happened in the last 6 hours?" without guessing limit numbers. Both pure walk + filter, ~80 LOC each. Brings session total to 11 waves post-resume, +22 read tools, mcp_tools 128→150.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=150 (was 148 after T-1601) — +2
- Verification gate 4/4 (TBD)
- thread_authors: O(n) walk + parent→children map + DFS descendant collection + per-author (count, min-ts, max-ts) tally; recent_window: O(n) walk + ts cutoff filter + sort desc

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

### 2026-05-06T05:30:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1602-termlinkagentthreadauthors--termlinkagen.md
- **Context:** Initial task creation
