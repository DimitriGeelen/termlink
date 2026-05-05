---
id: T-1573
name: "termlink_agent_pinned + termlink_agent_starred — MCP read tools for chat-arc curation views"
description: >
  termlink_agent_pinned + termlink_agent_starred — MCP read tools for chat-arc curation views

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T16:20:38Z
last_update: 2026-05-05T16:20:38Z
date_finished: null
---

# T-1573: termlink_agent_pinned + termlink_agent_starred — MCP read tools for chat-arc curation views

## Context

T-1571 + T-1572 shipped the foundational MCP read primitives (recent + search). This wave layers two **derived** read tools on top — curation views that reduce the topic to its current pinned/starred state by walking msg_type=pin/star envelopes and applying latest-wins-per-target dedupe.

- `termlink_agent_pinned` — walks pin envelopes, groups by `pin_target`, keeps the latest by ts, returns only those whose final `action="pin"` (i.e. not subsequently unpinned). Mirrors CLI T-1517 `agent pinned`. Fleet-wide curation view.
- `termlink_agent_starred` — walks star envelopes, groups by `(sender_id, star_target)`, keeps latest by ts, returns only entries whose final `star="true"`. Optional `peer_fp` filter (default = all peers' stars). Mirrors CLI T-1518 `agent starred`. Personal bookmark view.

Pairs with `termlink_agent_pin` (T-1564) and `termlink_agent_star` (T-1565) — completes the curation read↔write surface for MCP-aware agents. Bundled per T-1559/T-1570 precedent (companion read pair, useless without their write counterparts).

## Acceptance Criteria

### Agent
- [x] New `AgentPinnedParams` struct (limit Option<u64>)
- [x] New `AgentStarredParams` struct (peer_fp Option<String>, limit Option<u64>)
- [x] New `termlink_agent_pinned` tool method that walks pin envelopes + dedupes by pin_target latest-wins + filters action=pin
- [x] New `termlink_agent_starred` tool method that walks star envelopes + dedupes by (sender_id, star_target) latest-wins + filters star=true
- [x] Both return JSON arrays (newest-first) with target offset, sender_id, ts_unix_ms
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=92 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_pinned` + `termlink_agent_starred` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_pinned`
  2. Compare with `target/release/termlink agent pinned`
  3. Call `termlink_agent_starred`
  4. Compare with `target/release/termlink agent starred`
  **Expected:** MCP returns matching curation entries; CLI shows similar set.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*(9[2-9]|1[0-9][0-9])'
grep -q '"termlink_agent_pinned"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_starred"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Third + fourth MCP read tools, completing the curation read↔write surface. Both reduce chat-arc to current state by walking msg_type-filtered envelopes and applying latest-wins-per-target dedupe (pinned: per pin_target; starred: per (sender_id, star_target)). Pairs cleanly with `termlink_agent_pin` (T-1564) + `termlink_agent_star` (T-1565). MCP-aware agents can now read curation state without shelling out — moving past raw envelope walks (T-1571/T-1572) into reduced-state reads.
**Evidence:**
- Build clean (4m 19s)
- `termlink version --json` reports mcp_tools=92 (was 90 after T-1572) — +2
- Verification gate 4/4 passed (grep × 2 + build + count)
- Reduce pattern (sort by ts ascending → HashMap insert → final filter) — established here, reusable for future state-reduction tools (e.g. ack-status, edits-of-final, redaction-final)

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

### 2026-05-05T16:20:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1573-termlinkagentpinned--termlinkagentstarre.md
- **Context:** Initial task creation
