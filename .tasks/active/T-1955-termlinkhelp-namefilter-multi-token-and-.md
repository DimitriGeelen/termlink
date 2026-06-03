---
id: T-1955
name: "termlink_help: name_filter multi-token AND search (intent-based discovery)"
description: >
  Split name_filter on whitespace into tokens; match tools whose name+description contains EVERY token (case-insensitive, any order). Single-token behavior unchanged. Closes the intent-search gap: LLMs querying 'send message' or 'agent post' get conjunctive results instead of empty/spurious matches. T-1954 follow-up.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T21:22:40Z
last_update: 2026-06-03T21:22:40Z
date_finished: null
---

# T-1955: termlink_help: name_filter multi-token AND search (intent-based discovery)

## Context

`name_filter` currently does exact substring search over the concatenation
of name + description. An LLM querying by intent (e.g., `"send message"`,
`"agent post"`, `"pin reaction"`) gets empty/poor results because the
exact phrase rarely appears verbatim. Splitting on whitespace and requiring
ALL tokens to match (any order, case-insensitive) closes the intent-search
gap. Single-token queries are unchanged.

## Acceptance Criteria

### Agent
- [ ] `build_help_json` `name_filter` branch tokenizes on whitespace (`split_whitespace`); matches when EVERY non-empty token appears in `name.to_lowercase()` OR `desc.to_lowercase()` (logical AND across tokens, OR within name+desc per token).
- [ ] Single-token behavior is unchanged: existing tests (`help_filter_substring_*`, `help_filter_category_scope_*`) continue to pass without modification.
- [ ] Unit test `name_filter_multi_token_and_matches_combined` — `name_filter="agent post"`, asserts that `termlink_agent_post` appears in results (both tokens present in name).
- [ ] Unit test `name_filter_multi_token_and_drops_partial_matches` — `name_filter="agent zzz"` (one valid token + one nonsense token), asserts `total_matches=0` (the nonsense token blocks all results that the bare "agent" would have matched).
- [ ] Unit test `name_filter_multi_token_distributes_across_name_and_desc` — `name_filter="pin react"`, asserts at least one tool returns where one token matches the name and the other matches the description (e.g., `termlink_agent_pin` whose desc mentions reactions, or `termlink_agent_reaction*` whose desc mentions pin).
- [ ] `cargo test -p termlink-mcp --lib` passes — 694 total, 0 failed (691 prior + 3 new).

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
cargo check -p termlink-mcp --tests
cargo test -p termlink-mcp --lib --quiet 2>&1 | tail -5 | grep -E "test result.*ok"

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

### 2026-06-03T21:22:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1955-termlinkhelp-namefilter-multi-token-and-.md
- **Context:** Initial task creation
