---
id: T-2002
name: "CLI termlink help — parity with MCP help registry (cycle 13 slice 1: pub wrapper + flat top-level subcommand)"
description: >
  CLI termlink help — parity with MCP help registry (cycle 13 slice 1: pub wrapper + flat top-level subcommand)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T17:20:29Z
last_update: 2026-06-05T17:36:56Z
date_finished: 2026-06-05T17:47:17Z
---

# T-2002: CLI termlink help — parity with MCP help registry (cycle 13 slice 1: pub wrapper + flat top-level subcommand)

## Context

Cycle 12 (T-1984..T-2000) shipped the MCP `termlink_help` paged-ranked-filtered-projected
API and cut LLM cold-start context-eat ~30× (~50KB → ~1.5KB for the canonical call).
But shell operators running `termlink help` get only clap's default usage (~3KB
auto-generated) — there is NO top-level `help` subcommand at all. The registry
shipped through cycle 12 is invisible from the shell.

This slice (cycle 13 #1) adds `termlink help` as a real top-level subcommand that
wraps the same `build_help_json` registry through a thin pub wrapper, so the JSON
shape and axis surface are identical to MCP `termlink_help` — operators get the
same answer regardless of interface. The default render is human-readable; `--json`
exposes the raw envelope for piping to `jq`.

`termlink-cli` already depends on `termlink-mcp`, so wiring is direct: pub-wrap the
private function in `tools.rs`, add `Cli::Help(HelpArgs)`, route to a `help` module.

Reuses learnings: PL-202 (cycle-12 slice recipe), PL-172 (MCP↔CLI silent-strip
prevention recipe — surface the new axis on both sides at the same time).

## Acceptance Criteria

### Agent
- [x] `pub fn build_cli_help_json(...)` exists in `termlink-mcp::tools` (or `termlink-mcp` re-export), takes the same axis surface as MCP `HelpParams` (limit/offset/sort_by/name_filter/category/categories/exclude_categories/fields/min_parameters/max_parameters/exclude_deprecated/deprecated_only/list_categories/tool_detail/summary/essentials), and walks the same internal `tool_categories()` registry — re-exported in `termlink-mcp/src/lib.rs`
- [x] CLI gains a top-level `Help(HelpArgs)` clap subcommand exposing all of the above as flags with kebab-case names — see `crates/termlink-cli/src/cli.rs::Command::Help`
- [x] `termlink help --json` returns a JSON envelope structurally identical to MCP `termlink_help(...)` for every axis combination tested — locked by `tools::tests::build_cli_help_json_matches_mcp_shape` over 7 axis combos (default/canonical-PL202/list_categories/tool_detail/summary/essentials/scope+exclude)
- [x] `termlink help` (no args) prints a human-readable categorized listing of every tool name + one-line blurb — verified live: 252 tools, 569 lines, ~21KB
- [x] `termlink help --limit 10 --sort-by required_arity --json` prints exactly 10 rows sorted by required-arity ASC — verified live with `--limit 3` returning 3 zero-arg primitives in expected order
- [x] `termlink help --name-filter channel --json` returns the same `matches[]` array as the MCP call — parity test case "canonical-pl202" covers this exact shape
- [x] `cargo build -p termlink` (the CLI crate name is `termlink`, not `termlink-cli` — original AC corrected) succeeds with zero new warnings — release build clean
- [x] `cargo test -p termlink --bins` passes (existing CLI test surface stays green) — 806 passed / 0 failed (+3 new help.rs tests)
- [x] `cargo test -p termlink-mcp --lib` passes (existing 834 MCP tests stay green; new pub-wrapper test added) — 835 passed / 0 failed (+1 parity test)
- [x] At least one new integration test in `crates/termlink-mcp/src/tools.rs` asserts `build_cli_help_json(...) == build_help_json(...)` for a representative axis combination (shape-parity invariant) — `build_cli_help_json_matches_mcp_shape` covers 7 axis combos

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
cargo build -p termlink --release 2>&1 | tail -5
cargo test -p termlink-mcp --lib 2>&1 | tail -5
target/release/termlink help --json --limit 5 --sort-by required_arity 2>&1 | python3 -c "import sys, json; d=json.load(sys.stdin); assert 'matches' in d or 'categories' in d, 'envelope missing'; print('shape: ok')"

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

### 2026-06-05T17:20:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2002-cli-termlink-help--parity-with-mcp-help-.md
- **Context:** Initial task creation
