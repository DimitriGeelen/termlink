---
id: T-1580
name: "termlink_agent_emoji_stats + termlink_agent_ack_history — topic-wide reaction aggregator + per-sender ack timeline MCP read tools"
description: >
  termlink_agent_emoji_stats + termlink_agent_ack_history — topic-wide reaction aggregator + per-sender ack timeline MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T17:57:33Z
last_update: 2026-05-20T13:23:28Z
date_finished: 2026-05-05T18:06:06Z
---

# T-1580: termlink_agent_emoji_stats + termlink_agent_ack_history — topic-wide reaction aggregator + per-sender ack timeline MCP read tools

## Context

T-1579 capped the navigation/curation read suite at 104 tools. This wave adds two **analyst-style aggregators** — zoom-in and zoom-out companions to existing pin-point reads:

- `termlink_agent_emoji_stats` — topic-wide reaction aggregator. Walks `msg_type=reaction` envelopes, groups by emoji (payload), counts uses + tracks last_used_ts. Returns `[{emoji, count, last_used_ts}, ...]` sorted by count desc. Mirrors CLI T-1538. Zooms OUT from T-1571 `agent_reactions` (per-offset) — answers "what's resonating across the whole topic?".
- `termlink_agent_ack_history` — per-sender receipt timeline. Walks `msg_type=receipt` envelopes filtered to one sender_id, returns `[{up_to, ts_unix_ms}, ...]` sorted newest-first. Mirrors CLI T-1539 family. Zooms IN from T-1577 `agent_ack_status` (frontier-per-sender across whole topic) — answers "show me one sender's full ack timeline".

Both pure walk + filter + aggregate. Continue the established walk-loop pattern.

## Acceptance Criteria

### Agent
- [x] New `AgentEmojiStatsParams` struct (limit Option<u64>)
- [x] New `AgentAckHistoryParams` struct (sender_id Option<String>, limit Option<u64>)
- [x] New `termlink_agent_emoji_stats` tool method that walks topic + filters msg_type=reaction + groups by emoji
- [x] New `termlink_agent_ack_history` tool method that walks topic + filters msg_type=receipt + by sender_id
- [x] emoji_stats returns JSON array sorted count desc; ack_history returns JSON array sorted ts desc
- [x] ack_history defaults sender_id to caller's local Identity fingerprint
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=106 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_emoji_stats` + `termlink_agent_ack_history` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_emoji_stats`
  2. Compare with `target/release/termlink agent emoji-stats`
  3. Call `termlink_agent_ack_history`
  4. Compare with `target/release/termlink agent ack-history`
  **Expected:** MCP returns matching aggregates/timeline; CLI shows similar set.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_emoji_stats"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_ack_history"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two analyst-style aggregator reads — emoji_stats (zoom-out from per-offset reactions) + ack_history (zoom-in from per-sender frontier). They round out the engagement/acknowledgment read surface to support both bird's-eye and microscope views. emoji_stats answers "what's resonating?" (chat tone), ack_history answers "what's one sender's full ack timeline?" (audit). Both pure walk+filter+aggregate, ~70 LOC each, identical pattern to T-1577.
**Evidence:**
- Build clean (4m 59s)
- `termlink version --json` reports mcp_tools=106 (was 104 after T-1579) — +2
- Verification gate 4/4 passed
- emoji_stats: O(n) walk + HashMap-aggregate; ack_history: O(n) walk + filter + sort

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

### 2026-05-05T17:57:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1580-termlinkagentemojistats--termlinkagentac.md
- **Context:** Initial task creation

### 2026-05-05T18:06:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 9/9, build clean, mcp_tools=106. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).


### 2026-05-20T13:23:28Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_emoji_stats`, `termlink_agent_ack_history`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
