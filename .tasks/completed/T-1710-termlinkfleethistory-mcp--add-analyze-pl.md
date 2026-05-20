---
id: T-1710
name: "termlink_fleet_history MCP — add analyze (PL-021 flap detection parity with CLI T-1690, G-057 punch-list #1)"
description: >
  termlink_fleet_history MCP — add analyze (PL-021 flap detection parity with CLI T-1690, G-057 punch-list #1)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-19T12:07:01Z
last_update: 2026-05-19T12:14:04Z
date_finished: 2026-05-19T12:14:04Z
---

# T-1710: termlink_fleet_history MCP — add analyze (PL-021 flap detection parity with CLI T-1690, G-057 punch-list #1)

## Context

T-1690 added `--analyze` to CLI `fleet history` — classifies each hub's rotation
history into `clean`/`cert-only`/`secret-only`/`single-double-rotation`/`pl021-candidate`
to detect volatile runtime_dir (PL-021 signature: ≥2 simultaneous cert+secret
rotations in window). T-1687 shipped MCP parity for `fleet history` but ONLY
the chronological listing path — the `--analyze` switch is silently absent
from `termlink_fleet_history` MCP. G-057/PL-167 pattern.

Value for LLM agents: a flap-investigating agent ("is this hub's drift the
1st or Nth time? is it PL-021?") can answer both questions in one MCP call
(`since_days`, `analyze: true`) instead of either parsing the chronological
output itself or shelling out to the CLI.

Punch-list source: T-1707/T-1708/T-1709 round-7 wrap-up audit; this is gap #1
from the G-057 diagnostic-verb-family scan.

## Acceptance Criteria

### Agent
- [x] `FleetHistoryParams` gains `analyze: Option<bool>` field with rustdoc
- [x] Tool description string explicitly mentions `analyze` + PL-021 detection
- [x] When `analyze=true`, handler short-circuits BEFORE chronological listing path and returns the per-hub flap report (parity with CLI T-1690 JSON shape: `{ok, since_days, hub_filter, log_path, hubs[], pl021_candidates}`)
- [x] Pure-function `analyze_pl021_mcp(&[entries]) -> Vec<HubFlapReport>` reimplements T-1690's classifier inline (G-057 convention — CLI's `analyze_pl021` is `pub(crate)`, not reachable from termlink-mcp crate)
- [x] Unit tests cover all 5 verdicts (`clean`/`cert-only`/`secret-only`/`single-double-rotation`/`pl021-candidate`) + cross-hub isolation + recovery-transition-not-counted
- [x] `cargo build -p termlink-mcp` passes (no new warnings introduced)
- [x] All existing `tests::fleet_doctor` and `tests::fleet_history` tests still pass

## Verification

cargo build -p termlink-mcp 2>&1 | tail -3 | grep -qE "Finished"
cargo test -p termlink-mcp --lib tests::fleet_history 2>&1 | tail -3 | grep -qE "test result: ok\..*0 failed"
cargo test -p termlink-mcp --lib tests::fleet_doctor 2>&1 | tail -3 | grep -qE "test result: ok\..*0 failed"
grep -q "pub analyze: Option<bool>" crates/termlink-mcp/src/tools.rs
grep -q "analyze_pl021_mcp" crates/termlink-mcp/src/tools.rs

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

### 2026-05-19T12:30Z — implementation complete [agent]
- **Code:** crates/termlink-mcp/src/tools.rs
  - Added `FleetHistoryParams.analyze: Option<bool>` (with rustdoc).
  - Added `FleetHistoryFlapVerdict` enum + `FleetHistoryFlapReport` struct (parallels CLI T-1690's `pub(crate)` types).
  - Added pure `analyze_pl021_mcp(&[entries]) -> Vec<FleetHistoryFlapReport>` classifier.
  - Wired `analyze=true` short-circuit in `termlink_fleet_history` handler: returns CLI T-1690 JSON shape (`{ok, since_days, hub_filter, log_path, hubs[], pl021_candidates, malformed_lines_skipped}`) — NOT the chronological-listing shape.
  - Empty-state path returns analyze shape with `hubs=[], pl021_candidates=false` when analyze=true.
  - `include_heals` forced to `false` when `analyze=true` (classifier is rotation-only).
  - Updated tool description string to advertise analyze + PL-021 detection.
- **Tests:** 11 new unit tests (`analyze_pl021_mcp_*`) + 2 new e2e tests (`fleet_history_e2e_analyze_*`). Covers all 5 verdicts (clean/cert-only/secret-only/single-double-rotation/pl021-candidate), cross-hub isolation, recovery-not-counted, already-drifted-not-counted, heal-event-skipped, kind=new-skipped, and CLI fallthrough (cert+secret in separate rows).
- **Verification:**
  - `cargo build -p termlink-mcp` → `Finished` (only pre-existing unused_assignments warning at line 14052, not my code)
  - `cargo test tests::analyze_pl021` → 11 passed; 0 failed
  - `cargo test tests::fleet_history` → 11 passed; 0 failed (9 existing + 2 new)
  - `cargo test tests::fleet_doctor` → 28 passed; 0 failed (regression-clean)

### 2026-05-19T12:07:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1710-termlinkfleethistory-mcp--add-analyze-pl.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-e53ded6f
- **Timestamp:** 2026-05-19T12:14:13Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-19T12:14:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
