---
id: T-1964
name: "termlink_help macro drift test covers T-1963 summary fields"
description: >
  Extend the T-1962 drift-detection test to require the termlink_help macro description mentions T-1963's new fields: 'summary', 'total_deprecated', 'largest_categories', 'smallest_categories', 'deprecated_by_category'. Locks the schema-doc contract so future return-shape additions cannot ship without macro-text updates.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [mcp, help-registry]
components: []
related_tasks: []
created: 2026-06-03T22:38:36Z
last_update: 2026-06-03T22:38:36Z
date_finished: null
---

# T-1964: termlink_help macro drift test covers T-1963 summary fields

## Context

T-1962 ships a drift-detection test that asserts the `termlink_help`
`#[tool(description=...)]` macro mentions specific return fields. Each cycle
that adds new return-shape fields should extend the required-fields list so
a future regression where the macro description loses or omits a documented
field surfaces in CI, not at consumer-discovery time. T-1963 added `summary`
mode + 5 new fields. Lock them in.

## Acceptance Criteria

### Agent
- [ ] `help_macro_description_documents_post_t1953_fields` test extended with: `summary`, `total_deprecated`, `largest_categories`, `smallest_categories`, `deprecated_by_category`
- [ ] Each new entry tagged with `T-1963` ticket reference for traceability
- [ ] Test passes against current macro text (proves the T-1963 macro update was complete)
- [ ] `cargo test --lib --package termlink-mcp -- help_macro_description` reports `1 passed`
- [ ] Full suite passes — 714+ tests, 0 failed

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
cargo test --lib --package termlink-mcp -- help_macro_description 2>&1 | tail -10 | grep -q "1 passed"
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

### 2026-06-03T22:38:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1964-termlinkhelp-macro-drift-test-covers-t-1.md
- **Context:** Initial task creation
