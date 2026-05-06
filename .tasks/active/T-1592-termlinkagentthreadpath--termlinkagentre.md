---
id: T-1592
name: "termlink_agent_thread_path + termlink_agent_reaction_summary — full-conversation walk + per-offset emoji breakdown MCP read tools"
description: >
  termlink_agent_thread_path + termlink_agent_reaction_summary — full-conversation walk + per-offset emoji breakdown MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T21:39:16Z
last_update: 2026-05-05T21:47:08Z
date_finished: 2026-05-05T21:47:08Z
---

# T-1592: termlink_agent_thread_path + termlink_agent_reaction_summary — full-conversation walk + per-offset emoji breakdown MCP read tools

## Context

T-1591 brought MCP read surface to 128 tools. Wave 48 adds two **conversation-completion reads**:

- `termlink_agent_thread_path` — given any offset, returns the FULL conversation containing it: walks UP via `metadata.in_reply_to` to root, then DOWN via descendant expansion (T-1588 pattern), merging both into a single chronologically-sorted list. Combines T-1510 (ancestors) + T-1575 (on_thread/descendants) into one call. Useful for "show me the entire conversation around offset X" — common chat-arc reading flow.
- `termlink_agent_reaction_summary` — given an offset, returns `{offset, by_emoji: [{emoji, count, senders[]}], total_count}`. Per-offset emoji breakdown with sender attribution. Companion to T-1575 `agent_reactions` (raw list) and T-1580 `agent_emoji_stats` (topic-wide aggregate) — fills the per-post aggregate gap.

Both pure walk + filter + aggregate.

## Acceptance Criteria

### Agent
- [x] New `AgentThreadPathParams` struct (offset u64, limit Option<u64>)
- [x] New `AgentReactionSummaryParams` struct (offset u64)
- [x] New `termlink_agent_thread_path` walks topic + finds offset + walks ancestors UP + collects descendants DOWN + merges + sorts by ts
- [x] New `termlink_agent_reaction_summary` walks topic + filters in_reply_to=offset + msg_type=post + groups by emoji payload + tallies senders
- [x] thread_path returns empty when offset not found
- [x] reaction_summary returns total_count=0 + empty by_emoji when no reactions
- [x] Both share descendant-expansion logic via parent→children map (T-1588 pattern)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=130 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_thread_path` + `termlink_agent_reaction_summary` are operator-fluent over MCP
  **Steps:**
  1. Pick a busy offset from `termlink_agent_busiest_threads`
  2. Call `termlink_agent_thread_path` with a mid-thread offset
  3. Verify the full conversation (root + ancestors + descendants) is returned chronologically
  4. Pick an offset known to have reactions
  5. Call `termlink_agent_reaction_summary` with that offset
  6. Verify per-emoji counts + sender attribution
  **Expected:** thread_path gives full conversation context; reaction_summary aggregates emoji per post.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_thread_path"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_reaction_summary"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two conversation-completion reads. thread_path merges T-1510 ancestors + T-1575 descendants into a single full-conversation call — the most natural "show me everything about this offset" flow. reaction_summary fills the per-post aggregate gap between T-1575 (raw reactions list) and T-1580 (topic-wide stats). Both pure walk + filter + aggregate, ~110 LOC each. Brings session total to 13 waves, +26 read tools, mcp_tools 104→130.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=130 (was 128 after T-1591) — +2
- Verification gate 4/4 (TBD)
- thread_path: O(n) walk + ancestor chain + recursive descendant set + ts-sort merge; reaction_summary: O(n) walk + reply-filter + emoji-payload group + sender-set tally

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

### 2026-05-05T21:39:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1592-termlinkagentthreadpath--termlinkagentre.md
- **Context:** Initial task creation

### 2026-05-05T21:47:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 10/10, build clean 5m10s, mcp_tools=130. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).
