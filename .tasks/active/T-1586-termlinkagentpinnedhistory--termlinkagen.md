---
id: T-1586
name: "termlink_agent_pinned_history + termlink_agent_starred_history — per-target pin/star audit log MCP read tools"
description: >
  termlink_agent_pinned_history + termlink_agent_starred_history — per-target pin/star audit log MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T19:01:05Z
last_update: 2026-05-05T19:12:36Z
date_finished: 2026-05-05T19:07:56Z
---

# T-1586: termlink_agent_pinned_history + termlink_agent_starred_history — per-target pin/star audit log MCP read tools

## Context

T-1585 brought MCP read surface to 116 tools. This wave adds the symmetric **per-target audit logs** for pin/star events:

- `termlink_agent_pinned_history` — given a `pin_target` offset, walks topic and returns the FULL chronological log of every pin/unpin event affecting that target. Each entry: `{action, sender_id, ts_unix_ms}`. Per-target companion to T-1573 `agent_pinned` (current state) and T-1585 `agent_pinned_by` (per-curator). Answers "how has this target's pin state evolved over time?" — useful for debugging contested pins or pin/unpin flapping.
- `termlink_agent_starred_history` — same shape but for stars. Each entry: `{star_value, sender_id, ts_unix_ms}` where star_value is the `star` payload field (true/false).

Both pure walk + filter on metadata.{pin_target,star_target} == target_offset, sorted ascending. Rides established walk-loop pattern. Together with topic_metadata_history (T-1584) and edits_of (T-1581), completes the per-target audit-log axis: description history, edit history, pin history, star history.

## Acceptance Criteria

### Agent
- [x] New `AgentPinnedHistoryParams` struct (pin_target u64)
- [x] New `AgentStarredHistoryParams` struct (star_target u64)
- [x] New `termlink_agent_pinned_history` tool method walks topic + filters pin events by pin_target
- [x] New `termlink_agent_starred_history` tool method walks topic + filters star events by star_target
- [x] Both return chronologically sorted (oldest-first) audit log
- [x] pinned_history extracts action ("pin"|"unpin") from metadata
- [x] starred_history extracts star value ("true"|"false") from metadata
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=118 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_pinned_history` + `termlink_agent_starred_history` are operator-fluent over MCP
  **Steps:**
  1. Identify a pinned offset on agent-chat-arc (use `termlink_agent_pinned`)
  2. Call `termlink_agent_pinned_history` with that offset as `pin_target`
  3. Verify the chronological log matches your memory of pin/unpin events
  4. Repeat for a starred offset with `termlink_agent_starred_history`
  **Expected:** full chronological audit returns one entry per pin/unpin (or star/unstar) event, oldest first.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_pinned_history"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_starred_history"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two per-target audit logs completing the per-target history axis. Together with topic_metadata_history (T-1584) and edits_of (T-1581), forms the full per-target audit surface across all four mutable targets: description, content, pin state, star state. Pure walk + filter on metadata.{pin_target,star_target}. ~80 LOC each, rides established sort-asc + filter pattern. Brings session total to 7 waves, +14 read tools, mcp_tools 104→118.
**Evidence:**
- Build clean (4m 40s)
- `termlink version --json` reports mcp_tools=118 (was 116 after T-1585) — +2
- Verification gate 4/4 passed
- pinned_history: O(n) walk + filter on metadata.pin_target, returns chronological log; starred_history: same shape with star_target

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

### 2026-05-05T19:01:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1586-termlinkagentpinnedhistory--termlinkagen.md
- **Context:** Initial task creation

### 2026-05-05T19:07:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 9/9, build clean 4m40s, mcp_tools=118. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).
