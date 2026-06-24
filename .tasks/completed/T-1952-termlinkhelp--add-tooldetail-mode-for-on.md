---
id: T-1952
name: "termlink_help — add tool_detail mode for one-tool drill-in (closes 3-step discovery loop)"
description: >
  Return full macro description for a named tool — extract from source via cached regex

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-03T20:46:58Z
last_update: 2026-06-03T20:51:52Z
date_finished: 2026-06-03T20:53:06Z
---

# T-1952: termlink_help — add tool_detail mode for one-tool drill-in (closes 3-step discovery loop)

## Context

`termlink_help` now supports three modes:
1. Default — per-category listings (built T-1942/etc, ~252 entries)
2. `name_filter` — substring search (T-1940)
3. `list_categories` — overview, ~27 categories with counts (T-1948)

What's missing for the discovery loop: after the LLM narrows to one tool via
overview or substring, the only data available is the registry one-liner
(~10-20 words). The full `#[tool(description=…)]` macro text — typically
2-5 sentences with envelope shape, return type, and usage notes — is
invisible until the LLM actually invokes the tool. That's a wasted round-trip
when the LLM is unsure which of 2-3 candidates is right.

This slice adds a fourth mode: `tool_detail: Option<String>` — pass a tool
name, get back its category, short description (from help_categories), and
full description (extracted from the macro). One round-trip closes the
3-step pattern: list_categories → category → tool_detail.

Implementation: scan `tools.rs` source via `include_str!` + regex (same
pattern as T-1941's phantom guard), cache in a `OnceLock<HashMap>` so the
extraction is paid once per process.

## Acceptance Criteria

### Agent
- [x] `HelpParams` struct gets `tool_detail: Option<String>` field with rustdoc
  - Evidence: 5-line rustdoc on the new field at tools.rs (commit `3d1278da`); explains 3-step discovery pattern
- [x] New free fn `tool_descriptions() -> &'static HashMap<&'static str, &'static str>` extracts (name, full_desc) pairs from `tools.rs` via include_str! + regex, cached via `OnceLock`
  - Evidence: new fn at the top of `build_help_json` block; uses `OnceLock` + `include_str!("./tools.rs")` + `regex::Regex` matching `name = "X",\s*description = "Y..."`
- [x] `build_help_json` branches to detail mode when set, returning `{tool, name, category, short_description, full_description}`
  - Evidence: detail branch at top of `build_help_json` returns `{tool, name, category, short_description, full_description}` JSON
- [x] Detail mode returns error with available-modes hint when tool name not found (not just empty result)
  - Evidence: `json_err(format!("Unknown tool '{target}'. Use list_categories=true ... or name_filter ..."))`
- [x] `termlink_help` wrapper passes the new param through
  - Evidence: `let detail = p.tool_detail.as_deref();` then `build_help_json(..., detail)`
- [x] Top-level `termlink_help` tool description mentions tool_detail mode
  - Evidence: description now says "Four modes: ... (4) `tool_detail=<tool_name>` returns one tool's category + short ... + FULL macro description (T-1952)"
- [x] Unit test: `help_tool_detail_returns_full_description` — verifies a known tool returns both short (registry) and full (macro) descriptions with category
  - Evidence: test passes — calls `tool_detail=Some("termlink_help")`, asserts category/short/full all populated and full > short in length
- [x] Unit test: `help_tool_detail_unknown_returns_error` — verifies error path with actionable hint
  - Evidence: test passes — calls `tool_detail=Some("termlink_does_not_exist")`, asserts error echoes the name and mentions a discovery mode
- [x] Unit test: `tool_descriptions_extracts_all_real_tools` — sanity check the regex catches every `#[tool(name=…)]` entry (parallels phantom-guard pattern)
  - Evidence: test passes — compares phantom-guard's regex-extracted name set against `tool_descriptions()` keys, asserts no missing
- [x] `cargo test -p termlink-mcp --lib` passes 685 (682 + 3 new)
  - Evidence: `test result: ok. 685 passed; 0 failed; 0 ignored; 0 measured` — +3 from prior 682 baseline

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
cargo test -p termlink-mcp --lib tool_descriptions -- --nocapture
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

### 2026-06-03T20:46:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1952-termlinkhelp--add-tooldetail-mode-for-on.md
- **Context:** Initial task creation
