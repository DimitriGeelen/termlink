---
id: T-1948
name: "termlink_help — add list_categories mode for cold-start tree-walk discovery"
description: >
  New boolean param returning categories + tool counts only — for LLMs that want to drill in

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T20:26:03Z
last_update: 2026-06-03T20:26:03Z
date_finished: null
---

# T-1948: termlink_help — add list_categories mode for cold-start tree-walk discovery

## Context

`termlink_help` currently has two modes:
1. Default (no args / `category=…`) — returns full per-tool listings per category
2. `name_filter=…` (T-1940) — returns flat array of substring matches

Both return ~252 entries when scoped to "all". An LLM consumer cold-starting
without any signal about what categories exist must take the full dump into
context to begin. With 27 categories now (T-1945), an overview-first browse
pattern is missing.

This slice adds a third mode: `list_categories=true` returns only category
names + tool counts (~27 entries, kilobytes of context vs hundreds).
The LLM can then drill in via `category=<name>` to read the per-tool list.

When `list_categories=true`, `category` and `name_filter` are ignored —
no meaningful composition (overview is overview).

## Acceptance Criteria

### Agent
- [ ] `HelpParams` struct gets `list_categories: Option<bool>` field with rustdoc
- [ ] `build_help_json` branches to category-list mode when set, returning `{categories: [{name, tool_count}, ...], total_categories, total_tools}`
- [ ] `termlink_help` wrapper passes the new param through
- [ ] Top-level `termlink_help` tool description mentions the new mode so LLMs discover it
- [ ] Unit test: `help_list_categories_returns_just_counts` — verifies all 27 categories appear with correct counts and no per-tool fields
- [ ] Unit test: `help_list_categories_overrides_filters` — verifies category/name_filter args are ignored when list_categories=true
- [ ] `cargo test -p termlink-mcp --lib` passes 681 (679 + 2 new)

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

### 2026-06-03T20:26:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1948-termlinkhelp--add-listcategories-mode-fo.md
- **Context:** Initial task creation
