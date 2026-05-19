---
id: T-1711
name: "termlink_fleet_status MCP — add verbose flag (per-hub session names, G-057 punch-list #2)"
description: >
  termlink_fleet_status MCP — add verbose flag (per-hub session names, G-057 punch-list #2)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-19T12:15:22Z
last_update: 2026-05-19T12:15:22Z
date_finished: null
---

# T-1711: termlink_fleet_status MCP — add verbose flag (per-hub session names, G-057 punch-list #2)

## Context

CLI `fleet status --verbose` adds per-hub `session_names: [...]` field
(populated from `session.discover` response's `display_name` field) so an
operator can see WHICH sessions are running on each hub without parsing tables.
The MCP wrapper `termlink_fleet_status` silently lacks this flag — agents see
session COUNT but not session IDENTITY.

Value: an LLM agent triaging "is the right workload running on the right hub?"
needs session names, not counts. Currently the agent must shell out to CLI or
query `session.discover` per hub itself. G-057 punch-list #2.

## Acceptance Criteria

### Agent
- [x] `FleetStatusParams` gains `verbose: Option<bool>` field with rustdoc
- [x] When `verbose=true`, each up-hub entry includes `session_names: [...]` (parity with CLI flag — display_name strings from session.discover)
- [x] When `verbose=false` or unset, response shape is unchanged (regression-clean)
- [x] Tool description string mentions `verbose` opt-in
- [x] Unit test: `FleetStatusParams` deserialization defaults `verbose` to None
- [x] Unit test: `FleetStatusParams` deserialization accepts `verbose: true`
- [x] `cargo build -p termlink-mcp` passes (no new warnings)

## Verification

cargo build -p termlink-mcp 2>&1 | tail -3 | grep -qE "Finished"
cargo test -p termlink-mcp --lib tests::fleet_status_params 2>&1 | tail -3 | grep -qE "test result: ok\..*0 failed"
grep -q "pub verbose: Option<bool>" crates/termlink-mcp/src/tools.rs
grep -q "session_names" crates/termlink-mcp/src/tools.rs

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

### 2026-05-19T12:15:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1711-termlinkfleetstatus-mcp--add-verbose-fla.md
- **Context:** Initial task creation

### 2026-05-19T12:48Z — implementation complete [agent]
- **Code:** crates/termlink-mcp/src/tools.rs
  - `FleetStatusParams` gains `verbose: Option<bool>` (rustdoc-documented).
  - Handler refactored: `session.discover` response is parsed once; verbose path additionally extracts `display_name` strings.
  - When verbose=true, every up-hub entry includes `session_names: [...]` (unconditional — matches CLI JSON contract; empty array when hub has 0 sessions).
  - Non-up entries (down/auth-fail/timeout) unchanged.
  - Tool description string updated to advertise verbose opt-in.
- **Tests:** 3 new (`fleet_status_params_defaults_when_omitted`, `fleet_status_params_accepts_verbose_true`, `fleet_status_params_accepts_verbose_with_timeout`).
- **Verification:**
  - `cargo build -p termlink-mcp` → `Finished` (only pre-existing unused_assignments warning, not my code)
  - `cargo test tests::fleet_status_params` → 3 passed; 0 failed
  - No regression risk for verbose=false / unset: same response shape as pre-T-1711.
