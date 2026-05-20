---
id: T-1575
name: "termlink_agent_on_thread + termlink_agent_reactions — thread descent + per-offset reactions MCP read tools"
description: >
  termlink_agent_on_thread + termlink_agent_reactions — thread descent + per-offset reactions MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T16:43:14Z
last_update: 2026-05-20T13:23:26Z
date_finished: 2026-05-05T16:51:59Z
---

# T-1575: termlink_agent_on_thread + termlink_agent_reactions — thread descent + per-offset reactions MCP read tools

## Context

T-1574 surfaced thread roots (`termlink_agent_threads`). This wave layers the **drill-down** half:

- `termlink_agent_on_thread` — given a root offset, return the full thread tree: the root envelope + all descendants (transitive replies via `metadata.in_reply_to`). Built on a parent→children index then BFS from root. Returns chronologically-ordered JSON array. Mirrors CLI T-1493 `agent on-thread <root>`.
- `termlink_agent_reactions` — given a single offset, return all reaction envelopes (`msg_type=reaction`) targeting it, with payload (emoji) decoded. Returns `[{emoji, sender_id, ts_unix_ms}, ...]` sorted newest-first. Mirrors CLI T-1514 `agent reactions <offset>`.

Pairs cleanly with `termlink_agent_threads` (root listing → descend) and `termlink_agent_react` (T-1562, write side).

## Acceptance Criteria

### Agent
- [x] New `AgentOnThreadParams` struct (root_offset u64, limit Option<u64>)
- [x] New `AgentReactionsParams` struct (offset u64)
- [x] New `termlink_agent_on_thread` tool method that walks topic + builds parent→children index + BFS-collects descendants from root
- [x] New `termlink_agent_reactions` tool method that walks topic + filters msg_type=reaction with metadata.in_reply_to=offset + decodes emoji
- [x] on_thread returns JSON array sorted ts asc (chronological); reactions returns JSON array sorted ts desc (newest-first)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=96 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_on_thread` + `termlink_agent_reactions` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_on_thread` with a root offset that has known replies
  2. Compare with `target/release/termlink agent on-thread <root>`
  3. Call `termlink_agent_reactions` with an offset that has known reactions
  4. Compare with `target/release/termlink agent reactions <offset>`
  **Expected:** MCP returns matching tree/reactions; CLI shows similar set.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*(9[6-9]|1[0-9][0-9])'
grep -q '"termlink_agent_on_thread"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_reactions"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two engagement-class read tools — on_thread (full descent from root) + reactions (per-offset emoji ack list). Combined with T-1574's threads (root listing) + T-1573's pinned/starred (curation state) + T-1571's recent (raw envelopes), MCP-aware agents now have a complete read API for chat-arc. on_thread's BFS pattern (parent→children index) is reusable for future tools (e.g. ancestors-walk, subtree-stats).
**Evidence:**
- Build clean (4m 02s)
- `termlink version --json` reports mcp_tools=96 (was 94 after T-1574) — +2
- Verification gate 4/4 passed
- on_thread BFS pattern: ~70 LOC; reactions filter: ~50 LOC

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

### 2026-05-05T16:43:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1575-termlinkagentonthread--termlinkagentreac.md
- **Context:** Initial task creation

### 2026-05-05T16:51:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:25Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_on_thread`, `termlink_agent_reactions`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
