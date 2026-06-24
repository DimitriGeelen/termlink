---
id: T-1584
name: "termlink_agent_topic_metadata_history + termlink_agent_reactions_by — description audit log + per-sender reaction history MCP read tools"
description: >
  termlink_agent_topic_metadata_history + termlink_agent_reactions_by — description audit log + per-sender reaction history MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T18:30:13Z
last_update: 2026-05-20T13:23:31Z
date_finished: 2026-05-05T18:37:00Z
---

# T-1584: termlink_agent_topic_metadata_history + termlink_agent_reactions_by — description audit log + per-sender reaction history MCP read tools

## Context

T-1583 brought MCP read surface to 112 tools. This wave completes the per-sender / per-topic mirror by adding two **per-axis history reads**:

- `termlink_agent_topic_metadata_history` — chronological audit log of topic_metadata description changes. Walks topic, filters `msg_type=topic_metadata`, returns `[{description, sender_id, ts_unix_ms}, ...]` sorted oldest-first. Companion to T-1576 `agent_info` (current description only) — answers "how has this topic's description evolved?".
- `termlink_agent_reactions_by` — per-sender reaction history. Walks topic, filters `msg_type=reaction` by `sender_id` (default = caller's local Identity), returns `[{emoji, in_reply_to, ts_unix_ms}, ...]` sorted newest-first. Triangulates with T-1571 reactions (per-offset) and T-1580 emoji_stats (topic-wide) to give all three dimensions: by-target, by-emoji, by-sender.

Both pure walk + filter. Both ride the established walk-loop pattern.

## Acceptance Criteria

### Agent
- [x] New `AgentTopicMetadataHistoryParams` struct (limit Option<u64>)
- [x] New `AgentReactionsByParams` struct (sender_id Option<String>, limit Option<u64>)
- [x] New `termlink_agent_topic_metadata_history` tool method walks topic + filters topic_metadata + decodes payload
- [x] New `termlink_agent_reactions_by` tool method walks topic + filters reaction by sender_id
- [x] history sorted oldest-first (chronological); reactions_by sorted newest-first
- [x] reactions_by defaults sender_id to caller's local Identity
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=114 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_topic_metadata_history` + `termlink_agent_reactions_by` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_topic_metadata_history`
  2. Spot-check that the latest entry matches `termlink_agent_info`'s description field
  3. Call `termlink_agent_reactions_by` with default sender_id
  4. Verify it returns the caller's reactions on chat-arc
  **Expected:** history shows full description audit; reactions_by shows local agent's emoji history.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_topic_metadata_history"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_reactions_by"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two per-axis history reads completing the reaction-data triangulation. topic_metadata_history audits how the topic description has evolved (chronological log, base64-decoded). reactions_by(sender_id) gives the third orthogonal view of reaction data — joining reactions(offset)=by-target and emoji_stats=by-emoji to form the full {by-target, by-emoji, by-sender} cube. Both pure walk + filter, ~70 LOC each. Brings MCP read surface to 114 tools — 5 waves (T-1580..T-1584) shipped this session for +10 read tools.
**Evidence:**
- Build clean (4m 59s)
- `termlink version --json` reports mcp_tools=114 (was 112 after T-1583) — +2
- Verification gate 4/4 passed
- topic_metadata_history: O(n) walk + base64 decode; reactions_by: O(n) walk + dual filter

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

### 2026-05-05T18:30:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1584-termlinkagenttopicmetadatahistory--terml.md
- **Context:** Initial task creation

### 2026-05-05T18:37:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 9/9, build clean, mcp_tools=114. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).


### 2026-05-20T13:23:31Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_topic_metadata_history`, `termlink_agent_reactions_by`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
