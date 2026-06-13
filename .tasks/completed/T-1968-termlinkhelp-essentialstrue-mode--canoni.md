---
id: T-1968
name: "termlink_help essentials=true mode — canonical entry-point per category"
description: >
  Add essentials=true mode to termlink_help returning the first non-deprecated tool of each category as a flat list. Auto-derived from help_categories() registry order — drift-proof. Gives MCP clients a focused starter set (~27 tools out of 252) for cold-start learning without curating an essentials list manually.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [mcp, help-registry]
components: []
related_tasks: []
created: 2026-06-03T23:01:48Z
last_update: 2026-06-03T23:08:47Z
date_finished: 2026-06-03T23:10:27Z
---

# T-1968: termlink_help essentials=true mode — canonical entry-point per category

## Context

The full registry is 252 tools across 27 categories. An LLM landing fresh
needs a focused starter set — not the whole catalog. Curating an
"essentials" list by hand is drift-prone. But `help_categories()` already
encodes the canonical ordering: the FIRST tool of each category is the
fundamental entry point by author convention (e.g. session →
`termlink_list_sessions`, channel → `termlink_channel_create`, fleet →
`termlink_fleet_verify`). Skipping deprecated tools gives a clean ~27-tool
starter set auto-derived from registry order.

## Acceptance Criteria

### Agent
- [x] `HelpParams.essentials: Option<bool>` field added with struct doc — `crates/termlink-mcp/src/tools.rs:7837-7846`
- [x] `build_help_json` handles `essentials=true` returning `{essentials: [{name, category, description}], total}` — `tools.rs:1070-1087`
- [x] Each entry is the FIRST non-deprecated tool of its category — `tools.rs:1074` uses `tools.iter().find(|(_, d)| !is_deprecated(d))`
- [x] Categories whose tools are ALL deprecated are skipped — `find()` returns None → category contributes no row; verified by `help_essentials_skips_all_deprecated_categories`
- [x] `essentials` mode in precedence order (essentials > summary > list_categories > name_filter, below tool_detail) — `tools.rs:1070` branch placed before summary branch; verified by `help_essentials_takes_precedence_over_summary_and_lower`
- [x] Macro description updated for `essentials` mode + return shape — `tools.rs:12028` "Six modes: ... (6) essentials=true returns ..."
- [x] Drift test (T-1964) extended with `essentials` — `tools.rs:35741-35742`
- [x] Invariant test: no entry has `deprecated=true` — `help_essentials_picks_one_non_deprecated_per_category` walks every emitted row and asserts `!is_deprecated(desc)` (`tools.rs:35810-35850`)
- [x] Invariant test: each category contributes ≤1 entry — same test uses `seen_cats.insert()` and asserts the insertion was new
- [x] Invariant test: every entry's name is a real tool — both `help_essentials_picks_one_non_deprecated_per_category` AND `help_essentials_first_non_deprecated_in_registry_order` cross-look-up the registry by name and panic on miss
- [x] Full suite passes — 722 tests (+4 from 718), 0 failed

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
cargo test --lib --package termlink-mcp -- help_essentials 2>&1 | tail -5 | grep -q "test result: ok"
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

### 2026-06-03T23:01:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1968-termlinkhelp-essentialstrue-mode--canoni.md
- **Context:** Initial task creation

### 2026-06-03T23:02:37Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
