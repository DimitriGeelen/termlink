---
id: T-1965
name: "termlink_help tool_detail enriches with category_description + category_tool_count"
description: >
  Extend tool_detail return envelope with category-context fields: category_description (from T-1957 category_descriptions()) and category_tool_count (size of the target's category). Drift-proof — both derived live. Gives LLMs drilling into one tool the sizing + semantic context of its category without a second round-trip.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [mcp, help-registry]
components: []
related_tasks: []
created: 2026-06-03T22:41:18Z
last_update: 2026-06-03T22:44:08Z
date_finished: 2026-06-03T22:46:24Z
---

# T-1965: termlink_help tool_detail enriches with category_description + category_tool_count

## Context

`tool_detail` mode currently returns `{tool, name, category, short_description,
full_description, parameters, related_tools, deprecated, verb_cognates?}`. The
`category` field is just the name — the LLM gets no signal about category SIZE
or PURPOSE without a second `list_categories` round-trip. Adding both:
`category_description` (T-1957 source) and `category_tool_count` (registry-derived)
closes that gap. Composes with `related_tools` (which caps at 10) so when the
category is large (40+ tools) the LLM knows to browse siblings deeper.

## Acceptance Criteria

### Agent
- [x] `tool_detail` return envelope includes `category_description: string` — `crates/termlink-mcp/src/tools.rs:1028-1042`
- [x] `tool_detail` return envelope includes `category_tool_count: number` — same site, sourced from `found_cat_size` captured during the registry walk at `tools.rs:990-1000`
- [x] Both fields are present on EVERY tool_detail success response — locked by `tool_detail_carries_category_context_for_every_tool` (walks every tool in `help_categories()`)
- [x] Macro description updated with both new fields — `tools.rs:12018` `tool_detail returns {...category_description, category_tool_count...}`
- [x] Drift test (T-1964) extended with `category_description` and `category_tool_count` — `tools.rs:35658-35664`
- [x] Invariant test: `category_tool_count` matches category size for every tool — `tool_detail_carries_category_context_for_every_tool` at `tools.rs:35715-35761`
- [x] Invariant test: `category_description` non-empty for every tool's category — same test, fails on any empty string
- [x] Full suite passes — 715 tests (+1 from 714), 0 failed

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
cargo test --lib --package termlink-mcp -- tool_detail_carries_category_context 2>&1 | tail -5 | grep -q "test result: ok"
cargo test --lib --package termlink-mcp -- help_macro_description 2>&1 | tail -5 | grep -q "1 passed"
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

### 2026-06-03T22:41:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1965-termlinkhelp-tooldetail-enriches-with-ca.md
- **Context:** Initial task creation

### 2026-06-03T22:42:15Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
