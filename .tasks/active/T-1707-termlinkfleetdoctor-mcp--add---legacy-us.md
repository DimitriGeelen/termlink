---
id: T-1707
name: "termlink_fleet_doctor MCP — add --legacy-usage parity with CLI (drives T-1166 cut readiness from MCP)"
description: >
  termlink_fleet_doctor MCP — add --legacy-usage parity with CLI (drives T-1166 cut readiness from MCP)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [mcp, fleet-doctor, T-1166, T-1432, G-057, PL-167]
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: [T-1432, T-1166, T-1692, T-1706]
created: 2026-05-19T07:13:53Z
last_update: 2026-05-19T07:13:53Z
date_finished: null
---

# T-1707: termlink_fleet_doctor MCP — add --legacy-usage parity with CLI (drives T-1166 cut readiness from MCP)

## Context

The CLI `termlink fleet doctor --legacy-usage --legacy-window-days N` (T-1432)
returns the authoritative T-1166 cut-readiness verdict by aggregating each
hub's `hub.legacy_usage` Tier-A RPC result into one of CUT-READY,
CUT-READY-DECAYING, WAIT, or UNCERTAIN. The MCP `termlink_fleet_doctor` tool
shipped pre-T-1432 and only returns connectivity status (ok / error /
timeout) — the rich cut-readiness signal is invisible to LLM-agent callers.
That matches the G-057 / PL-167 pattern: a protocol/CLI feature exists but
the MCP wrapper doesn't surface it.

This task adds `legacy_usage: Option<bool>` and `legacy_window_days: Option<u64>`
params to MCP `termlink_fleet_doctor` and, when enabled, calls
`hub.legacy_usage` per hub, aggregates the verdict using the same logic as
the CLI (replicated inline rather than cross-crate, matching the T-1706
pattern from G-057's parallel-implementation reality), and surfaces a
`legacy_summary` object in the response. Pre-T-1432 hubs return
`audit_unsupported: true` with an upgrade hint (mirrors CLI).

## Acceptance Criteria

### Agent
- [x] `FleetDoctorParams` gains `legacy_usage: Option<bool>` (default false) and `legacy_window_days: Option<u64>` (default 7, clamped 1..=90)
- [x] When `legacy_usage = true`: for each successfully-connected hub, call `hub.legacy_usage` with `{"window_seconds": days * 86400}`. Method-not-found / RPC error → per-hub `legacy_usage = {audit_unsupported: true, hint: "..."}`. Success → per-hub `legacy_usage = <result>`.
- [x] When `legacy_usage = true`: top-level response gains `legacy_summary` with `{verdict, window_days, total_legacy_fleet, hubs_clean, hubs_with_traffic, hubs_unsupported, hubs_no_audit}`. Verdict computed identically to CLI: CUT-READY / CUT-READY-DECAYING / WAIT / UNCERTAIN (5-min active-traffic threshold).
- [x] When `legacy_usage = false` (default): response shape is byte-identical to pre-T-1707 (no behavior change for existing callers)
- [x] Tool description string updated to mention `--legacy-usage` option and link to T-1166
- [x] Unit tests cover: verdict computation (CUT-READY / DECAYING / WAIT / UNCERTAIN cases), params shape with/without legacy fields, `legacy_window_days` clamp behavior
- [x] `cargo build -p termlink-mcp` clean; no clippy warnings introduced

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

cargo build -p termlink-mcp 2>&1 | tail -3 | grep -qE "Finished"
cargo test -p termlink-mcp --lib tests::fleet_doctor 2>&1 | tail -5 | grep -qE "11 passed"

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

### 2026-05-19T07:13:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1707-termlinkfleetdoctor-mcp--add---legacy-us.md
- **Context:** Initial task creation

### 2026-05-19T07:25Z — implementation complete [agent]
- **Files:** `crates/termlink-mcp/src/tools.rs` (FleetDoctorParams + termlink_fleet_doctor handler + aggregate_legacy_summary + compute_legacy_verdict free fns + 7 new unit tests)
- **Behavior:** opt-in `legacy_usage` param triggers per-hub `hub.legacy_usage` RPC; aggregation produces `legacy_summary` keyed by verdict. Default (no param) shape is byte-identical to pre-T-1707.
- **Tests:** 11/0/0 (5 pre-existing + 6 new under `tests::fleet_doctor`). Verdict cases covered: CUT-READY (clean), CUT-READY-DECAYING (stale traffic), WAIT (active traffic), UNCERTAIN (unsupported), UNCERTAIN (zero hubs).
- **Pattern:** G-057/PL-167 parallel-implementation matched T-1706 (MCP doctor identity check). CLI and MCP doctor remain separate compilation units; shared logic replicated rather than cross-crate dep.
