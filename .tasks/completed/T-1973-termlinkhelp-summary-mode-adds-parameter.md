---
id: T-1973
name: "termlink_help: summary mode adds parameter-count aggregates (cycle 10 slice 4)"
description: >
  MCP arc cycle 10 slice 4: extend the summary-mode aggregate envelope (T-1963) with parameter-count rollups composing T-1971/T-1972. Adds total_parameters, zero_arity_tools count, and highest_arity_tools (top 5). All derived live from tool_params().

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-04T05:56:06Z
last_update: 2026-06-04T05:58:42Z
date_finished: 2026-06-04T06:00:52Z
---

# T-1973: termlink_help: summary mode adds parameter-count aggregates (cycle 10 slice 4)

## Context

MCP arc cycle 10 slice 4. T-1963 introduced `summary` mode (aggregate registry stats); T-1971+T-1972 added `parameter_count` to per-tool envelopes. This slice composes them: surface registry-wide arity aggregates so an LLM sizing the registry sees the complexity landscape in one round-trip. Three new fields: `total_parameters` (sum across all tools), `zero_arity_tools` (count — these are the canonical no-config primitives), `highest_arity_tools` (top 5 by arity for cold-start risk-awareness).

## Acceptance Criteria

### Agent
- [x] `summary` envelope adds `total_parameters` field — integer, sum of `tool_params()[name].len()` over every tool (tools.rs:1212-1227, accumulated during the registry walk)
- [x] `summary` envelope adds `zero_arity_tools` field — integer, count of tools whose `tool_params()` entry is None or empty (tools.rs:1219-1222, counted alongside total)
- [x] `summary` envelope adds `highest_arity_tools` field — array of `{name, parameter_count}` rows, top 5 by parameter_count, ties broken by name asc for determinism (tools.rs:1228-1232)
- [x] Macro doc-string shape for summary mode updated to include the three new fields (tools.rs:12215, T-1973 block appended to summary line)
- [x] Drift test extended: `total_parameters`, `zero_arity_tools`, `highest_arity_tools` added to required_fields (tools.rs:35831-35836)
- [x] Invariant test: `total_parameters` equals the sum of per-tool `tool_detail.parameter_count` over the full registry (tools.rs:36694-36715 help_summary_total_parameters_matches_sum_over_tool_detail)
- [x] Invariant test: `zero_arity_tools` equals the count of tools whose `tool_detail.parameter_count == 0` (tools.rs:36717-36738 help_summary_zero_arity_tools_matches_walk_count)
- [x] Invariant test: `highest_arity_tools` rows are sorted descending by parameter_count, capped at 5, and each row's count matches the registry (tools.rs:36740-36789 help_summary_highest_arity_tools_ranked_and_real)
- [x] `cargo test --lib --package termlink-mcp` passes (736 tests, 0 failed; +3 from T-1972 baseline of 733)

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
cargo test --lib --package termlink-mcp --quiet 2>&1 | tail -5 | grep -q "test result: ok"

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

### 2026-06-04T05:56:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1973-termlinkhelp-summary-mode-adds-parameter.md
- **Context:** Initial task creation

### 2026-06-04T05:56:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
