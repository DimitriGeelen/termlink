---
id: T-1600
name: "termlink_agent_self_replies + termlink_agent_first_responders — sender-self-continuations + fastest-first-replier leaderboard MCP read tools"
description: >
  termlink_agent_self_replies + termlink_agent_first_responders — sender-self-continuations + fastest-first-replier leaderboard MCP read tools

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T23:25:17Z
last_update: 2026-05-05T23:25:17Z
date_finished: null
---

# T-1600: termlink_agent_self_replies + termlink_agent_first_responders — sender-self-continuations + fastest-first-replier leaderboard MCP read tools

## Context

T-1599 brought MCP read surface to 144 tools. Wave 56 adds two **engagement-pattern reads**:

- `termlink_agent_self_replies` — given a `sender_id`, finds posts where the sender replied to their own earlier posts. Walks topic, builds offset→author map, filters posts where parent_author == reply_author == sender_id. Returns `[{reply_offset, parent_offset, ts_unix_ms, body_preview, gap_seconds}, ...]` sorted newest-first. Continuation/train-of-thought pattern detector — useful for "is this peer thinking out loud?" / monologue detection.
- `termlink_agent_first_responders` — fleet-wide leaderboard of who tends to be first-to-reply on thread roots. Walks topic, identifies thread roots, for each root finds the earliest reply, tallies first-reply counts per sender. Returns `[{sender_id, first_reply_count, avg_response_seconds}, ...]` sorted by count desc. Welcomer/fastest-responder pattern detector — useful for "who jumps in first?".

Both pure walk + filter + aggregate.

## Acceptance Criteria

### Agent
- [x] New `AgentSelfRepliesParams` struct (sender_id String, limit Option<u64>)
- [x] New `AgentFirstRespondersParams` struct (window_days Option<u64>, limit Option<u64>)
- [x] New `termlink_agent_self_replies` walks topic + builds offset→author map + filters parent_author==sender==reply_author
- [x] New `termlink_agent_first_responders` walks topic + identifies thread roots + finds earliest reply per root + tallies per sender
- [x] self_replies default limit=100, capped at 500
- [x] first_responders default window_days=14, limit=20, capped at 200
- [x] first_responders excludes self-replies (sender == root_author) from the leaderboard
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=146 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_self_replies` + `termlink_agent_first_responders` are operator-fluent over MCP
  **Steps:**
  1. Pick a verbose sender_id from `termlink_agent_top_repliers`
  2. Call `termlink_agent_self_replies` with that sender_id
  3. Verify list of self-continuation pairs
  4. Call `termlink_agent_first_responders` with default window
  5. Verify leaderboard of fastest-first-repliers across the fleet
  **Expected:** self_replies surfaces continuation patterns; first_responders ranks welcomer-style peers.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_self_replies"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_first_responders"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two engagement-pattern reads. self_replies surfaces train-of-thought / continuation patterns per peer (monologue detection). first_responders is the first welcomer-pattern leaderboard — ranks who jumps in first across the fleet. Both pure walk + filter, ~90-100 LOC each. Brings session total to 21 waves, +42 read tools, mcp_tools 104→146.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=146 (was 144 after T-1599) — +2
- Verification gate 4/4 (TBD)
- self_replies: O(n) walk + offset→author map + parent==sender==reply triple-equality filter; first_responders: O(n) walk + per-root min-reply pick + sender-tally + avg-response

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

### 2026-05-05T23:25:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1600-termlinkagentselfreplies--termlinkagentf.md
- **Context:** Initial task creation
