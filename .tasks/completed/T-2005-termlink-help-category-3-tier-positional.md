---
id: T-2005
name: "termlink help <category>: 3-tier positional routing (exact tool > exact category > substring) — cycle 13 slice 3"
description: >
  termlink help <category>: 3-tier positional routing (exact tool > exact category > substring) — cycle 13 slice 3

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-05T18:17:56Z
last_update: 2026-06-05T18:26:56Z
date_finished: 2026-06-05T18:34:44Z
---

# T-2005: termlink help <category>: 3-tier positional routing (exact tool > exact category > substring) — cycle 13 slice 3

## Context

T-2004 added 2-tier positional routing (exact tool → tool_detail, else → name_filter).
But `termlink help channel` currently does substring search and returns 60+ noisy matches
that include any tool mentioning "channel" in its description. Operators who type
`termlink help channel` almost certainly want the channel category specifically — that's
the intuitive shell idiom.

This slice (cycle 13 #3) adds a middle tier so positional routing checks 3 priorities:

1. **Exact tool name** (e.g. `termlink_channel_post`) → `tool_detail` (drill-in)
2. **Exact category name** (e.g. `channel`, `agent_chat`, `fleet`) → `category` (just that namespace, dense)
3. **Anything else** → `name_filter` (substring search across names + descriptions)

A new `pub fn registry_category_names() -> &'static HashSet<&'static str>` mirrors
the T-2004 helper so the dispatcher can check membership cheaply.

## Acceptance Criteria

### Agent
- [x] `pub fn registry_category_names() -> &'static HashSet<&'static str>` in `termlink-mcp::tools` — OnceLock-cached, walks `help_categories()` once
- [x] `commands/help.rs::PositionalRoute` gains `Scoped(String)` variant
- [x] `resolve_positional` checks priority: exact tool → `Drilled`, exact category → `Scoped`, else → `Filtered`
- [x] `Scoped(name)` routes to `category` field of `build_cli_help_json` — see dispatch arm at help.rs:108-112
- [x] Conflict policy extended: positional + explicit `--category` → error with hint via process::exit(2)
- [x] Unit tests added: `positional_exact_category_routes_to_scoped` (covers channel/agent_chat/fleet/session), `positional_with_explicit_category_errors`, `positional_unknown_routes_to_name_filter` (replaces & widens the T-2004 single-case test); all 11 help::tests pass
- [x] Re-export `registry_category_names` from `termlink-mcp/src/lib.rs`
- [x] `cargo build -p termlink --release` clean — `Finished release profile [optimized] target(s) in 5m 51s`
- [x] `cargo test -p termlink-mcp --lib` 835 pass / 0 failed (registry_category_names exercised transitively)
- [x] Live smoke: `termlink help channel` renders 17 channel-category tools (header: "TermLink MCP tool registry — 17 tools across 1 categories") — confirmed scope-based render, no substring noise
- [x] Live smoke: `termlink help termlink_channel_post` still drills in (priority 1)
- [x] Live smoke: `termlink help zzzzz` still substring-searches (priority 3 fallback, 0 matches)

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
target/release/termlink help zzzzz >/dev/null 2>&1

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

### 2026-06-05T18:17:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2005-termlink-help-category-3-tier-positional.md
- **Context:** Initial task creation
