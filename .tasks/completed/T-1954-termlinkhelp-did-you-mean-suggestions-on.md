---
id: T-1954
name: "termlink_help: did-you-mean suggestions on unknown tool / unknown category"
description: >
  When tool_detail or category receives an unknown name, return a did_you_mean array of nearest matches (substring overlap + Levenshtein distance). Closes the self-correcting-error gap: LLM consumers fix typos and near-misses in one round-trip instead of falling back to list_categories or name_filter. Tiny slice, high LLM ergonomics gain. T-1953 follow-up.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-03T21:15:54Z
last_update: 2026-06-03T21:20:55Z
date_finished: 2026-06-03T21:23:29Z
---

# T-1954: termlink_help: did-you-mean suggestions on unknown tool / unknown category

## Context

Current `tool_detail` and category error paths give a generic hint:
"Unknown tool 'X'. Use `list_categories=true` to browse..." — forcing the LLM
to do a second call. Adding a `did_you_mean: [...]` array of nearest matches
makes the error self-correcting in one round-trip. Same pattern for
`category=<unknown>`. Source-of-truth derivation (no hard-coded lists).

## Acceptance Criteria

### Agent
- [x] New helpers added to tools.rs: `levenshtein()`, `tool_distance_score()` (with semantic-name substring boost — strips `termlink_` prefix before containment check), `nearest_tools()`, `nearest_categories()`. Implementation: tools.rs:749-820.
- [x] `tool_detail` unknown-tool error path returns JSON `{error, did_you_mean: [...]}` (up to 5 entries). Implementation: tools.rs:842-852.
- [x] `category=<unknown>` error path returns JSON `{error, did_you_mean: [...]}`. Implementation: tools.rs:899-913.
- [x] Unit test `tool_detail_unknown_includes_suggestions` — `termlink_post` yields both `termlink_agent_post` and `termlink_channel_post` in did_you_mean (pure Levenshtein would have favored `termlink_ping` instead — substring boost validated). tools.rs:35028-35051.
- [x] Unit test `tool_detail_typo_includes_suggestions` — `termlink_agent_recents` yields `termlink_agent_recent` (single-char typo, edit distance 1). tools.rs:35053-35069.
- [x] Unit test `category_unknown_includes_suggestions` — `chanel` yields at least one `channel*` category. tools.rs:35071-35087.
- [x] `cargo test -p termlink-mcp --lib` passes — 691 total, 0 failed (688 prior + 3 new = exact +3 delta).

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

### 2026-06-03T21:15:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1954-termlinkhelp-did-you-mean-suggestions-on.md
- **Context:** Initial task creation
