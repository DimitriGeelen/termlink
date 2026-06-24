---
id: T-1708
name: "termlink_fleet_doctor MCP — add include_pin_check + pin_check_summary (T-1666 parity for LLM agents)"
description: >
  termlink_fleet_doctor MCP — add include_pin_check + pin_check_summary (T-1666 parity for LLM agents)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [mcp, fleet-doctor, T-1666, G-057, PL-167]
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: [T-1666, T-1707, T-1692, T-1706]
created: 2026-05-19T07:23:59Z
last_update: 2026-05-19T07:27:38Z
date_finished: 2026-05-19T07:27:38Z
---

# T-1708: termlink_fleet_doctor MCP — add include_pin_check + pin_check_summary (T-1666 parity for LLM agents)

## Context

CLI `termlink fleet doctor --include-pin-check` (T-1666) runs TLS probes in
parallel across every configured hub and reports per-profile pin status
(match / drift / no-pin / probe-fail) alongside auth diagnostics — a single
command that answers "auth-mismatch OR cert-drift OR both?" The MCP
`termlink_fleet_doctor` after T-1707 carries the auth + legacy_usage signal
but not the pin signal — so an LLM agent investigating a fleet incident
must call `termlink_fleet_verify` separately and correlate by hub name.

This task extends `FleetDoctorParams` with `include_pin_check: Option<bool>`,
runs the same parallel `probe_cert_with_timeout` pattern already used by
`termlink_fleet_verify` (T-1661) before the per-hub connect loop, injects
the per-hub pin status into each hub_obj, and attaches a top-level
`pin_check_summary {verdict, profiles}` mirroring the CLI shape. Closes
G-057/PL-167 on the pin axis of fleet_doctor and completes the "unified
single-call diagnostic" experience for MCP consumers.

## Acceptance Criteria

### Agent
- [x] `FleetDoctorParams` gains `include_pin_check: Option<bool>` (default false)
- [x] When `include_pin_check = true`: parallel TLS-probe every configured hub (`probe_cert_with_timeout` with the doctor's per-hub timeout); inject `pin_check: {status, wire, pinned, error}` into each per-hub object. Reuses the same `termlink_session::tofu::KnownHubStore::default_store()` + `probe_cert_with_timeout` primitives as `termlink_fleet_verify` so the two tools agree on rotation state.
- [x] When `include_pin_check = true`: response gains top-level `pin_check_summary: {verdict, any_drift, any_probe_fail, any_no_pin, profiles}` where verdict is one of `match` / `drift` / `probe-fail` / `no-pin` (drift dominates).
- [x] When `include_pin_check = false` (default): response shape is byte-identical to T-1707 (no behavior change for existing callers)
- [x] Tool description string mentions `include_pin_check` and links to T-1666
- [x] Unit tests cover: verdict-precedence rules (drift > probe-fail > no-pin > match), the params shape (with/without pin_check), and a smoke test that aggregate_pin_check_summary handles an empty profile list
- [x] `cargo build -p termlink-mcp` clean; tests in `tests::fleet_doctor_pin_check` all pass

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
cargo test -p termlink-mcp --lib tests::fleet_doctor 2>&1 | tail -5 | grep -qE "passed"

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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

### 2026-05-19T07:23:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1708-termlinkfleetdoctor-mcp--add-includepinc.md
- **Context:** Initial task creation

### 2026-05-19T07:35Z — implementation complete [agent]
- **Files:** `crates/termlink-mcp/src/tools.rs` (FleetDoctorParams gains `include_pin_check`; new helpers `inject_pin_check` + `aggregate_pin_check_summary`; handler now parallel-probes TLS pre-loop using same primitives as `termlink_fleet_verify`)
- **Behavior:** opt-in `include_pin_check` adds per-hub `pin_check {status, wire, pinned, error}` AND top-level `pin_check_summary {verdict, any_*, profiles}`. Pin_check is orthogonal to RPC connectivity — injected on ok/error/timeout paths. Default off → byte-identical to T-1707.
- **Tests:** 20/0/0 under `tests::fleet_doctor` (11 prior + 9 new): verdict precedence (drift>probe-fail>no-pin>match), empty-profile vacuous-match, params-shape, inject no-op vs populated.
- **Single-call diagnostic:** LLM agent can now ask `termlink_fleet_doctor {legacy_usage: true, include_pin_check: true}` and get back EVERY rotation signal (auth, cert, cut-readiness) in one call. Matches CLI T-1666's unified-doctor goal.

## Reviewer Verdict (v1.4)

- **Scan ID:** R-d114b33b
- **Timestamp:** 2026-05-19T07:27:49Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-19T07:27:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
