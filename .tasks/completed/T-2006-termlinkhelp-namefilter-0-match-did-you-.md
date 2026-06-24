---
id: T-2006
name: "termlink_help name_filter 0-match: did-you-mean suggestions over tool+category names — cycle 13 slice 4"
description: >
  termlink_help name_filter 0-match: did-you-mean suggestions over tool+category names — cycle 13 slice 4

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-05T18:34:07Z
last_update: 2026-06-05T19:57:58Z
date_finished: 2026-06-05T20:11:49Z
---

# T-2006: termlink_help name_filter 0-match: did-you-mean suggestions over tool+category names — cycle 13 slice 4

## Context

Currently `termlink_help(name_filter='chanel')` (typo for `channel`) returns:

```
{"matches": [], "total_matches": 0, "hint": "No tools matched 'chanel'. Try a different substring..."}
```

The hint is generic — operator must guess what they meant. T-1954 already added
`did_you_mean` for `tool_detail` and `category` error paths, plus
`nearest_tools` / `nearest_categories` helpers. This slice extends the same
treatment to `name_filter` 0-match envelopes.

When `name_filter` returns 0 matches AND a real needle was supplied (not the
bulk-paging zero-needle path), emit:

```
"did_you_mean": ["channel", "termlink_channel_post", "agent_chat", ...]
```

Mixed array: top-3 tool names + top-3 category names, dedup'd, sorted by
distance. Surface on both MCP (envelope field) and CLI (human render line).

## Acceptance Criteria

### Agent
- [x] `build_help_json` `name_filter` 0-match block emits `did_you_mean` array when needle is set AND results > 0
- [x] Suggestions combine `nearest_tools(needle_ref, ..., 3)` + `nearest_categories(needle_ref, ..., 3)`, deduped, capped at 6
- [x] No `did_you_mean` for the standalone-arity-filter / bulk-paging-no-needle paths (no needle to compare against)
- [x] No `did_you_mean` when matches.len() > 0 (it would be confusing alongside matches)
- [x] CLI `render_matches` surfaces did_you_mean as `Did you mean: a, b, c` line when non-empty
- [x] Shape-parity invariant test continues to pass (`build_cli_help_json_matches_mcp_shape`)
- [x] New MCP test covers typo → suggestions
- [x] `cargo build -p termlink --release` clean
- [x] `cargo test -p termlink-mcp --lib` 836+ pass
- [x] `cargo test -p termlink --bins` 814+ pass
- [x] Live smoke: `termlink help chanel --json` envelope carries `did_you_mean: ["channel", ...]`
- [x] Live smoke: `termlink help chanel` human render shows `Did you mean: channel, ...`

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
cargo build -p termlink --release 2>&1 | tail -3
cargo test -p termlink-mcp --lib 2>&1 | tail -3
target/release/termlink help chanel --json 2>&1 | grep -q did_you_mean
target/release/termlink help chanel 2>&1 | grep -q 'Did you mean'

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

### 2026-06-05T18:34:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2006-termlinkhelp-namefilter-0-match-did-you-.md
- **Context:** Initial task creation
