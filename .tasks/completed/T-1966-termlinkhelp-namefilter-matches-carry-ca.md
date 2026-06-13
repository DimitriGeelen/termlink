---
id: T-1966
name: "termlink_help name_filter matches carry category_tool_count"
description: >
  Enrich each name_filter match row with category_tool_count (size of the match's category) so an LLM ranking search results sees the namespace bound — a match in an 8-tool category is easier to learn than one in a 40-tool category. Composes with the existing {category, name, description, deprecated} shape. Drift-proof — derived live from help_categories().

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [mcp, help-registry]
components: []
related_tasks: []
created: 2026-06-03T22:45:23Z
last_update: 2026-06-03T22:48:06Z
date_finished: 2026-06-03T22:49:55Z
---

# T-1966: termlink_help name_filter matches carry category_tool_count

## Context

T-1965 added `category_description` + `category_tool_count` to `tool_detail`
so drilling into one tool gives the LLM category-context for free. The
`name_filter` (search) path doesn't yet carry the same signal. An LLM
searching for `"post"` gets 6+ matches across `agent_chat`, `channel`,
`agent_engagement_metrics`, etc. Without sibling-count per match, ranking is
guesswork. Adding `category_tool_count` to each match row lets the LLM
prefer matches in smaller categories (tighter namespace, easier to learn).

## Acceptance Criteria

### Agent
- [x] Every name_filter match row includes `category_tool_count: number` — `crates/termlink-mcp/src/tools.rs:1118-1144` (search loop) + `1127` capture
- [x] Value equals the live `tools.len()` of the match's category — `cat_size = tools.len()` captured per-category at `tools.rs:1124`, attached to every match row
- [x] Macro description updated for `name_filter` envelope — `tools.rs:12018` `{matches:[{category,category_tool_count,name,description,deprecated}], total_matches}`
- [x] Drift test (T-1964) already covers `category_tool_count` per T-1965 addition — `tools.rs:35664` confirmed
- [x] Invariant test: `name_filter_match_rows_carry_category_tool_count` walks every category, searches by first tool, verifies match's `category_tool_count` matches `tools.len()` — `tools.rs:35715-35759`
- [x] Full suite passes — 716 tests (+1 from 715), 0 failed

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
cargo test --lib --package termlink-mcp -- name_filter_match_rows_carry_category_tool_count 2>&1 | tail -5 | grep -q "test result: ok"
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

### 2026-06-03T22:45:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1966-termlinkhelp-namefilter-matches-carry-ca.md
- **Context:** Initial task creation

### 2026-06-03T22:46:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
