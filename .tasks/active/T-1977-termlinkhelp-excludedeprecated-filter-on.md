---
id: T-1977
name: "termlink_help: exclude_deprecated filter on name_filter — discovery without retirement-WIP noise"
description: >
  Add exclude_deprecated: Option<bool> to HelpParams. When true, name_filter and standalone-arity-filter modes suppress rows where deprecated==true. Composes with name_filter + min_parameters + max_parameters for clean discovery queries: 'find me live, low-arity tools matching channel'. Mirrors the T-1975/T-1976 filter pattern: same call site, same envelope shape, only the row set shrinks. Real value: LLM clients no longer fetch deprecated rows just to filter them out.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-04T07:10:05Z
last_update: 2026-06-04T07:11:07Z
date_finished: null
---

# T-1977: termlink_help: exclude_deprecated filter on name_filter — discovery without retirement-WIP noise

## Context

Cycle 11 of `termlink_help` hardening — slice 2 (slice 1 = T-1976 min_parameters). Adds `exclude_deprecated: Option<bool>` to HelpParams. LLM clients doing discovery (e.g. `name_filter="inbox"`) currently get retirement-WIP rows mixed with live alternatives; they have to client-side-filter rows where `deprecated==true`. This slice moves that filter server-side, composes with the other filters, and keeps the envelope shape stable.

## Acceptance Criteria

### Agent
- [ ] `HelpParams` gains `exclude_deprecated: Option<bool>` field
- [ ] `build_help_json` signature accepts `exclude_deprecated: bool` argument; all callers migrated
- [ ] `name_filter` branch drops rows where `is_deprecated(desc)` AND `exclude_deprecated==true`
- [ ] Standalone-arity-filter branch ALSO applies the deprecated gate (same `if`-block)
- [ ] Test: `exclude_deprecated_drops_deprecated_rows` — query `name_filter="inbox"` with `exclude_deprecated=true` returns zero rows with `deprecated==true`
- [ ] Test: `exclude_deprecated_off_keeps_deprecated_rows` — same query without flag returns at least one `deprecated==true` row (baseline)
- [ ] Test: `exclude_deprecated_composes_with_min_max` — `min=1, exclude_deprecated=true, name_filter="inbox"` returns only live AND arity≥1 rows
- [ ] Drift test gains required field `("exclude_deprecated", "T-1977")`
- [ ] `cargo test --lib --package termlink-mcp` passes; new test count == 743 + 3 = 746

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

### 2026-06-04T07:10:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1977-termlinkhelp-excludedeprecated-filter-on.md
- **Context:** Initial task creation

### 2026-06-04T07:11:07Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
