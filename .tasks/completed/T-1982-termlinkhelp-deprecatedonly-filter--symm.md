---
id: T-1982
name: "termlink_help: deprecated_only filter — symmetric inverse of exclude_deprecated"
description: >
  Add deprecated_only: Option<bool> to HelpParams. When true on name_filter or standalone-arity-filter, suppresses live rows (keeps only deprecated). Symmetric inverse of T-1977's exclude_deprecated. LLMs can query 'show me deprecated channel tools' for migration planning in one round-trip. Mutex with exclude_deprecated — if both set true, the response is empty (their intersection is the empty set) and the hint notes it. Same envelope shape.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-04T07:32:24Z
last_update: 2026-06-04T07:39:12Z
date_finished: 2026-06-04T07:40:46Z
---

# T-1982: termlink_help: deprecated_only filter — symmetric inverse of exclude_deprecated

## Context

Cycle 11 slice 7 — the symmetric inverse of T-1977's `exclude_deprecated`. LLMs migrating off T-1166 inbox primitives can query "show me deprecated tools matching X" in one round-trip instead of fetching all + client-side filtering. Composes with the arity bounds (T-1975/T-1976), the name_filter needle, and the category scope.

## Acceptance Criteria

### Agent
- [x] `HelpParams` gains `deprecated_only: Option<bool>` field
- [x] `build_help_json` signature accepts `deprecated_only: bool` argument; all callers migrated
- [x] `name_filter` / standalone-arity-filter branch drops rows where `!is_deprecated(desc)` AND `deprecated_only==true`
- [x] When both `exclude_deprecated` AND `deprecated_only` are true, result is empty + hint mentions the conflict
- [x] Test: `deprecated_only_keeps_only_deprecated_rows` — `name_filter="inbox", deprecated_only=true` returns rows ALL marked `deprecated: true`
- [x] Test: `deprecated_only_with_exclude_deprecated_is_empty_with_hint` — both flags true yields empty matches array and a hint mentioning the conflict
- [x] Test: `deprecated_only_composes_with_min_parameters` — `min=1, deprecated_only=true, name_filter="inbox"` returns deprecated AND arity>=1 rows
- [x] Drift test gains required field `("deprecated_only", "T-1982")`
- [x] `cargo test --lib --package termlink-mcp` passes; new test count == 758 + 3 = 761

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

### 2026-06-04T07:32:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1982-termlinkhelp-deprecatedonly-filter--symm.md
- **Context:** Initial task creation

### 2026-06-04T07:33:10Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
