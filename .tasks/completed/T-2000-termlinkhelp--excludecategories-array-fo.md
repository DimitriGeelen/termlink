---
id: T-2000
name: "termlink_help — exclude_categories array for negative multi-namespace filtering (cycle 12 slice 8)"
description: >
  termlink_help — exclude_categories array for negative multi-namespace filtering (cycle 12 slice 8)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-05T15:36:03Z
last_update: 2026-06-05T15:36:03Z
date_finished: 2026-06-05T17:03:27Z
---

# T-2000: termlink_help — exclude_categories array for negative multi-namespace filtering (cycle 12 slice 8)

## Context

Cycle-12 slice 8. T-1999 added `categories` for positive multi-namespace scoping. This slice closes the symmetric story with `exclude_categories: Option<Vec<String>>` — drops rows whose category is in the array. Use case: "show me everything EXCEPT the agent_chat/agent_inbox noise" or "exclude T-1166 retirement namespaces from registry-wide queries".

Composes with `categories` (when both set, exclude wins on overlap) and with `category` (when both set, the negative filter applies on top of the positive scope). Unknown names dropped from the filter AND surfaced via envelope `exclude_categories_unknown`; recognized ones echoed via `exclude_categories_applied`.

Applies in `name_filter` and (via T-1997) bulk-flat-listing mode.

## Acceptance Criteria

### Agent
- [x] HelpParams gains `exclude_categories: Option<Vec<String>>` field with doc comment referencing T-2000
- [x] `build_help_json` signature grows 16 → 17 args (new `exclude_categories: Option<&[String]>` last)
- [x] All callers patched to pass `None` for the new arg
- [x] Production caller (`call_termlink_help`) wires `let exclude_categories_filter = p.exclude_categories.as_deref();`
- [x] Recognized exclusion categories computed upfront against the registry; unknown ones captured separately
- [x] Per-category loop drops rows whose category is in the recognized exclusion set
- [x] When `categories` AND `exclude_categories` both set, exclusion wins on overlap (intersection-minus-exclusion semantic)
- [x] Envelope emits `exclude_categories_applied: [...]` when array is non-empty AND at least one recognized
- [x] Envelope emits `exclude_categories_unknown: [...]` when array has any unknown name
- [x] Empty `exclude_categories:[]` treated as no exclusion (omits both envelope flags)
- [x] Drift table gains `("exclude_categories_applied", "T-2000")` and `("exclude_categories_unknown", "T-2000")`
- [x] 9 invariant tests added: single_exclusion_drops_that_category, multi_exclusion_drops_union_of_categories, exclude_wins_over_categories_on_overlap, exclude_composes_with_single_category, unknown_name_surfaces_in_envelope, unset_omits_envelope_fields_backcompat, empty_array_omits_both, composes_with_limit, composes_with_bulk_flat_listing_no_needle
- [x] `cargo test -p termlink-mcp --lib`: baseline 825 → 834 passed, 0 failed

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

### 2026-06-05T15:36:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2000-termlinkhelp--excludecategories-array-fo.md
- **Context:** Initial task creation
