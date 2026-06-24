---
id: T-1852
name: "termlink_agent_chat_arc_recent MCP wrapper (T-1849 follow-on)"
description: >
  Add MCP tool wrapping scripts/agent-chat-arc-recent.sh — agent-callable parity with termlink_fleet_adoption_snapshot (T-1847) and termlink_agent_listeners_fleet (T-1839). Completes MCP coverage of the discovery triangle.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [doorbell-mail, mcp, t-1849-followon]
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-28T19:53:21Z
last_update: 2026-05-28T19:55:29Z
date_finished: 2026-05-28T19:55:29Z
---

# T-1852: termlink_agent_chat_arc_recent MCP wrapper (T-1849 follow-on)

## Context

T-1849 shipped `scripts/agent-chat-arc-recent.sh`. T-1847 + T-1839 established the MCP wrapper pattern (using `resolve_t1836_script` + `run_t1836_subprocess`). This task fills the third + final MCP wrapper for the discovery triangle: agents using MCP can now answer "who's there / is it healthy / what's been said" all natively without shelling out.

## Acceptance Criteria

### Agent
- [x] `AgentChatArcRecentParams` struct in `crates/termlink-mcp/src/tools.rs` next to existing T-1836-family params.
- [x] `termlink_agent_chat_arc_recent` `#[tool]` method on the same impl block as `termlink_fleet_adoption_snapshot`. Always passes `--json`. Reuses `resolve_t1836_script` + `run_t1836_subprocess`. Default timeout 30s clamped 1..=120.
- [x] Params: all 8 fields wired with proper clamps.
- [x] Tool description: discovery-triangle role + PL-188/189/191 references + parsed envelope shape.
- [x] `cargo check --package termlink-mcp` clean (10.90s, only pre-existing L23029 warning).
- [x] `cargo build --package termlink-mcp` clean (17.01s).

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

grep -q 'termlink_agent_chat_arc_recent' crates/termlink-mcp/src/tools.rs
grep -q 'AgentChatArcRecentParams' crates/termlink-mcp/src/tools.rs
cargo check --package termlink-mcp 2>&1 | tail -5

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

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-28T19:53:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1852-termlinkagentchatarcrecent-mcp-wrapper-t.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-57a31e25
- **Timestamp:** 2026-05-28T19:55:34Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T19:55:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
