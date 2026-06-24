---
id: T-1967
name: "termlink_help list_categories rows carry deprecated_count"
description: >
  Enrich every list_categories row with deprecated_count: number — the count of deprecated tools in that category (derived from is_deprecated() on each tool's description). Composes with existing {name, tool_count, description} to complete the category-shape signal at discovery time. Drift-proof — derived live.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [mcp, help-registry]
components: []
related_tasks: []
created: 2026-06-03T22:49:06Z
last_update: 2026-06-03T22:51:39Z
date_finished: 2026-06-03T22:52:48Z
---

# T-1967: termlink_help list_categories rows carry deprecated_count

## Context

`list_categories` rows currently expose `{name, tool_count, description}` —
no retirement-debt signal at the discovery step. To see deprecated-count an
LLM has to invoke summary mode (T-1963) or scan default mode. Adding
`deprecated_count` to every list_categories row gives one-stop shape sizing:
"this category has N tools, K of them deprecated". Composes with T-1963's
`deprecated_by_category` (which is the same data at a different shape).

## Acceptance Criteria

### Agent
- [x] Every list_categories row includes `deprecated_count: number` — `crates/termlink-mcp/src/tools.rs:1101-1107` (filter+count loop) + `1112` (json key)
- [x] Value equals `tools.iter().filter(|(_,d)| is_deprecated(d)).count()` — same site, single-source derivation
- [x] Macro description updated for `list_categories` envelope — `tools.rs:12018` `{categories:[{name,tool_count,description,deprecated_count}], ...}`
- [x] Drift test (T-1964) extended with `deprecated_count` — `tools.rs:35666-35671`
- [x] Invariant test: `list_categories_rows_carry_deprecated_count` walks every row, recomputes, asserts equality — `tools.rs:35715-35759`
- [x] Invariant test: `list_categories_deprecated_sum_matches_summary` asserts cross-mode arithmetic identity (sum of per-row == summary.total_deprecated) — `tools.rs:35761-35784`
- [x] Full suite passes — 718 tests (+2 from 716), 0 failed

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
cargo test --lib --package termlink-mcp -- list_categories_rows_carry_deprecated_count 2>&1 | tail -5 | grep -q "test result: ok"
cargo test --lib --package termlink-mcp -- list_categories_deprecated_sum_matches_summary 2>&1 | tail -5 | grep -q "test result: ok"
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

### 2026-06-03T22:49:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1967-termlinkhelp-listcategories-rows-carry-d.md
- **Context:** Initial task creation

### 2026-06-03T22:49:55Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
