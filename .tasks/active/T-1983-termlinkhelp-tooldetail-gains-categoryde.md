---
id: T-1983
name: "termlink_help: tool_detail gains category_deprecated_count + category_live_tool_count"
description: >
  Add two flat fields to tool_detail: category_deprecated_count and category_live_tool_count. Completes per-mode metadata symmetry — list_categories rows now carry deprecated_count/live_tool_count (T-1967/T-1979), category=X carries category_meta (T-1981), and tool_detail already carries category_tool_count (T-1965). The remaining gap is that tool_detail does not show the retirement status of its category. This slice closes that. Same source: per-category walk of is_deprecated().

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-04T07:39:59Z
last_update: 2026-06-04T07:40:45Z
date_finished: null
---

# T-1983: termlink_help: tool_detail gains category_deprecated_count + category_live_tool_count

## Context

Cycle 11 slice 8 — completion of per-mode category-metadata symmetry. T-1965 added `category_tool_count` to tool_detail; this slice adds the retirement counterparts so tool_detail also shows the namespace's retirement status. LLMs landing on a tool in a heavily-retired category see that signal at the same round-trip.

## Acceptance Criteria

### Agent
- [ ] `tool_detail` JSON gains `category_deprecated_count` field (== count of deprecated tools in the same category)
- [ ] `tool_detail` JSON gains `category_live_tool_count` field (== `category_tool_count` - `category_deprecated_count`)
- [ ] Test: `tool_detail_category_counts_consistent_with_walk` — both fields match independent per-category walks
- [ ] Test: `tool_detail_category_counts_sum_to_tool_count` — live + deprecated == category_tool_count
- [ ] Test: `tool_detail_category_counts_match_list_categories_row` — counts match what list_categories reports for the same category
- [ ] Drift test gains 2 required fields: `category_deprecated_count`, `category_live_tool_count`
- [ ] `cargo test --lib --package termlink-mcp` passes; new test count == 761 + 3 = 764

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

### 2026-06-04T07:39:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1983-termlinkhelp-tooldetail-gains-categoryde.md
- **Context:** Initial task creation

### 2026-06-04T07:40:45Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
