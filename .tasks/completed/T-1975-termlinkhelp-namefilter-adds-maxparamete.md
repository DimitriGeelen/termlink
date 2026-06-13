---
id: T-1975
name: "termlink_help: name_filter adds max_parameters filter (cycle 10 slice 6)"
description: >
  MCP arc cycle 10 slice 6: add max_parameters: Option<usize> to HelpParams. When set in combination with name_filter, suppresses matches whose parameter_count exceeds the threshold. Lets LLMs ask 'find me simple tools matching X' in one round-trip. Composes T-1972's name_filter parameter_count signal with explicit filtering.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-04T06:03:29Z
last_update: 2026-06-04T06:10:28Z
date_finished: 2026-06-04T06:11:46Z
---

# T-1975: termlink_help: name_filter adds max_parameters filter (cycle 10 slice 6)

## Context

MCP arc cycle 10 slice 6. Adds `max_parameters: Option<usize>` to `HelpParams`. When set in combination with `name_filter` (or alone), suppresses rows whose `parameter_count` exceeds the threshold. Lets LLMs ask "find me tools matching X with at most N params" in one round-trip — the cost-aware-search verb that composes T-1972's per-row parameter_count signal with explicit filtering. Solo use (no name_filter) walks the full registry, filtered by arity only — answers "what are all the zero-arg primitives?" in one shot.

## Acceptance Criteria

### Agent
- [x] `HelpParams` gains `max_parameters: Option<usize>` field with doc comment (tools.rs:8079-8087)
- [x] `termlink_help` method threads the field through to `build_help_json` (tools.rs:12289-12290)
- [x] `build_help_json` signature gains `max_parameters: Option<usize>` argument (tools.rs:1001-1010); all 64 callers migrated
- [x] When `max_parameters` is set, `name_filter` mode suppresses match rows whose `parameter_count` > threshold (tools.rs:1340-1347 arity_predicate gate)
- [x] When `max_parameters` is set WITHOUT `name_filter`, walks the registry and returns matches in the same envelope shape (tools.rs:1290-1297, `standalone_arity_filter` opens the search branch with empty-needle = match-all)
- [x] Macro doc-string updated: mentions `max_parameters`, describes combined-with-name_filter and standalone use (tools.rs:12310, T-1975 block appended to name_filter shape line)
- [x] Drift test required_fields includes `max_parameters` (tools.rs:35840-35842)
- [x] Invariant test: with `max_parameters=0`, every returned row has `parameter_count == 0` (tools.rs:36694-36713 max_parameters_zero_returns_only_zero_arity_tools)
- [x] Invariant test: with `max_parameters=N`, every returned row has `parameter_count <= N` (tools.rs:36715-36744 max_parameters_n_caps_arity_in_matches)
- [x] Invariant test: standalone `max_parameters=0` total_matches equals summary.zero_arity_tools (tools.rs:36746-36768 max_parameters_standalone_count_matches_summary_zero_arity_tools)
- [x] `cargo test --lib --package termlink-mcp` passes (740 tests, 0 failed; +3 from T-1974 baseline of 737)

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

### 2026-06-04T06:03:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1975-termlinkhelp-namefilter-adds-maxparamete.md
- **Context:** Initial task creation

### 2026-06-04T06:04:19Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
