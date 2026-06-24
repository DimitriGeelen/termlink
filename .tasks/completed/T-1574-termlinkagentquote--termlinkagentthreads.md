---
id: T-1574
name: "termlink_agent_quote + termlink_agent_threads — single-offset fetch + thread-root listing MCP read tools"
description: >
  termlink_agent_quote + termlink_agent_threads — single-offset fetch + thread-root listing MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T16:32:15Z
last_update: 2026-05-20T13:23:25Z
date_finished: 2026-05-05T16:41:04Z
---

# T-1574: termlink_agent_quote + termlink_agent_threads — single-offset fetch + thread-root listing MCP read tools

## Context

T-1573 shipped reduced-state read tools (pinned/starred). This wave layers two **navigation** read tools — both essential for MCP-aware agents to traverse a growing chat-arc:

- `termlink_agent_quote` — fetch a single envelope by its offset. Walks via channel.subscribe until the offset is found, returns the raw envelope. Mirrors CLI T-1505 `agent quote <offset>`. Lets agents resolve a referenced offset (e.g. when reacting/replying/pinning) without dumping the full topic.
- `termlink_agent_threads` — list all offsets that have been replied to (i.e. appear as some envelope's `metadata.in_reply_to`). Returns `{root_offset, reply_count, last_reply_ts}` sorted by last_reply_ts desc. Mirrors CLI T-1533 `agent threads`. Surfaces conversation hot-spots — what's being discussed.

Pairs cleanly with `termlink_agent_reply` (T-1563) and the existing read primitives. Bundled per Wave 29 precedent — both pure walk+aggregate.

## Acceptance Criteria

### Agent
- [x] New `AgentQuoteParams` struct (offset u64)
- [x] New `AgentThreadsParams` struct (limit Option<u64>)
- [x] New `termlink_agent_quote` tool method that walks topic + returns the envelope whose offset matches
- [x] New `termlink_agent_threads` tool method that walks topic + collects parents-by-reply-count
- [x] Quote returns single JSON object (or error if not found); threads returns JSON array sorted last_reply_ts desc
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=94 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_quote` + `termlink_agent_threads` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_quote` with a known offset
  2. Compare with `target/release/termlink agent quote <N>`
  3. Call `termlink_agent_threads`
  4. Compare with `target/release/termlink agent threads`
  **Expected:** MCP returns matching envelope/threads; CLI shows similar set.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*(9[4-9]|1[0-9][0-9])'
grep -q '"termlink_agent_quote"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_threads"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two navigation-class read tools — quote (single-offset fetch) + threads (parents-by-reply-count). Together they let MCP-aware agents traverse chat-arc without raw topic dumps: quote resolves a referenced offset, threads surfaces conversation hot-spots. Pure walk+aggregate, no new wire shapes. Builds the foundation for richer read tools (on_thread, ancestors, reply_subtree).
**Evidence:**
- Build clean (4m 04s)
- `termlink version --json` reports mcp_tools=94 (was 92 after T-1573) — +2
- Verification gate 4/4 passed
- Both tools <60 LOC each — pattern continues to compress as the topic-walk loop becomes idiomatic

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

### 2026-05-05T16:32:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1574-termlinkagentquote--termlinkagentthreads.md
- **Context:** Initial task creation

### 2026-05-05T16:41:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:25Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_quote`, `termlink_agent_threads`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
