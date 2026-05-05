---
id: T-1589
name: "termlink_agent_busiest_threads + termlink_agent_recent_decisions — top-thread leaderboard + decision-bearing post heuristic search MCP read tools"
description: >
  termlink_agent_busiest_threads + termlink_agent_recent_decisions — top-thread leaderboard + decision-bearing post heuristic search MCP read tools

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T19:39:28Z
last_update: 2026-05-05T19:39:28Z
date_finished: null
---

# T-1589: termlink_agent_busiest_threads + termlink_agent_recent_decisions — top-thread leaderboard + decision-bearing post heuristic search MCP read tools

## Context

T-1588 brought MCP read surface to 122 tools. Wave 45 adds two **analytical/heuristic reads**:

- `termlink_agent_busiest_threads` — top-N threads by descendant count in window. Walks topic, identifies thread roots (post envelopes with no `metadata.in_reply_to`) created within window, counts each root's descendants, returns `[{root_offset, body_preview, sender_id, ts_unix_ms, descendant_count}, ...]` sorted by descendant_count descending. Companion to T-1574 `agent_threads` (lists all roots) and T-1581 `topic_stats` (aggregate counts).
- `termlink_agent_recent_decisions` — heuristic search for decision-bearing posts. Walks topic in window, filters `msg_type=post` with payload matching common decision markers (case-insensitive: "GO:", "NO-GO:", "DECISION:", "DECIDED:", "RECOMMEND:", "RECOMMENDATION:", "VERDICT:"). Returns `[{offset, sender_id, body_preview, marker, ts_unix_ms}, ...]` sorted newest-first. Useful for chat-arc forensics — "what decisions were made this week?".

Both pure walk + filter + aggregate.

## Acceptance Criteria

### Agent
- [x] New `AgentBusiestThreadsParams` struct (window_days Option<u64>, limit Option<u64>)
- [x] New `AgentRecentDecisionsParams` struct (window_days Option<u64>, limit Option<u64>)
- [x] New `termlink_agent_busiest_threads` walks topic + identifies root posts in window + counts descendants
- [x] New `termlink_agent_recent_decisions` walks topic + base64-decodes payload + matches decision markers
- [x] busiest_threads default window_days=7, limit=20
- [x] recent_decisions default window_days=14, limit=50
- [x] recent_decisions matches at least 7 markers ("GO:", "NO-GO:", "DECISION:", "DECIDED:", "RECOMMEND:", "RECOMMENDATION:", "VERDICT:")
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=124 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_busiest_threads` + `termlink_agent_recent_decisions` are operator-fluent over MCP
  **Steps:**
  1. Call `termlink_agent_busiest_threads` with default window
  2. Spot-check the leaderboard against `termlink_agent_threads` (T-1574)
  3. Call `termlink_agent_recent_decisions`
  4. Verify hits include known recommendation/decision posts
  **Expected:** busiest_threads ranks roots by descendant count; recent_decisions catches decision-marker posts.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_busiest_threads"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_recent_decisions"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two analytical/heuristic reads completing the chat-arc forensics surface. busiest_threads gives "what's hot right now?" (top-N by descendant count) — companion to T-1574 + T-1581 + T-1587 top_repliers. recent_decisions is the first heuristic content tool — substring match on common decision markers. Both pure walk + aggregate, ~110 LOC each. Brings session total to 10 waves, +20 read tools, mcp_tools 104→124.
**Evidence:**
- Build clean (4m 16s)
- `termlink version --json` reports mcp_tools=124 (was 122 after T-1588) — +2
- Verification gate 4/4 passed
- busiest_threads: O(n) walk + recursive descendant counter + window-cutoff; recent_decisions: O(n) walk + base64-decode + 7-marker substring match

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

### 2026-05-05T19:39:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1589-termlinkagentbusiestthreads--termlinkage.md
- **Context:** Initial task creation
