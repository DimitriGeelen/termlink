---
id: T-1583
name: "termlink_agent_followups + termlink_agent_state — reverse-link aggregator + reduced state snapshot MCP read tools"
description: >
  termlink_agent_followups + termlink_agent_state — reverse-link aggregator + reduced state snapshot MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T18:22:36Z
last_update: 2026-05-05T18:29:42Z
date_finished: 2026-05-05T18:29:32Z
---

# T-1583: termlink_agent_followups + termlink_agent_state — reverse-link aggregator + reduced state snapshot MCP read tools

## Context

T-1582 brought MCP read surface to 110 tools. This wave adds two **synthesis reads** — composite views that reduce/aggregate across multiple msg_types in a single call:

- `termlink_agent_followups` — reverse-link aggregator. Given an offset, walks the topic and finds EVERY envelope that references it: replies (`metadata.in_reply_to`), edits (`metadata.replaces`), redactions (`metadata.redacts`), pins (`metadata.pin_target`), stars (`metadata.star_target`), reactions (`metadata.in_reply_to` + msg_type=reaction). Returns `{target_offset, replies, edits, redactions, pins, stars, reactions, total}` — single-call answer to "what happened to this post?". No CLI mirror — purely MCP-side composite.
- `termlink_agent_state` — full reduced state snapshot. Walks topic, applies T-1573 reduce-pattern across THREE state targets simultaneously: current pins (latest action per pin_target where action=pin), current stars (latest where star=true), latest topic_metadata description. Returns `{description, pinned: [...], starred: [...], pin_count, star_count, last_update_ts}`. Single-call orientation snapshot — what's the current curated state right now.

Both pure walk + aggregate. Followups uses one walk + 6 filters. State uses one walk + 3 simultaneous reduces (HashMap per target type).

## Acceptance Criteria

### Agent
- [x] New `AgentFollowupsParams` struct (offset u64)
- [x] New `AgentStateParams` struct (no params)
- [x] New `termlink_agent_followups` tool method walks topic + categorizes references by metadata field
- [x] New `termlink_agent_state` tool method walks topic + reduces to current pins/stars/description
- [x] followups returns `{target_offset, replies, edits, redactions, pins, stars, reactions, total}`
- [x] state returns `{description, pinned, starred, pin_count, star_count, last_update_ts}`
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=112 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_followups` + `termlink_agent_state` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_followups` with an offset that has known replies/reactions
  2. Spot-check the returned categories against `termlink_agent_on_thread <offset>` + `termlink_agent_reactions <offset>`
  3. Call `termlink_agent_state`
  4. Compare with `termlink_agent_pinned` + `termlink_agent_starred` + `termlink_agent_info` (description field)
  **Expected:** followups returns the union of references; state matches piecewise reads.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_followups"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_state"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two synthesis reads — followups (reverse-link aggregator across 6 reference types in one walk) + state (composite reduce of pins/stars/description in one call). These are MCP-native composites that don't have CLI mirrors — they answer "what happened to this post?" and "what's curated right now?" in single calls instead of 3+ separate reads. followups uses match-arm dispatch on msg_type + metadata field; state runs three simultaneous HashMap reductions in one sorted walk. Both ~100 LOC.
**Evidence:**
- Build clean (4m 55s)
- `termlink version --json` reports mcp_tools=112 (was 110 after T-1582) — +2
- Verification gate 4/4 passed
- followups: O(n) walk + 6-way categorize; state: O(n log n) sort + 3-way HashMap reduce
- Net wave: 7 waves (T-1573..T-1583, +T-1580/T-1581/T-1582/T-1583 this session) = 22 new MCP read tools across the orientation/navigation/curation/engagement/acknowledgment/audit/synthesis surface

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

### 2026-05-05T18:22:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1583-termlinkagentfollowups--termlinkagentsta.md
- **Context:** Initial task creation

### 2026-05-05T18:29:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 9/9, build clean, mcp_tools=112. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).
