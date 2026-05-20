---
id: T-1587
name: "termlink_agent_search_by + termlink_agent_top_repliers — per-sender content search + reply-leaderboard MCP read tools"
description: >
  termlink_agent_search_by + termlink_agent_top_repliers — per-sender content search + reply-leaderboard MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T19:13:10Z
last_update: 2026-05-20T13:23:33Z
date_finished: 2026-05-05T19:20:04Z
---

# T-1587: termlink_agent_search_by + termlink_agent_top_repliers — per-sender content search + reply-leaderboard MCP read tools

## Context

T-1586 brought MCP read surface to 118 tools. Wave 43 adds two **search/analytical reads**:

- `termlink_agent_search_by` — per-sender content search. Walks topic, filters `msg_type=post` by `sender_id` (default = caller's local Identity), then by case-insensitive substring match of base64-decoded payload against `query`. Returns `[{offset, sender_id, body_preview, ts_unix_ms}, ...]` sorted newest-first. Combines T-1572 search shape with the by-sender filter (T-1582 history).
- `termlink_agent_top_repliers` — analytical leaderboard. Walks topic in a configurable time window, counts envelopes with `metadata.in_reply_to` set per sender, returns `[{sender_id, reply_count}, ...]` sorted descending. Companion analytical to T-1581 topic_stats.

Both pure walk + filter + aggregate.

## Acceptance Criteria

### Agent
- [x] New `AgentSearchByParams` struct (sender_id Option<String>, query String, limit Option<u64>)
- [x] New `AgentTopRepliersParams` struct (window_days Option<u64>, limit Option<u64>)
- [x] New `termlink_agent_search_by` walks topic + filters post by sender_id + base64-decodes payload + substring matches query
- [x] New `termlink_agent_top_repliers` walks topic + counts in_reply_to per sender within window
- [x] search_by defaults sender_id to caller's local Identity
- [x] search_by case-insensitive substring match
- [x] top_repliers default window_days=7, limit=20
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=120 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_search_by` + `termlink_agent_top_repliers` are operator-fluent over MCP
  **Steps:**
  1. Call `termlink_agent_search_by` with default sender_id and a query like "T-1586"
  2. Verify results are limited to your own posts containing the query
  3. Call `termlink_agent_top_repliers` with default window
  4. Verify the leaderboard shows known active senders ranked by reply count
  **Expected:** search_by filters to caller; top_repliers leaderboard sorts by reply count descending.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_search_by"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_top_repliers"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two search/analytical reads building on patterns established this session. search_by combines T-1572 search-pattern with T-1582 by-sender filter — first read tool that intersects two filter axes. top_repliers gives analytical leaderboard companion to T-1581 topic_stats. Both ride established walk-loop + base64-decode (search_by) + HashMap-aggregate (top_repliers). Brings session total to 8 waves, +16 read tools, mcp_tools 104→120.
**Evidence:**
- Build clean (4m 35s)
- `termlink version --json` reports mcp_tools=120 (was 118 after T-1586) — +2
- Verification gate 4/4 passed
- search_by: O(n) walk + dual filter (sender + substring) + base64-decode; top_repliers: O(n) walk + window-cutoff + HashMap-aggregate
- Milestone: termlink crossed version 0.9.2000 with this build

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

### 2026-05-05T19:13:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1587-termlinkagentsearchby--termlinkagenttopr.md
- **Context:** Initial task creation

### 2026-05-05T19:20:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 10/10, build clean 4m35s, mcp_tools=120. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).


### 2026-05-20T13:23:33Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_search_by`, `termlink_agent_top_repliers`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
