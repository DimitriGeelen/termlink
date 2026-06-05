---
id: T-2004
name: "termlink help target: positional arg routes to tool_detail (exact) or name_filter (substring) — cycle 13 slice 2 ergonomics"
description: >
  termlink help target: positional arg routes to tool_detail (exact) or name_filter (substring) — cycle 13 slice 2 ergonomics

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T17:48:32Z
last_update: 2026-06-05T18:01:08Z
date_finished: null
---

# T-2004: termlink help target: positional arg routes to tool_detail (exact) or name_filter (substring) — cycle 13 slice 2 ergonomics

## Context

T-2002 shipped `termlink help` with the full MCP axis surface as flags. The
single biggest ergonomic gap is the absence of a positional shortcut: operators
have to type `termlink help --name-filter channel` for what would naturally be
`termlink help channel`, and `termlink help --tool-detail termlink_channel_post`
for `termlink help termlink_channel_post`.

This slice (cycle 13 #2) adds a positional `<target>` arg that routes:
1. **Exact tool name match** → behaves as `--tool-detail <target>` (drill-in)
2. **Anything else** → behaves as `--name-filter <target>` (substring search)

## Acceptance Criteria

### Agent
- [x] `Command::Help` clap variant gains an optional positional `target: Option<String>` — `cli.rs::Command::Help.target`
- [x] When `target` is exactly one of the registry's tool names → CLI passes `Some(target)` to `tool_detail` — `commands/help.rs::resolve_positional` returns `Drilled`
- [x] When `target` is set but NOT a known tool name → CLI passes `Some(target)` to `name_filter` — returns `Filtered`
- [x] If `target` AND explicit `--tool-detail` or `--name-filter` both set → exit with stderr error + usage hint — `process::exit(2)` with PL-151-style message
- [x] Unit tests cover all paths: `positional_exact_tool_routes_to_tool_detail`, `positional_non_tool_routes_to_name_filter`, `positional_with_explicit_tool_detail_errors`, `positional_with_explicit_name_filter_errors`, `no_positional_is_inactive` (5 tests)
- [x] New `pub fn registry_tool_names() -> &'static HashSet<&'static str>` in `termlink-mcp` — `OnceLock`-cached, re-exported via `lib.rs`
- [x] `cargo build -p termlink --release` clean — finished `release` profile
- [x] `cargo test -p termlink --bins` 810 pass (805 baseline + 5 new); 1 pre-existing `isolate_rejects_non_git_dir` flake unrelated (passes in isolation, also flaked during T-2002 close)
- [x] `cargo test -p termlink-mcp --lib` 835 pass / 0 failed
- [x] Live smoke: `termlink help channel` → 60 substring matches with categorized + arity tags rendered
- [x] Live smoke: `termlink help termlink_channel_post` → full tool_detail render (category, short desc, full desc, parameters)

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
target/release/termlink help channel >/dev/null 2>&1
target/release/termlink help termlink_channel_post >/dev/null 2>&1

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

### 2026-06-05T17:48:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2004-termlink-help-target-positional-arg-rout.md
- **Context:** Initial task creation
