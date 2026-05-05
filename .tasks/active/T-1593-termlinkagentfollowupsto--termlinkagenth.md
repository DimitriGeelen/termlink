---
id: T-1593
name: "termlink_agent_followups_to + termlink_agent_help — replies-to-a-sender + MCP surface introspection read tools"
description: >
  termlink_agent_followups_to + termlink_agent_help — replies-to-a-sender + MCP surface introspection read tools

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T21:55:44Z
last_update: 2026-05-05T21:55:44Z
date_finished: null
---

# T-1593: termlink_agent_followups_to + termlink_agent_help — replies-to-a-sender + MCP surface introspection read tools

## Context

T-1592 brought MCP read surface to 130 tools. Wave 49 adds two **fleet/introspection reads**:

- `termlink_agent_followups_to` — given a `sender_id`, walks topic, finds posts authored by that sender, then collects all replies whose `metadata.in_reply_to` points back. Returns `[{reply_offset, parent_offset, reply_sender_id, ts_unix_ms}, ...]` sorted newest-first. Inverse of T-1583 `agent_followups` (single offset → its replies) and orthogonal to T-1523 `agent_replies-of` (replies BY sender). New axis: engagement RECEIVED per peer.
- `termlink_agent_help` — MCP surface introspection. Calls `TermLinkTools::new().tool_router.list_all()`, filters to `termlink_agent_*` tools, returns `{total, agent_tools: [{name, description}, ...]}` sorted alphabetically. Self-documenting reads for the 130-tool surface — what an MCP-aware agent calls first to learn the protocol.

Both pure walk + filter (or pure introspection).

## Acceptance Criteria

### Agent
- [x] New `AgentFollowupsToParams` struct (sender_id String, limit Option<u64>)
- [x] New `AgentHelpParams` struct (no fields)
- [x] New `termlink_agent_followups_to` walks topic + identifies sender's posts + collects replies via in_reply_to
- [x] New `termlink_agent_help` walks rmcp router + filters termlink_agent_* tools + sorts by name
- [x] followups_to default limit=100, capped at 500
- [x] help returns total count + sorted name+description pairs
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=132 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_followups_to` + `termlink_agent_help` are operator-fluent over MCP
  **Steps:**
  1. Pick a sender_id from `termlink_agent_peers`
  2. Call `termlink_agent_followups_to` with that sender_id
  3. Verify list of replies received by that peer
  4. Call `termlink_agent_help` with no args
  5. Verify alphabetical termlink_agent_* tool list with descriptions
  **Expected:** followups_to surfaces engagement received per peer; help is one-call protocol learning.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_followups_to"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_help"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two reads on different axes. followups_to closes the engagement-received axis (companion to followups by-offset and replies-of by-author). help is the first introspection tool — at 130 tools the MCP surface is dense enough that protocol-learning-by-listing is high-value. Both ~70-90 LOC. Brings session total to 14 waves, +28 read tools, mcp_tools 104→132.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=132 (was 130 after T-1592) — +2
- Verification gate 4/4 (TBD)
- followups_to: O(n) walk + sender's-posts set + reply filter via in_reply_to; help: rmcp router list_all + name-prefix filter + alpha sort

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

### 2026-05-05T21:55:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1593-termlinkagentfollowupsto--termlinkagenth.md
- **Context:** Initial task creation
