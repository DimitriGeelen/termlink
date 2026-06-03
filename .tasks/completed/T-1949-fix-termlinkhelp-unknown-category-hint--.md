---
id: T-1949
name: "Fix termlink_help unknown-category hint — derive list from help_categories() (drift bug, 6 missing)"
description: >
  Hard-coded category list in error path silently diverged; derive structurally

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T20:30:53Z
last_update: 2026-06-03T20:33:04Z
date_finished: null
---

# T-1949: Fix termlink_help unknown-category hint — derive list from help_categories() (drift bug, 6 missing)

## Context

`build_help_json` at `tools.rs:648` returns an error when an unknown category
is requested. The error message has a hard-coded list of 23 category names,
which has silently diverged from `help_categories()`:

Missing from the hint (added at T-1943/44/45): `channel_admin`, `channel_poll`,
`agent_engagement_metrics`, `agent_rankings`, `agent_stats`, `agent_thread_health`.

Symptom: LLM consumer requesting category `agent_stats` (which exists) but
misspelling as `agent_stat` gets an error listing the available categories —
and the list lies, omitting 6 categories the LLM might want.

Root cause is structural: the hint hard-codes the source of truth that
lives in `help_categories()`. Every new category requires a parallel manual
update in the error string. Fix: derive the hint from `help_categories()`.

## Acceptance Criteria

### Agent
- [x] Hard-coded category list in `build_help_json` unknown-category error replaced with a derivation from the `categories` slice (same source the function already walks)
  - Evidence: `crates/termlink-mcp/src/tools.rs:647-657` — `let available: Vec<&str> = categories.iter().map(|(name, _)| *name).collect();` + `available.join(", ")`. Commit `a879d58d`
- [x] Unit test: `help_unknown_category_hint_lists_all_real_categories` — verifies the error hint mentions every category present in `help_categories()`, dynamically (no hard-coded list in the test either)
  - Evidence: `crates/termlink-mcp/src/tools.rs:34472` — walks `help_categories()` and asserts each cat name appears in `err`. Test passes.
- [x] `cargo test -p termlink-mcp --lib` passes 682 (681 + 1 new)
  - Evidence: `test result: ok. 682 passed; 0 failed; 0 ignored; 0 measured` — +1 from prior 681 baseline

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
cargo test -p termlink-mcp --lib help_ -- --nocapture
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

## RCA

**Symptom:** An LLM consumer calling `termlink_help(category="<misspelled-or-unknown>")`
receives an error string listing 23 available categories. The list omits
6 categories that actually exist (`channel_admin`, `channel_poll`,
`agent_engagement_metrics`, `agent_rankings`, `agent_stats`,
`agent_thread_health`). The error misleads the LLM into thinking those
6 categories are unavailable when they are surfaceable.

**Root cause:** The error message at `tools.rs:648` hard-codes the category
list, duplicating the source of truth that lives in `help_categories()`.
T-1943/44/45 added 6 new categories to `help_categories()` but did not
update the parallel error string, because no test exercised the
unknown-category error path against the registry.

**Why structurally allowed:** The completion gate runs the verification
commands the task author writes. T-1943/44/45 verification ran
`cargo test help_` and post-coverage diff — both passed. Neither exercised
the error path because no test linked the hint to the actual registry.
The bi-directional invariant from T-1941/T-1946 covers `name` entries but
not error-message metadata.

**Prevention:** The new test `help_unknown_category_hint_lists_all_real_categories`
asserts the error message mentions every category in `help_categories()` —
dynamically, no hard-coded list. Adding a new category without updating the
hint will fail this test. The fix simultaneously eliminates the drift class
by deriving the hint from `help_categories()` at runtime, so the test is
defence-in-depth, not the only safeguard.

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

### 2026-06-03T20:30:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1949-fix-termlinkhelp-unknown-category-hint--.md
- **Context:** Initial task creation
