---
id: T-1999
name: "termlink_help — category accepts array for multi-namespace scoping (cycle 12 slice 7)"
description: >
  termlink_help — category accepts array for multi-namespace scoping (cycle 12 slice 7)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-05T15:29:11Z
last_update: 2026-06-05T15:29:11Z
date_finished: 2026-06-05T15:40:17Z
---

# T-1999: termlink_help — category accepts array for multi-namespace scoping (cycle 12 slice 7)

## Context

Cycle-12 slice 7. The existing `category` param is single-string — multi-namespace scoping requires N round-trips or post-filtering. This slice adds a separate `categories: Option<Vec<String>>` param so an LLM client can say "show me cheap tools in {channel, agent_chat, agent_inbox}" in one call.

Additive (non-breaking) — `category` semantics unchanged. When `categories` is set (non-empty), it takes precedence over single `category` and filters to rows whose category is in the array. Unknown category names dropped from the filter AND surfaced via envelope `categories_unknown` (same silently-ignored-input pattern as sort_by_unknown / fields_unknown).

Applies in `name_filter` mode and (via T-1997) bulk-flat-listing mode. Other modes unaffected. Empty `categories:[]` is degenerate and treated as no multi-filter (degrades to single `category` if set, otherwise no scope).

## Acceptance Criteria

### Agent
- [x] HelpParams gains `categories: Option<Vec<String>>` field with doc comment referencing T-1999
- [x] `build_help_json` signature grows 15 → 16 args (new `categories: Option<&[String]>` last)
- [x] All callers patched to pass `None` for the new arg
- [x] Production caller (`call_termlink_help`) wires `let categories = p.categories.as_deref();`
- [x] Recognized categories computed upfront against the registry's category names; unknown ones captured separately
- [x] When `categories` is `Some(non-empty)` and at least one recognized, that set takes precedence over single `category` (which is ignored)
- [x] Per-category loop checks membership in the recognized set
- [x] Envelope emits `categories_applied: [...]` (recognized subset) when array is non-empty AND at least one recognized
- [x] Envelope emits `categories_unknown: [...]` (silently-dropped names) when array has any unknown name
- [x] Empty `categories:[]` treated as no projection (omits both flags; falls back to single `category` if set)
- [x] Drift table gains `("categories_applied", "T-1999")` and `("categories_unknown", "T-1999")`
- [x] 10 invariant tests added: single_category_array_matches_legacy_behavior, multi_category_returns_union, categories_precedence_over_single_category, unknown_category_name_surfaces_in_envelope, mixed_known_unknown_filters_to_known_only, unset_omits_envelope_fields_backcompat, empty_array_treated_as_no_scope, composes_with_limit, composes_with_sort_by, composes_with_bulk_flat_listing_no_needle
- [x] `cargo test -p termlink-mcp --lib`: baseline 815 → 825 passed, 0 failed

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

### 2026-06-05T15:29:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1999-termlinkhelp--category-accepts-array-for.md
- **Context:** Initial task creation
