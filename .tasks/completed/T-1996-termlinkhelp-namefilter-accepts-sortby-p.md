---
id: T-1996
name: "termlink_help name_filter accepts sort_by param — deterministic ordering (cycle 12 slice 4)"
description: >
  termlink_help name_filter accepts sort_by param — deterministic ordering (cycle 12 slice 4)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T14:52:50Z
last_update: 2026-06-05T14:52:50Z
date_finished: 2026-06-05T15:09:04Z
---

# T-1996: termlink_help name_filter accepts sort_by param — deterministic ordering (cycle 12 slice 4)

## Context

Cycle-12 slice 4. Completes the paging trio (T-1984 limit, T-1994 offset, T-1995 parameter_required_count) by adding deterministic ordering. Without sort_by, the paginated window is over registry-walk order — fine for stable iteration but blind to client ranking goals (cheapest-first, alphabetical, by category). With sort_by, the LLM sorts ONCE server-side then paginates, instead of fetching every page and re-sorting client-side. Pairs especially well with parameter_required_count for cost-aware ranking.

Applies in name_filter mode only (consistent with limit/offset). Stable sort so registry order remains the tiebreak — pagination invariants hold across pages. Unknown values fall through (no sort) but surface a `sort_by_unknown` envelope field so the LLM sees its request was ignored.

## Acceptance Criteria

### Agent
- [x] HelpParams gains `sort_by: Option<String>` field with doc comment referencing T-1996
- [x] `build_help_json` signature grows 13 → 14 args (new `sort_by: Option<String>` last)
- [x] All ~95 existing call sites patched to pass `None` for the new arg (test fixtures + production)
- [x] Production caller (`call_termlink_help`) wires `let sort_by = p.sort_by.clone();`
- [x] Sort applies in name_filter branch AFTER filter loop, BEFORE offset/limit slicing
- [x] Four valid axes: `name`, `arity`, `required_arity`, `category` (stable sort, registry-order tiebreak)
- [x] Envelope emits `sort_by_applied: <value>` when sort_by provided AND valid
- [x] Envelope emits `sort_by_unknown: <value>` when sort_by provided but unrecognized (matches stay in registry order)
- [x] Drift table gains `("sort_by_applied", "T-1996")` and `("sort_by_unknown", "T-1996")`
- [x] 11 invariant tests added: by_name_alphabetical, by_arity_ascending_stable, by_required_arity_ascending_stable, by_category_alphabetical, unknown_value_emits_sort_by_unknown_and_preserves_registry_order, composes_with_limit_applied_after_sort, composes_with_offset_applied_after_sort, composes_with_limit_and_offset_yields_window_over_sorted, composes_with_exclude_deprecated, unset_omits_sort_by_applied_backcompat, stable_sort_preserves_registry_order_on_ties
- [x] `cargo test -p termlink-mcp --lib`: baseline 785 → 796 passed, 0 failed

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

### 2026-06-05T14:52:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1996-termlinkhelp-namefilter-accepts-sortby-p.md
- **Context:** Initial task creation
