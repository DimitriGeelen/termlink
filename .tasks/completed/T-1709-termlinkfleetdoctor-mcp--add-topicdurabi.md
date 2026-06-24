---
id: T-1709
name: "termlink_fleet_doctor MCP — add topic_durability + bus_state_summary (T-1446 / G-050 parity for LLM agents)"
description: >
  termlink_fleet_doctor MCP — add topic_durability + bus_state_summary (T-1446 / G-050 parity for LLM agents)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [mcp, fleet-doctor, T-1446, G-050, PL-021, G-057, PL-167]
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: [T-1446, T-1707, T-1708, T-1294, T-1296]
created: 2026-05-19T07:32:45Z
last_update: 2026-05-19T07:35:53Z
date_finished: 2026-05-19T07:35:53Z
---

# T-1709: termlink_fleet_doctor MCP — add topic_durability + bus_state_summary (T-1446 / G-050 parity for LLM agents)

## Context

CLI `termlink fleet doctor --topic-durability` (T-1446) probes each hub's
`hub.bus_state` Tier-A RPC and aggregates a fleet-wide G-050 verdict:
DURABLE / VOLATILE / UNCERTAIN. VOLATILE means the hub's `runtime_dir`
lives on /tmp (or another wipe-on-boot mount) so identity rotates every
reboot — the structural cause of PL-021. The CLI surfaces this with
operator-actionable hints ("migrate runtime_dir off /tmp"). The MCP
`termlink_fleet_doctor` after T-1707/T-1708 lacks this signal.

This task completes the fleet_doctor MCP feature surface by adding
`topic_durability: Option<bool>` to FleetDoctorParams and a top-level
`bus_state_summary {verdict, hubs_durable, hubs_volatile, hubs_missing,
hubs_unsupported}` response field that mirrors the CLI. LLM agents can
now ask one fleet_doctor call with `{legacy_usage: true, include_pin_check:
true, topic_durability: true}` and get the full fleet-rotation/cut-readiness/
durability picture in a single round-trip.

## Acceptance Criteria

### Agent
- [x] `FleetDoctorParams` gains `topic_durability: Option<bool>` (default false)
- [x] When `topic_durability = true`: for each successfully-connected hub, call `hub.bus_state` (no params); on method-not-found / RPC error → per-hub `bus_state = {audit_unsupported: true, hint: ...}`. Success → per-hub `bus_state = <result>`.
- [x] When `topic_durability = true`: top-level response gains `bus_state_summary` with `{verdict, hubs_durable, hubs_volatile, hubs_missing, hubs_unsupported}`. Verdict: VOLATILE (any vol=true) > UNCERTAIN (unsupported or missing meta.db) > DURABLE (all present + not-volatile). Mirrors CLI rule exactly.
- [x] When `topic_durability = false` (default): response shape is byte-identical to T-1708 (no behavior change for existing callers)
- [x] Tool description string mentions `topic_durability` and links to T-1446 / G-050
- [x] Unit tests cover: verdict precedence (VOLATILE > UNCERTAIN > DURABLE), params shape, empty-fleet aggregation
- [x] `cargo build -p termlink-mcp` clean; existing fleet_doctor tests still pass

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

### 2026-05-19T07:32:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1709-termlinkfleetdoctor-mcp--add-topicdurabi.md
- **Context:** Initial task creation

### 2026-05-19T07:45Z — implementation complete [agent]
- **Files:** `crates/termlink-mcp/src/tools.rs` (FleetDoctorParams gains `topic_durability`; new helper `aggregate_bus_state_summary`; handler probes `hub.bus_state` per hub on the ok path)
- **Behavior:** opt-in `topic_durability` adds per-hub `bus_state` field AND top-level `bus_state_summary {verdict, hubs_durable, hubs_volatile, hubs_missing, hubs_unsupported}`. Verdict precedence: VOLATILE > UNCERTAIN > DURABLE. Default off → byte-identical to T-1708.
- **Tests:** 28/0/0 under `tests::fleet_doctor` (20 prior + 8 new for topic_durability): durable, volatile-dominates, uncertain-on-missing-meta-db, uncertain-on-unsupported-only, skip-hubs-without-bus-state, empty-fleet, params shape.
- **Fleet doctor MCP feature surface complete:** legacy_usage (T-1707) + include_pin_check (T-1708) + topic_durability (T-1709) — LLM agents now have full CLI parity for the three opt-in cut-readiness / rotation / durability diagnostics in a single call.

## Reviewer Verdict (v1.4)

- **Scan ID:** R-26a80614
- **Timestamp:** 2026-05-19T07:36:07Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-19T07:35:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
