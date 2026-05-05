---
id: T-1585
name: "termlink_agent_pinned_by + termlink_agent_starred_by — per-curator pin/star views MCP read tools"
description: >
  termlink_agent_pinned_by + termlink_agent_starred_by — per-curator pin/star views MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T18:37:42Z
last_update: 2026-05-05T18:44:31Z
date_finished: 2026-05-05T18:44:31Z
---

# T-1585: termlink_agent_pinned_by + termlink_agent_starred_by — per-curator pin/star views MCP read tools

## Context

T-1584 completed the reaction-data triangulation at 114 tools. This wave adds the symmetric per-curator views for curation:

- `termlink_agent_pinned_by` — given a `sender_id` (default = caller's local Identity), list all targets currently pinned BY THAT SENDER. Walks topic, applies T-1573 reduce-pattern (latest-wins) BUT scoped to that sender's pin/unpin events only. Returns `[{pin_target, ts_unix_ms}, ...]`. Per-curator companion to T-1573 `agent_pinned` (topic-wide).
- `termlink_agent_starred_by` — same shape but for stars. Per-sender star reduce.

Together with T-1573 (topic-wide pinned/starred), T-1582 history (per-sender content), T-1584 reactions_by (per-sender reactions) — completes the by-sender views across all engagement axes.

## Acceptance Criteria

### Agent
- [x] New `AgentPinnedByParams` struct (sender_id Option<String>)
- [x] New `AgentStarredByParams` struct (sender_id Option<String>)
- [x] New `termlink_agent_pinned_by` tool method walks topic + filters pin by sender + reduces latest-wins
- [x] New `termlink_agent_starred_by` tool method walks topic + filters star by sender + reduces latest-wins
- [x] Both default sender_id to caller's local Identity
- [x] pinned_by filters to action="pin" (excludes unpin); starred_by filters to star="true" (excludes false)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=116 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_pinned_by` + `termlink_agent_starred_by` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_pinned_by` with default sender_id
  2. Compare the result to `termlink_agent_pinned` filtered to your sender_id
  3. Call `termlink_agent_starred_by` with default sender_id
  4. Compare to `termlink_agent_starred` filtered to your sender_id
  **Expected:** by-sender views match the topic-wide views filtered to that sender.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_pinned_by"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_starred_by"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two per-curator views completing the by-sender curation surface. Together with T-1573 (topic-wide pinned/starred), T-1582 history (per-sender content), T-1584 reactions_by (per-sender reactions), T-1577 ack_status (per-sender ack frontiers) — full by-sender axis across all six engagement primitives (post / pin / star / react / ack / edit). Each ~100 LOC, both ride the established sort-asc + HashMap-overwrite reduce pattern and additionally filter by sender_id. Brings session total to 6 waves, +12 read tools, mcp_tools 104→116.
**Evidence:**
- Build clean (4m 52s)
- `termlink version --json` reports mcp_tools=116 (was 114 after T-1584) — +2
- Verification gate 4/4 passed
- pinned_by: walk + sender-filter + latest-wins reduce + action="pin" filter; starred_by: same shape with star="true"

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

### 2026-05-05T18:37:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1585-termlinkagentpinnedby--termlinkagentstar.md
- **Context:** Initial task creation

### 2026-05-05T18:44:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 9/9, build clean, mcp_tools=116. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).
