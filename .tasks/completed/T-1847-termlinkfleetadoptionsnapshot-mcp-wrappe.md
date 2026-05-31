---
id: T-1847
name: "termlink_fleet_adoption_snapshot MCP wrapper (T-1843 follow-on)"
description: >
  Add MCP tool wrapping scripts/fleet-adoption-snapshot.sh — agent-callable parity with termlink_agent_listeners_fleet (T-1839). Without this, MCP-side agents must shell out to inspect adoption_state.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [doorbell-mail, mcp, adoption, t-1843-followon]
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-28T18:43:50Z
last_update: 2026-05-28T18:46:24Z
date_finished: 2026-05-28T18:46:24Z
---

# T-1847: termlink_fleet_adoption_snapshot MCP wrapper (T-1843 follow-on)

## Context

T-1843 shipped `scripts/fleet-adoption-snapshot.sh`. T-1839 already added MCP parity for `agent-listeners-fleet.sh` (`termlink_agent_listeners_fleet`). This task fills the symmetric gap for the adoption gauge so MCP agents can query HOT/WARM/COLD without shelling out.

## Acceptance Criteria

### Agent
- [x] `FleetAdoptionSnapshotParams` struct added to `crates/termlink-mcp/src/tools.rs` near the other T-1836 family params: `since_hours: Option<u32>`, `hubs_file: Option<String>`, `timeout_secs: Option<u64>`.
- [x] `termlink_fleet_adoption_snapshot` `#[tool]` method registered on the same impl block as `termlink_agent_listeners_fleet`. Reuses `resolve_t1836_script` + `run_t1836_subprocess`. Always passes `--json`. Default timeout 30s, clamped 1..=120. Since-hours clamp 1..=720 (mirrors script-side validation).
- [x] Tool description references T-1843 + T-1846 and explains HOT/WARM/COLD semantics so MCP clients understand the gauge.
- [x] `cargo check --package termlink-mcp` clean (12.52s, only pre-existing warning at line 23001).
- [x] `cargo build --package termlink-mcp` succeeds (18.35s — crate is lib-only, no bin target; AC overspecified `--bin termlink-mcp` which doesn't exist).

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

cargo check --package termlink-mcp 2>&1 | tail -5
grep -q 'termlink_fleet_adoption_snapshot' crates/termlink-mcp/src/tools.rs
grep -q 'FleetAdoptionSnapshotParams' crates/termlink-mcp/src/tools.rs

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

### 2026-05-28T18:43:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1847-termlinkfleetadoptionsnapshot-mcp-wrappe.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-90fd2962
- **Timestamp:** 2026-05-28T18:46:29Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T18:46:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
