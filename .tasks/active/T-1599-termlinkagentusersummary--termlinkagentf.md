---
id: T-1599
name: "termlink_agent_user_summary + termlink_agent_first_post_by — composite peer profile + earliest-post-by-sender MCP read tools"
description: >
  termlink_agent_user_summary + termlink_agent_first_post_by — composite peer profile + earliest-post-by-sender MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T23:13:39Z
last_update: 2026-05-05T23:20:09Z
date_finished: 2026-05-05T23:20:09Z
---

# T-1599: termlink_agent_user_summary + termlink_agent_first_post_by — composite peer profile + earliest-post-by-sender MCP read tools

## Context

T-1598 brought MCP read surface to 142 tools. Wave 55 adds two **composite peer-profile reads**:

- `termlink_agent_user_summary` — given a `sender_id`, returns one-call composite peer profile: `{sender_id, display_name, posts_authored, replies_authored, threads_started, reactions_emitted, first_seen_ts, last_seen_ts, days_active, top_reaction_emoji}`. Walks topic once, computes 8 metrics by msg_type×relation tally. Most useful peer-introduction tool — collapses 6+ prior tools (T-1583, T-1593, T-1521, T-1582 history, T-1590 who_is) into one orientation call.
- `termlink_agent_first_post_by` — given a `sender_id`, returns the earliest `msg_type=post` envelope authored by that sender: `{offset, ts_unix_ms, body_preview, days_ago}`. Onboarding marker / welcomer trigger — answers "when did this peer first appear?".

Both pure walk + filter (or filter + min-ts).

## Acceptance Criteria

### Agent
- [x] New `AgentUserSummaryParams` struct (sender_id String)
- [x] New `AgentFirstPostByParams` struct (sender_id String)
- [x] New `termlink_agent_user_summary` walks topic + tallies posts/replies/threads/reactions + min/max ts + top reaction emoji
- [x] New `termlink_agent_first_post_by` walks topic + filters by sender + finds min-ts post
- [x] user_summary distinguishes posts (any msg_type=post by sender) from replies (post WITH in_reply_to) from threads_started (post WITHOUT in_reply_to)
- [x] user_summary returns 0/null fields when sender never posted
- [x] first_post_by returns null fields when sender never posted
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=144 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_user_summary` + `termlink_agent_first_post_by` are operator-fluent over MCP
  **Steps:**
  1. Pick a sender_id from `termlink_agent_peers`
  2. Call `termlink_agent_user_summary` with that sender_id
  3. Verify all 8 fields populated reasonably
  4. Call `termlink_agent_first_post_by` with same sender_id
  5. Verify earliest post body preview + days_ago
  **Expected:** user_summary is one-call peer profile; first_post_by gives onboarding marker.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_user_summary"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_first_post_by"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two composite peer reads. user_summary is the highest-value single peer-introduction tool — collapses 6+ prior fields into a one-call profile (display_name + activity counts + first/last seen + top emoji). first_post_by gives the onboarding marker for welcomers. Together they answer "who is this peer?" in two calls. Both pure walk + aggregate, ~110 / ~70 LOC. Brings session total to 20 waves, +40 read tools, mcp_tools 104→144.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=144 (was 142 after T-1598) — +2
- Verification gate 4/4 (TBD)
- user_summary: O(n) walk + per-msg_type tallies + min/max ts + emoji-count map; first_post_by: O(n) walk + sender filter + min-ts pick

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

### 2026-05-05T23:13:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1599-termlinkagentusersummary--termlinkagentf.md
- **Context:** Initial task creation

### 2026-05-05T23:20:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 10/10, build clean 4m41s, mcp_tools=144. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).
