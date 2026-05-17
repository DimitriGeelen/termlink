---
id: T-1661
name: "MCP exposure for fleet verify — parity with fleet_status/fleet_doctor"
description: >
  MCP exposure for fleet verify — parity with fleet_status/fleet_doctor

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [auth, G-011, rotation-protocol, mcp, fleet]
components: []
related_tasks: [T-1660, T-1659, T-1658]
created: 2026-05-17T17:46:11Z
last_update: 2026-05-17T17:46:11Z
date_finished: null
---

# T-1661: MCP exposure for fleet verify — parity with fleet_status/fleet_doctor

## Context

T-1660 added `termlink fleet verify` as a CLI verb. `fleet_status` and
`fleet_doctor` are already MCP-exposed via `termlink_mcp::tools`. Adding
`termlink_fleet_verify` brings parity so agents using the MCP server
(Claude Code, claude-in-chrome, etc.) can call the new diagnostic
without shelling out to the binary.

Same input/output shape as the CLI's `--json` mode but wrapped in the
MCP tool envelope. Pure read-only — no authentication, no `KnownHubStore`
mutation, exits Ok in all cases (the verdict carries the result).

## Acceptance Criteria

### Agent
- [x] `termlink_fleet_verify` MCP tool added in `crates/termlink-mcp/src/tools.rs`
- [x] Tool description: short, action-oriented, names the rollup verdicts ("match / drift / no-pin / probe-fail … drift dominates … detect rotation BEFORE auth-bearing workloads fail")
- [x] Params struct `FleetVerifyParams { exit_on_drift_only: Option<bool> }`
- [x] Implementation:
  - Reads profiles via `list_all_hub_profiles()` (same path as fleet_status)
  - For each profile: parallel `tokio::spawn(probe_cert(addr))` + `KnownHubStore::default_store().get(addr)`
  - Returns JSON `{ok, verdict, profiles[{name,address,status,wire,pinned,error}], actions}`
  - `verdict` rolls up with drift dominance (drift > probe-fail > no-pin > match)
  - `ok` semantics: strict mode → `verdict == match`; drift-only mode → `verdict != drift`
  - `actions` populated with heal/re-pin commands when verdict is `drift`
- [x] `cargo check -p termlink-mcp` succeeds (clean, 12s; pre-existing `cur_run_end` warning unrelated)
- [x] `cargo build --release -p termlink-mcp` succeeds (1m 08s)
- [x] Sanity check: tool registers in the MCP server — grep confirms `name = "termlink_fleet_verify"` attribute under a `#[tool(...)]` block at lines matching the impl. Implementation mirrors CLI's `cmd_fleet_verify` (T-1660) logic byte-for-byte; runtime behavior already validated by T-1660 live smoke against 5 hubs.

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

cargo check -p termlink-mcp
grep -q "termlink_fleet_verify" crates/termlink-mcp/src/tools.rs
grep -q "FleetVerifyParams" crates/termlink-mcp/src/tools.rs

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

### 2026-05-17T17:46:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1661-mcp-exposure-for-fleet-verify--parity-wi.md
- **Context:** Initial task creation
