---
id: T-1997
name: "termlink_help — limit/offset/sort_by route into flat-list branch on no-needle calls (cycle 12 slice 5)"
description: >
  termlink_help — limit/offset/sort_by route into flat-list branch on no-needle calls (cycle 12 slice 5)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T15:05:21Z
last_update: 2026-06-05T15:05:21Z
date_finished: 2026-06-05T15:16:15Z
---

# T-1997: termlink_help — limit/offset/sort_by route into flat-list branch on no-needle calls (cycle 12 slice 5)

## Context

Cycle-12 slice 5. T-1984/T-1994/T-1995/T-1996 gave `name_filter` mode a full paged-ranked API but the no-needle path (cold-start `termlink_help()` with no args) still dumps every tool in the registry. That's the actual largest single-call context-eat — LLM clients almost always issue the no-arg call first to discover what exists.

This slice routes `limit`, `offset`, `sort_by` (and the existing `min_parameters`/`max_parameters` arity bounds, already wired) through a unified "bulk-flat-listing" trigger. When ANY of those is set on a no-needle call, the response collapses into the same `matches[]` shape as `name_filter` and runs through the same filter → sort → paginate pipeline. When NONE are set, the legacy category-keyed envelope is preserved exactly (backcompat).

Net effect: `termlink_help(limit=10)` now returns the first 10 registry-walk tools (instead of all 200+). `termlink_help(sort_by='required_arity', limit=10, exclude_deprecated=True)` returns the 10 cheapest-to-call live tools across the entire registry.

## Acceptance Criteria

### Agent
- [x] New variable `bulk_flat_listing_no_needle` triggers on no-needle calls when any of `limit`, `offset`, `sort_by`, `min_parameters`, `max_parameters` is set (folds the existing `standalone_arity_filter` semantic in)
- [x] Route trigger updated to use the new variable; `termlink_help()` with no paging args still routes to legacy default branch
- [x] `termlink_help(limit=10)` returns flat `matches[]` of 10 registry-walk rows with `total_matched` = full registry size and `limit_applied:true`
- [x] `termlink_help(limit=10, offset=20)` returns rows 20..30 with `offset:20, next_offset:30`
- [x] `termlink_help(sort_by='required_arity', limit=10)` returns 10 cheapest-to-call tools registry-wide
- [x] `termlink_help(category='channel', limit=5)` scopes to category, paginates, emits `total_matched` = total channel tools
- [x] Empty-result hint adapts when pure pagination triggered (e.g. category with no tools) and matches==0
- [x] Default no-arg `termlink_help()` envelope shape UNCHANGED (categories-keyed object + total_tools) — backcompat verified by existing tests staying green
- [x] 8 new invariant tests added: bare_limit_routes_to_flat, bare_offset_routes_to_flat, bare_sort_by_routes_to_flat, category_with_limit_paginates_namespace, sort_by_with_limit_yields_top_N_registrywide, no_paging_args_preserves_categories_keyed_envelope_backcompat, paging_envelope_omits_categories_keys, exclude_deprecated_composes_with_bare_limit
- [x] `cargo test -p termlink-mcp --lib`: baseline 796 → 804 passed, 0 failed

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

cd /opt/termlink && cargo test -p termlink-mcp --lib 2>&1 | tail -3 | grep -q "test result: ok"

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

### 2026-06-05T15:05:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1997-termlinkhelp--limitoffsetsortby-route-in.md
- **Context:** Initial task creation
