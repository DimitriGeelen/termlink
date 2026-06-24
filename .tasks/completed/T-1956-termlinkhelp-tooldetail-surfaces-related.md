---
id: T-1956
name: "termlink_help: tool_detail surfaces related_tools (same name-prefix family)"
description: >
  When tool_detail returns a tool, also include related_tools[] — other tool names sharing the first 2-3 underscore segments. E.g., termlink_agent_react surfaces siblings termlink_agent_reactions, termlink_agent_react_*. Closes the workflow-continuity gap: LLMs see the verb family without a second category lookup. Derive-not-hardcode (no curation). T-1955 follow-up.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-03T21:27:57Z
last_update: 2026-06-03T21:30:14Z
date_finished: 2026-06-03T21:32:00Z
---

# T-1956: termlink_help: tool_detail surfaces related_tools (same name-prefix family)

## Context

When LLM resolves `tool_detail` for `termlink_agent_react`, it currently sees
description + params + did_you_mean (if unknown). Surfacing `related_tools[]`
— sibling names sharing the same name-prefix family — closes the workflow-
continuity gap: the LLM sees `termlink_agent_reactions`,
`termlink_agent_reaction_summary` etc. without a second round-trip. Derive
by splitting tool name on `_` and matching tools sharing the first 3
segments (e.g. `termlink_agent_react` → prefix `termlink_agent_react` →
family `termlink_agent_react*`). Excludes the tool itself.

## Acceptance Criteria

### Agent
- [x] New helper `related_tools(target, categories, limit) -> Vec<String>` at tools.rs:823-852 — splits target on `_`, takes first 3 segments as family prefix, returns siblings starting with that prefix (excluding self). Cap honored (10 in callsite).
- [x] `build_help_json` `tool_detail` success branch includes `related_tools: [...]` in JSON response. Implementation: tools.rs:803-806.
- [x] Unit test `tool_detail_related_tools_finds_family` — `termlink_agent_react` returns at least one of the reaction-family siblings (reactions/reaction_summary/reaction_rate/reactions_by). tools.rs:35029-35054.
- [x] Unit test `tool_detail_related_tools_excludes_self` — `termlink_agent_post` related_tools never contains itself. tools.rs:35056-35071.
- [x] `cargo test -p termlink-mcp --lib` passes — 696 total, 0 failed (694 prior + 2 new = exact +2 delta).

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

### 2026-06-03T21:27:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1956-termlinkhelp-tooldetail-surfaces-related.md
- **Context:** Initial task creation
