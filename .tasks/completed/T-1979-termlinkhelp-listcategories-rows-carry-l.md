---
id: T-1979
name: "termlink_help: list_categories rows carry live_tool_count (T-1978 mirror)"
description: >
  Add live_tool_count field to each row in list_categories mode, computed as tool_count - deprecated_count. Symmetric extension of T-1978's summary additions into the per-category enumeration. LLM cold-start drilling via list_categories sees 'channel has 17 tools but 12 live' at first round-trip, without summing client-side. Same source of truth as T-1967's deprecated_count + the existing tool_count walk.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-04T07:20:38Z
last_update: 2026-06-04T07:23:07Z
date_finished: null
---

# T-1979: termlink_help: list_categories rows carry live_tool_count (T-1978 mirror)

## Context

Cycle 11 slice 4 (slices 1-3 = T-1976, T-1977, T-1978). T-1978 extended summary with `total_live_tools` + `largest_live_categories`. This slice mirrors that into list_categories rows so LLMs drilling category-by-category see per-category live counts at the same round-trip as the existing `tool_count` + `deprecated_count`. Trivial composition: `live = tool_count - deprecated_count`.

## Acceptance Criteria

### Agent
- [x] Each `list_categories` row gains `live_tool_count` field (== `tool_count` - `deprecated_count`)
- [x] Test: `list_categories_live_tool_count_matches_arithmetic` — every row's `live_tool_count` equals `tool_count - deprecated_count`
- [x] Test: `list_categories_live_tool_count_matches_walk` — independent walk of categories computing `is_deprecated()` per tool equals reported `live_tool_count`
- [x] Test: `list_categories_live_tool_count_sums_to_summary_total_live_tools` — registry-wide sum equals summary mode's `total_live_tools` (cross-mode arithmetic with T-1978)
- [x] Drift test gains required field `("live_tool_count", "T-1979")`
- [x] `cargo test --lib --package termlink-mcp` passes; new test count == 749 + 3 = 752

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
cargo test --lib --package termlink-mcp --quiet 2>&1 | tail -3 | grep -q "test result: ok"

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

### 2026-06-04T07:20:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1979-termlinkhelp-listcategories-rows-carry-l.md
- **Context:** Initial task creation

### 2026-06-04T07:21:19Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
