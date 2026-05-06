---
id: T-1594
name: "termlink_agent_search_thread + termlink_agent_unanswered — per-thread content search + zero-reply detection MCP read tools"
description: >
  termlink_agent_search_thread + termlink_agent_unanswered — per-thread content search + zero-reply detection MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T22:13:08Z
last_update: 2026-05-05T22:20:18Z
date_finished: 2026-05-05T22:20:18Z
---

# T-1594: termlink_agent_search_thread + termlink_agent_unanswered — per-thread content search + zero-reply detection MCP read tools

## Context

T-1593 brought MCP read surface to 132 tools. Wave 50 adds two **scoped/anti-leaderboard reads**:

- `termlink_agent_search_thread` — substring search SCOPED to a single thread (`root_offset` + `query`). Walks topic, expands descendants from root via parent→children map (T-1588 pattern), then base64-decodes payloads and matches `query` (case-insensitive). Returns `[{offset, sender_id, body_preview, ts_unix_ms}, ...]` sorted newest-first. Companion to T-1572 `agent_search` (topic-wide) and T-1587 `search_by` (per-sender). New axis: per-thread content search.
- `termlink_agent_unanswered` — posts with zero replies. Walks topic in window, finds `msg_type=post` envelopes whose offset never appears as `metadata.in_reply_to` of any later post. Returns `[{offset, sender_id, body_preview, ts_unix_ms, age_hours}, ...]` sorted oldest-first (longest unanswered surface first). Anti-leaderboard companion to T-1588 `silent_senders` — surfaces re-engagement candidates and dropped conversation threads.

Both pure walk + filter (or filter + reply-set negation).

## Acceptance Criteria

### Agent
- [x] New `AgentSearchThreadParams` struct (root_offset u64, query String, limit Option<u64>)
- [x] New `AgentUnansweredParams` struct (window_days Option<u64>, limit Option<u64>)
- [x] New `termlink_agent_search_thread` walks topic + expands descendants from root + base64-decodes payload + case-insensitive substring match
- [x] New `termlink_agent_unanswered` walks topic + builds reply-target set + filters posts not in set + window-cutoff
- [x] search_thread default limit=100, capped at 500
- [x] unanswered default window_days=14, limit=50
- [x] unanswered excludes reactions/redactions/topic_metadata (only msg_type=post)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=134 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_search_thread` + `termlink_agent_unanswered` are operator-fluent over MCP
  **Steps:**
  1. Pick a thread root from `termlink_agent_busiest_threads`
  2. Call `termlink_agent_search_thread` with that root_offset + a known substring
  3. Verify only matches inside that thread are returned
  4. Call `termlink_agent_unanswered` with default window
  5. Verify list of zero-reply posts (re-engagement candidates)
  **Expected:** search_thread is scoped to thread; unanswered surfaces conversation gaps.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_search_thread"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_unanswered"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two reads on different axes. search_thread completes the search axis cube ({topic-wide, per-sender, per-thread}). unanswered is an anti-leaderboard companion to silent_senders — answers "what posts dropped on the floor?". Both pure walk + filter, ~100 LOC each. Brings session total to 15 waves, +30 read tools, mcp_tools 104→134.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=134 (was 132 after T-1593) — +2
- Verification gate 4/4 (TBD)
- search_thread: O(n) walk + recursive descendant set + base64-decode + substring match; unanswered: O(n) walk + reply-target HashSet + post filter + window cutoff

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

### 2026-05-05T22:13:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1594-termlinkagentsearchthread--termlinkagent.md
- **Context:** Initial task creation

### 2026-05-05T22:20:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 10/10, build clean 4m47s, mcp_tools=134. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).
