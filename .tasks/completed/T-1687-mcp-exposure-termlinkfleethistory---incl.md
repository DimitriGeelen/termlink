---
id: T-1687
name: "MCP exposure: termlink_fleet_history --include-heals — agent-callable retrospective"
description: >
  MCP exposure: termlink_fleet_history --include-heals — agent-callable retrospective

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-18T06:47:56Z
last_update: 2026-05-18T06:57:02Z
date_finished: 2026-05-18T06:57:02Z
---

# T-1687: MCP exposure: termlink_fleet_history --include-heals — agent-callable retrospective

## Context

`fleet history` (T-1671) and `--include-heals` (T-1686) let an operator answer "is this hub's drift the first or Nth time, and what did we do about it?" via the CLI. Agents currently cannot — `termlink_fleet_history` doesn't exist in the MCP surface. Mirrors the T-1661/T-1663 MCP-parity pattern that exposed earlier rotation diagnostic verbs. Pure read-only file parsing, no auth.

## Acceptance Criteria

### Agent
- [x] `termlink_fleet_history` MCP tool exists in `crates/termlink-mcp/src/tools.rs` with params `since_days` (default 7, clamped 1..=365), `hub` (optional filter), `include_heals` (default false)
- [x] Returns JSON `{ok, entries[], summary{total, per_hub_*, log_path, heal_log_path?}, hint?}` mirroring CLI `cmd_fleet_history` shape; entries tagged with `event_type: "rotation" | "heal"` and sorted chronologically
- [x] When `include_heals=true`, also reads `~/.termlink/heal.log` and merges entries; when log files don't exist, returns `entries: []` with an explanatory `hint`
- [x] Out-of-range `since_days` returns a structured error JSON (not a panic) — covered by `fleet_history_rejects_out_of_range_since`
- [x] `cargo check -p termlink` clean (rebuilds after MCP edit since CLI depends on MCP)
- [x] Smoke: 3 e2e tokio tests pin the contract with synthetic logs — `fleet_history_e2e_merges_rotation_and_heal` (3-entry chronological sort), `fleet_history_e2e_empty_state_returns_hint`, `fleet_history_rejects_out_of_range_since`; plus 6 parse-helper unit tests

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
cargo check -p termlink 2>&1 | tail -3 | grep -q "Finished"
cargo test -p termlink-mcp --lib fleet_history 2>&1 | tail -5 | grep -q "9 passed"

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

### 2026-05-18T06:47:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1687-mcp-exposure-termlinkfleethistory---incl.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-789eb036
- **Timestamp:** 2026-05-18T06:57:03Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Per-AC findings:**

- **AC#1 (Agent)** — `termlink_fleet_history` MCP tool exists in `crates/termlink-mcp/src/tools.rs` with params `since_days` (default 7, clamped 1..=365), `hub` (optional filter), `include_heals` (default false)
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-mcp/src/tools.rs in: `termlink_fleet_history` MCP tool exists in `crates/termlink-mcp/src/tools.rs` with params `since_days` (default 7, clamped 1..=365), `hub` (optional `
- **AC#3 (Agent)** — When `include_heals=true`, also reads `~/.termlink/heal.log` and merges entries; when log files don't exist, returns `entries: []` with an explanatory `hint`
  - **AC-verify-mismatch** (narrow, heuristic) — `path=termlink/heal.log in: When `include_heals=true`, also reads `~/.termlink/heal.log` and merges entries; when log files don't exist, returns `entries: []` with an explanatory`

### 2026-05-18T06:57:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
