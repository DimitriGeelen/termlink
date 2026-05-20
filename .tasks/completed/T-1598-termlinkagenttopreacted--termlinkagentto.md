---
id: T-1598
name: "termlink_agent_top_reacted + termlink_agent_top_quoted — most-reacted-to + most-forwarded posts MCP read tools"
description: >
  termlink_agent_top_reacted + termlink_agent_top_quoted — most-reacted-to + most-forwarded posts MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T23:01:46Z
last_update: 2026-05-20T13:23:39Z
date_finished: 2026-05-05T23:08:42Z
---

# T-1598: termlink_agent_top_reacted + termlink_agent_top_quoted — most-reacted-to + most-forwarded posts MCP read tools

## Context

T-1597 brought MCP read surface to 140 tools. Wave 54 adds two **per-post attention leaderboards**:

- `termlink_agent_top_reacted` — most-reacted-to posts (topic-wide reaction-count leaderboard). Walks topic, counts per-offset reaction envelopes, returns `[{offset, sender_id, body_preview, ts_unix_ms, reaction_count}, ...]` sorted by reaction_count desc. Window-scoped. Companion to T-1580 emoji_stats (topic-wide aggregate) and T-1592 reaction_summary (single-offset breakdown) — fills "which posts attracted the most reactions?" gap.
- `termlink_agent_top_replied` — most-replied-to posts (per-post reply-count leaderboard). Walks topic, counts in_reply_to references per offset, returns `[{offset, sender_id, body_preview, ts_unix_ms, reply_count}, ...]` sorted by reply_count desc. Per-post counterpart to T-1589 busiest_threads (which counts ALL descendants, not just direct replies). Fills "which single posts drew the most direct replies?" gap.

Both pure walk + per-offset tally + leaderboard sort.

## Acceptance Criteria

### Agent
- [x] New `AgentTopReactedParams` struct (window_days Option<u64>, limit Option<u64>)
- [x] New `AgentTopRepliedParams` struct (window_days Option<u64>, limit Option<u64>)
- [x] New `termlink_agent_top_reacted` walks topic + tallies reactions per parent offset + sorts desc
- [x] New `termlink_agent_top_replied` walks topic + tallies direct replies per parent offset + sorts desc
- [x] Both default window_days=14, limit=20, capped at 200
- [x] Both window-cutoff applied to PARENT post ts (not reaction/reply ts) so attention is about the post age
- [x] Both base64-decode body_preview (160 chars max)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=142 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_top_reacted` + `termlink_agent_top_replied` are operator-fluent over MCP
  **Steps:**
  1. Call `termlink_agent_top_reacted` with default window
  2. Verify ranked list of most-reacted posts looks plausible
  3. Call `termlink_agent_top_replied`
  4. Verify ranked list of most-replied posts (per-post not per-thread)
  **Expected:** top_reacted + top_replied surface the highest-attention posts on different attention axes.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_top_reacted"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_top_replied"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two per-post attention leaderboards. top_reacted answers "which posts attracted the most reactions?" — distinct from emoji_stats (topic-wide totals) and reaction_summary (single offset). top_replied is per-post reply-count leaderboard — distinct from busiest_threads (descendant total per ROOT). Together they expose the {reactions, replies} attention cube at the per-post granularity. Both pure walk + tally + sort, ~80 LOC each. Brings session total to 19 waves, +38 read tools, mcp_tools 104→142.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=142 (was 140 after T-1597) — +2
- Verification gate 4/4 (TBD)
- top_reacted: O(n) walk + per-parent reaction tally + sort desc + window-cutoff on parent ts; top_replied: same with reply-count instead of reaction-count

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

### 2026-05-05T23:01:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1598-termlinkagenttopreacted--termlinkagentto.md
- **Context:** Initial task creation

### 2026-05-05T23:08:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 10/10, build clean 4m49s, mcp_tools=142. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).


### 2026-05-20T13:23:39Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_top_reacted`, `termlink_agent_top_replied`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
