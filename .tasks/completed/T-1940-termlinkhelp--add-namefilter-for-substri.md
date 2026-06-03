---
id: T-1940
name: "termlink_help — add name_filter for substring search across categories"
description: >
  When LLM consumers don't know the exact category for the tool they want, they currently get 'Unknown category' on guesses. Add a  parameter to termlink_help that returns a flat list of {category, name, description} for tools whose name OR description contains the substring (case-insensitive). Works alongside or instead of . Delivers real LLM value: 'I want to redact a post' → search 'redact' → finds termlink_channel_redact + termlink_agent_redact + redactions verbs.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T05:32:22Z
last_update: 2026-06-03T05:42:49Z
date_finished: null
---

# T-1940: termlink_help — add name_filter for substring search across categories

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `HelpParams` struct gains `name_filter: Option<String>` field with JsonSchema doc — tools.rs:6858
- [x] When `name_filter` is set, `termlink_help` returns a flat `matches` array of `{category, name, description}` for tools whose name OR description contains the filter (case-insensitive) — tools.rs build_help_json() lines 252-308
- [x] `name_filter` works alone OR combined with `category` (intersection — filter applies within the selected category) — tested by `help_name_filter_with_category`
- [x] Empty filter result returns `{matches: [], total_matches: 0}` with informative `hint` field — tested by `help_name_filter_zero_matches_gives_hint`
- [x] Tool description in `#[tool(...)]` attribute mentions the new param — tools.rs:11041
- [x] Unit test: `help_name_filter_finds_redact` confirms searching "redact" surfaces channel_redact + agent_redact + redactions verbs — tools.rs:34283
- [x] Unit test: `help_name_filter_case_insensitive` confirms "REDACT" matches the same set — tools.rs:34298
- [x] Unit test: `help_name_filter_with_category` confirms combined filter respects both constraints — tools.rs:34310
- [x] `cargo build --release -p termlink-mcp` warning-free — verified 2026-06-03 (1m46s)
- [x] `cargo test --release -p termlink-mcp` passes (no regression) — 677 passed, 0 failed (5 new + 672 pre-existing)

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
! cargo build --release -p termlink-mcp 2>&1 | grep -q "warning:"
grep -q "name_filter: Option<String>" crates/termlink-mcp/src/tools.rs
grep -q "fn help_name_filter_finds_redact" crates/termlink-mcp/src/tools.rs
grep -q "fn help_name_filter_case_insensitive" crates/termlink-mcp/src/tools.rs
grep -q "fn help_name_filter_with_category" crates/termlink-mcp/src/tools.rs
cargo test --release -p termlink-mcp --lib help_name_filter 2>&1 | grep -qE "test result: ok\."

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

### 2026-06-03T05:32:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1940-termlinkhelp--add-namefilter-for-substri.md
- **Context:** Initial task creation
