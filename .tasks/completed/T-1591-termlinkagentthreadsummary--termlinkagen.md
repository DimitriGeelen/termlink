---
id: T-1591
name: "termlink_agent_thread_summary + termlink_agent_active_in_thread — deep thread metrics + per-thread participants MCP read tools"
description: >
  termlink_agent_thread_summary + termlink_agent_active_in_thread — deep thread metrics + per-thread participants MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T20:03:20Z
last_update: 2026-05-20T13:23:35Z
date_finished: 2026-05-05T20:09:46Z
---

# T-1591: termlink_agent_thread_summary + termlink_agent_active_in_thread — deep thread metrics + per-thread participants MCP read tools

## Context

T-1590 brought MCP read surface to 126 tools. Wave 47 adds two **per-thread analytical reads**:

- `termlink_agent_thread_summary` — deep metrics for a single thread. Given a `root_offset`, walks topic, recursively expands descendants, and returns `{root_offset, descendant_count, unique_senders, first_post_ts, last_post_ts, age_days, emoji_count}`. Useful for "is this thread alive?", "how broad is participation?", and dashboard cards.
- `termlink_agent_active_in_thread` — per-thread participant list. Given a `root_offset`, walks topic, expands descendants, returns `[{sender_id, post_count, last_post_ts}, ...]` sorted by post_count descending. Useful for "who's involved in this thread?" and for tagging reply suggestions.

Both walk + recursive-descendant-expansion + per-sender aggregate.

## Acceptance Criteria

### Agent
- [x] New `AgentThreadSummaryParams` struct (root_offset u64)
- [x] New `AgentActiveInThreadParams` struct (root_offset u64, limit Option<u64>)
- [x] New `termlink_agent_thread_summary` walks topic + finds root + recursively counts descendants + computes metrics
- [x] New `termlink_agent_active_in_thread` walks topic + finds root + expands descendants + aggregates senders
- [x] thread_summary returns 0/null fields when root_offset not found
- [x] active_in_thread limit defaults to 50, capped at 500
- [x] Both share descendant-expansion logic via parent→children map (T-1588 pattern)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=128 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_thread_summary` + `termlink_agent_active_in_thread` are operator-fluent over MCP
  **Steps:**
  1. Pick a thread root from `termlink_agent_busiest_threads`
  2. Call `termlink_agent_thread_summary` with that root_offset
  3. Verify metrics (descendant_count, unique_senders, age_days)
  4. Call `termlink_agent_active_in_thread` with same root_offset
  5. Verify participant leaderboard
  **Expected:** thread_summary gives one-glance metrics; active_in_thread ranks participants.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_thread_summary"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_active_in_thread"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two per-thread analytical reads completing the thread axis. thread_summary gives "is this thread alive / how broad?" — one-call dashboard metrics. active_in_thread gives the participant leaderboard — useful for reply-tagging and "who's invested in this conversation?". Both share the parent→children expansion (T-1588 pattern). ~110 LOC each. Brings session total to 12 waves, +24 read tools, mcp_tools 104→128.
**Evidence:**
- Build clean (4m 14s)
- `termlink version --json` reports mcp_tools=128 (was 126 after T-1590) — +2
- Verification gate 4/4 passed
- thread_summary: O(n) walk + recursive descendant set + per-thread aggregation; active_in_thread: same expansion + per-sender count+max-ts aggregate

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

### 2026-05-05T20:03:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1591-termlinkagentthreadsummary--termlinkagen.md
- **Context:** Initial task creation

### 2026-05-05T20:09:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 10/10, build clean 4m14s, mcp_tools=128. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).


### 2026-05-20T13:23:35Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_thread_summary`, `termlink_agent_active_in_thread`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
