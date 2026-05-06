---
id: T-1579
name: "termlink_agent_ancestors + termlink_agent_pin_history — reply-chain walk + curation event log MCP read tools"
description: >
  termlink_agent_ancestors + termlink_agent_pin_history — reply-chain walk + curation event log MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T17:28:26Z
last_update: 2026-05-05T17:34:24Z
date_finished: 2026-05-05T17:34:24Z
---

# T-1579: termlink_agent_ancestors + termlink_agent_pin_history — reply-chain walk + curation event log MCP read tools

## Context

T-1578 took mcp_tools to 102. This wave adds two more navigation/curation reads:

- `termlink_agent_ancestors` — given a reply offset, walk up the `metadata.in_reply_to` chain until reaching a post with no parent. Returns the chain root → leaf as a JSON array of envelopes (sorted by depth ascending — root first, leaf last). Mirrors CLI T-1510. Companion to `termlink_agent_on_thread` which descends; this ascends.
- `termlink_agent_pin_history` — list ALL pin/unpin events on agent-chat-arc (not just the current state which T-1573 already gives). Returns `[{pin_target, sender_id, action, ts_unix_ms}, ...]` sorted newest-first. Mirrors CLI T-1535. Lets agents see the full curation timeline — not just "what's pinned now" but "what was ever pinned/unpinned, by whom, when".

Both pure walk + filter. Continue the established topic-walk-loop pattern.

## Acceptance Criteria

### Agent
- [x] New `AgentAncestorsParams` struct (offset u64, max_depth Option<u64>)
- [x] New `AgentPinHistoryParams` struct (limit Option<u64>)
- [x] New `termlink_agent_ancestors` tool method that builds offset->envelope map + walks up in_reply_to chain
- [x] New `termlink_agent_pin_history` tool method that walks topic + filters msg_type=pin + returns full event log
- [x] ancestors returns JSON array root-first; pin_history returns JSON array newest-first
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=104 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_ancestors` + `termlink_agent_pin_history` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_ancestors` with a known reply offset
  2. Compare with `target/release/termlink agent ancestors <offset>`
  3. Call `termlink_agent_pin_history`
  4. Compare with `target/release/termlink agent pin-history`
  **Expected:** MCP returns matching chain/log; CLI shows similar set.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_ancestors"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_pin_history"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** ancestors completes the thread-traversal duo (descend + ascend) — paired with on_thread (T-1575), MCP-aware agents can navigate any conversation in either direction. pin_history adds the timeline view on top of pinned (T-1573, current state) — full curation event log including unpins. Both <70 LOC each, continuing to ride the established walk-loop pattern.
**Evidence:**
- Build clean (4m 11s)
- `termlink version --json` reports mcp_tools=104 (was 102 after T-1578) — +2
- Verification gate 4/4 passed
- ancestors: offset→envelope map + chain-walk with safety cap (max_depth, default 100); pin_history: walk + filter + sort

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

### 2026-05-05T17:28:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1579-termlinkagentancestors--termlinkagentpin.md
- **Context:** Initial task creation

### 2026-05-05T17:34:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
