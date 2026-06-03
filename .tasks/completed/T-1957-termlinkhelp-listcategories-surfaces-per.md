---
id: T-1957
name: "termlink_help list_categories surfaces per-category description"
description: >
  MCP client arc T-1957: list_categories mode currently returns {name, tool_count} per category. An LLM cold-discovering the registry sees 27 category names with counts but no purpose hint, forcing it to drill into each category just to read tool descriptions and infer the category's domain. Add a curated one-line description per category and surface it in list_categories output. Structural invariant test ensures the map covers every help_categories() entry — drift cannot land silently.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T21:44:57Z
last_update: 2026-06-03T21:48:35Z
date_finished: null
---

# T-1957: termlink_help list_categories surfaces per-category description

## Context

T-1948 introduced `list_categories: bool` for cold-start two-step discovery. The mode returns `{categories:[{name, tool_count}], total_categories, total_tools}`. An LLM seeing 27 category names — `agent_engagement_metrics`, `channel_admin`, `dispatch`, `tofu` — has no purpose hint without drilling in. Add a curated one-line description per category and surface it in `list_categories` output as a third field per row: `{name, tool_count, description}`. The descriptions live in a static map keyed by category name; a structural-invariant test asserts every `help_categories()` entry has an entry in the description map, so a new category cannot land without a description.

## Acceptance Criteria

### Agent
- [x] `tools.rs::category_descriptions()` (or equivalent) returns a `&'static HashMap<&'static str, &'static str>` populated with one-line purpose strings for every category present in `help_categories()`. — `crates/termlink-mcp/src/tools.rs:570-610` (OnceLock-cached, 27 entries, one per category).
- [x] `build_help_json` in `list_categories` mode includes `"description"` per row alongside `name` and `tool_count`. Existing `total_categories` and `total_tools` fields preserved. — `crates/termlink-mcp/src/tools.rs:976-996` (looks up `category_descriptions().get(cat_name)`, inserts as third field).
- [x] `HelpParams.list_categories` doc-comment and the `#[tool(description=...)]` macro for `termlink_help` mention the new `description` field in the return-shape documentation so LLMs see it via `tool_detail`. — `tools.rs:7700-7707` (HelpParams doc) and `tools.rs:11904` (macro: `{categories:[{name,tool_count,description}], ...}`).
- [x] New test `help_list_categories_includes_descriptions` asserts every row in the JSON output has a non-empty `description` field. — `tools.rs:34987-35008`, passes (1/1).
- [x] New test `category_descriptions_covers_all_categories` (structural invariant) asserts the descriptions map's key set equals the set of names from `help_categories()` — drift-detection so adding a category without a description fails CI. — `tools.rs:35010-35040` (checks both directions: missing-desc + orphan-desc), passes (1/1).
- [x] Existing tests `help_list_categories_returns_just_counts` and `help_list_categories_overrides_filters` still pass (with the description field added but other shape preserved). — 4 list_categories tests run together: all 4 pass.
- [x] Full lib test suite (`cargo test --lib --package termlink-mcp`) reports the prior pass count + the new tests, 0 failures. — `test result: ok. 698 passed; 0 failed` (+2 from 696, matches the 2 new tests).

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
cargo test --lib --package termlink-mcp help_list_categories_includes_descriptions 2>&1 | grep -q "test result: ok. 1 passed"
cargo test --lib --package termlink-mcp category_descriptions_covers_all_categories 2>&1 | grep -q "test result: ok. 1 passed"
cargo test --lib --package termlink-mcp help_list_categories 2>&1 | grep -qE "test result: ok\. [3-9] passed"

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

### 2026-06-03T21:44:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1957-termlinkhelp-listcategories-surfaces-per.md
- **Context:** Initial task creation
