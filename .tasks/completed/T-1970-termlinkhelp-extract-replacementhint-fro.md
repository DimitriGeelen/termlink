---
id: T-1970
name: "termlink_help: extract replacement_hint from deprecated tool descriptions (cycle 10)"
description: >
  MCP arc cycle 10 slice 1: surface a replacement_hint field on every deprecated tool so an LLM client routing the registry learns what to use instead. Derived from description text (parser extracts (use NAME instead) marker) — zero curated lists, auto-clears when T-1166 lands and the deprecation phrasing goes away. Surfaces in tool_detail, name_filter matches, and default mode for deprecated tools.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-04T05:42:54Z
last_update: 2026-06-04T05:47:30Z
date_finished: null
---

# T-1970: termlink_help: extract replacement_hint from deprecated tool descriptions (cycle 10)

## Context

MCP arc cycle 10, slice 1. Cycle 9 closed at T-1969 with a 6-mode `termlink_help` surface and 723 lib tests. Today there are 6 deprecated tools (3 `termlink_inbox_*` + 3 `termlink_remote_inbox_*`, all T-1166 retirement WIP). An LLM client routing the registry currently sees `deprecated: true` but nothing about the replacement. Adding a `replacement_hint` field, derived from a `(use NAME instead)` marker in the description, gives the client a one-shot upgrade path. Drift-proof: the marker lives in the description string, so when T-1166 lands and the deprecated tools are deleted, the hint vanishes automatically.

## Acceptance Criteria

### Agent
- [x] `extract_replacement_hint(description: &str) -> Option<&str>` parser added to tools.rs, matches the literal pattern `(use <name> instead)` and returns `<name>`; returns None when no marker present (tools.rs:589-605)
- [x] All 6 deprecated tool descriptions updated to end with `(use termlink_channel_subscribe instead)` — the canonical replacement per CLAUDE.md T-1166 retirement plan (tools.rs:322-324 for remote_inbox_*, tools.rs:557-559 for inbox_*)
- [x] `tool_detail` envelope adds `replacement_hint` field — populated for deprecated tools, omitted for live tools (tools.rs:1078-1086)
- [x] `name_filter` matches rows add `replacement_hint` field when the matched tool is deprecated (tools.rs:1265-1280)
- [x] Default mode tool entries add `replacement_hint` field when deprecated (tools.rs:1311-1330)
- [x] Drift test extended to assert the new fields appear in the rmcp `#[tool(description=...)]` macro doc string (tools.rs:35817-35821 adds `replacement_hint` to required_fields; macro updated tools.rs:12215 with `replacement_hint?` shape + T-1970 description block)
- [x] Invariant test: every deprecated tool has a non-empty `replacement_hint` (tools.rs:35923-35946 every_deprecated_tool_has_replacement_hint)
- [x] Invariant test: `extract_replacement_hint` returns None on every non-deprecated tool's description (tools.rs:35948-35968 extract_replacement_hint_returns_none_on_live_tools)
- [x] `cargo test --lib --package termlink-mcp` passes (730 tests passed, 0 failed; +7 from cycle-9 baseline of 723)

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
cargo test --lib --package termlink-mcp --quiet 2>&1 | tail -5 | grep -q "test result: ok"

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

### 2026-06-04T05:42:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1970-termlinkhelp-extract-replacementhint-fro.md
- **Context:** Initial task creation

### 2026-06-04T05:43:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
