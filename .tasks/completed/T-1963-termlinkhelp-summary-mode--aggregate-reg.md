---
id: T-1963
name: "termlink_help summary mode — aggregate registry stats"
description: >
  Add summary=true mode to termlink_help returning aggregate registry stats: total_tools, total_categories, total_deprecated, deprecated_by_category (non-zero only), largest_categories (top 5), smallest_categories (bottom 5). Drift-proof — all derived from help_categories() + is_deprecated(). Gives MCP-client consumers an O(1) cold-start snapshot of the API surface shape.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [mcp, help-registry]
components: []
related_tasks: []
created: 2026-06-03T22:30:37Z
last_update: 2026-06-03T22:36:54Z
date_finished: null
---

# T-1963: termlink_help summary mode — aggregate registry stats

## Context

Cycle 8 of the MCP-arc help-registry hardening. Cycle 7 closed at T-1962 (macro
description documents post-T-1953 fields). `termlink_help` now has 4 modes
(default / name_filter / list_categories / tool_detail) but no aggregate-shape
view — an LLM cold-discovering the server has to enumerate `list_categories`
+ N category-drill-ins to learn "this is a 252-tool / 27-category / 5-deprecated
server, biggest categories are X and Y". Adds `summary=true` for an O(1)
cold-start snapshot. Drift-proof: all numbers derived live from
`help_categories()` + `is_deprecated()`.

## Acceptance Criteria

### Agent
- [x] `HelpParams.summary: Option<bool>` field added, documented in struct doc — `crates/termlink-mcp/src/tools.rs:7825-7836`
- [x] `build_help_json` recognizes `summary=true` and returns `{total_tools, total_categories, total_deprecated, deprecated_by_category, largest_categories, smallest_categories}` — `tools.rs:1070-1118`
- [x] `deprecated_by_category` emits only non-zero entries (BTreeMap keyed by category name → alpha-sorted output) — `tools.rs:1083-1086`
- [x] `largest_categories` and `smallest_categories` are arrays of `{name, tool_count}` sorted descending and ascending respectively, capped at 5 — `tools.rs:1093-1108`
- [x] `summary` takes precedence over `list_categories` / `name_filter` / `category` (branch placed before `list_categories` and `name_filter`) — `tools.rs:1070`
- [x] Invariant test: `help_summary_totals_match_registry` — `tools.rs:35714-35734`
- [x] Invariant test: `help_summary_deprecated_arithmetic_consistent` — `tools.rs:35736-35774`
- [x] Invariant test: `help_summary_largest_smallest_are_real_categories` — `tools.rs:35776-35821`
- [x] `cargo test --lib --package termlink-mcp` passes — 714 tests (+4 from 710 baseline)

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
cargo test --lib --package termlink-mcp -- help_summary 2>&1 | tail -20 | grep -q "test result: ok"
! cargo test --lib --package termlink-mcp 2>&1 | tail -5 | grep -q "FAILED"

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

### 2026-06-03T22:30:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1963-termlinkhelp-summary-mode--aggregate-reg.md
- **Context:** Initial task creation

### 2026-06-03T22:31:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
