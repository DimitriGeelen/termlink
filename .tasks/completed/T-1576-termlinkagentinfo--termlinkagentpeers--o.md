---
id: T-1576
name: "termlink_agent_info + termlink_agent_peers — orientation MCP read tools (topic snapshot + participant directory)"
description: >
  termlink_agent_info + termlink_agent_peers — orientation MCP read tools (topic snapshot + participant directory)

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T16:54:05Z
last_update: 2026-05-20T13:23:26Z
date_finished: 2026-05-05T17:03:36Z
---

# T-1576: termlink_agent_info + termlink_agent_peers — orientation MCP read tools (topic snapshot + participant directory)

## Context

T-1571..T-1575 shipped 8 read tools covering navigation, curation, engagement, and search. This wave adds the **orientation** primitives — what an MCP-aware agent calls *first* when joining chat-arc:

- `termlink_agent_info` — single call returns topic snapshot: total envelope count, count-by-msg_type, unique senders count, last activity ts, latest topic description (from `topic_metadata` envelopes). Mirrors CLI T-1524 `agent info`.
- `termlink_agent_peers` — single call returns participant directory: each sender's post_count + last_post_ts, sorted by last_post_ts desc. Mirrors CLI T-1520 `agent peers`.

Both pure walk+aggregate. Establishes the "orientation read trio" (info → peers → recent) that mirrors how an operator first uses chat-arc.

## Acceptance Criteria

### Agent
- [x] New `AgentInfoParams` struct (empty)
- [x] New `AgentPeersParams` struct (limit Option<u64>)
- [x] New `termlink_agent_info` tool method that walks topic + aggregates summary
- [x] New `termlink_agent_peers` tool method that walks topic + groups by sender_id
- [x] info returns `{total, by_msg_type, unique_senders, last_activity_ts, description}`; peers returns `[{sender_id, post_count, last_post_ts}, ...]` sorted desc
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=98 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_info` + `termlink_agent_peers` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_info`
  2. Compare with `target/release/termlink agent info`
  3. Call `termlink_agent_peers`
  4. Compare with `target/release/termlink agent peers`
  **Expected:** MCP returns matching aggregates; CLI shows similar set.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*(9[8-9]|1[0-9][0-9])'
grep -q '"termlink_agent_info"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_peers"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two orientation-class read tools — info (single-call topic snapshot with by_msg_type breakdown + latest description) + peers (participant directory with post_count + last_post_ts). Together with `termlink_agent_recent`, they form the natural orientation triad: info ("what is this?") → peers ("who's here?") → recent ("what's happening?"). MCP-aware agents joining chat-arc for the first time can call these three tools in sequence to acclimate.
**Evidence:**
- Build clean (3m 59s)
- `termlink version --json` reports mcp_tools=98 (was 96 after T-1575) — +2
- Verification gate 4/4 passed
- info aggregates 5 dimensions in single walk; peers single sort+truncate

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

### 2026-05-05T16:54:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1576-termlinkagentinfo--termlinkagentpeers--o.md
- **Context:** Initial task creation

### 2026-05-05T17:03:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:26Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_info`, `termlink_agent_peers`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
