---
id: T-1972
name: "termlink_help: parameter_count on name_filter + default-mode rows (cycle 10 slice 3)"
description: >
  MCP arc cycle 10 slice 3: extend T-1971's parameter_count signal from tool_detail to name_filter matches and default-mode rows. Without this, an LLM browsing tools sees arity only via per-tool drill-in — costly. Each row carries its own parameter_count so the LLM can rank tools in a single round-trip. Drift-proof: sourced from the same parse_tool_parameters() output that backs tool_detail.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-04T05:52:02Z
last_update: 2026-06-04T05:53:05Z
date_finished: null
---

# T-1972: termlink_help: parameter_count on name_filter + default-mode rows (cycle 10 slice 3)

## Context

MCP arc cycle 10 slice 3. T-1971 surfaces `parameter_count` only in `tool_detail` (per-tool drill-in). When an LLM uses `name_filter` to browse multiple matches or scans the default-mode catalog, ranking by arity still requires per-tool drill-ins — defeating the round-trip economy. This slice adds the same field to both bulk-listing modes via O(1) `tool_params()` lookups.

## Acceptance Criteria

### Agent
- [ ] `name_filter` matches rows add `parameter_count` field (integer, == tool_params lookup → vec.len(), 0 when not in registry)
- [ ] Default-mode tool rows add `parameter_count` field, same source
- [ ] Macro doc-string shape lines updated to include `parameter_count` in BOTH `name_filter` and default-mode envelope shapes
- [ ] Drift test: already covers `parameter_count` (T-1971); no extension needed (verify the existing required_fields entry covers both new sites since the macro contains the token wherever it appears)
- [ ] Invariant test: `name_filter` row `parameter_count` matches the `tool_detail` parameter_count for the same tool name — cross-mode consistency
- [ ] Invariant test: default-mode row `parameter_count` matches `tool_detail` for the same tool — cross-mode consistency
- [ ] `cargo test --lib --package termlink-mcp` passes (expect 733+ tests, 0 failed)

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

### 2026-06-04T05:52:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1972-termlinkhelp-parametercount-on-namefilte.md
- **Context:** Initial task creation

### 2026-06-04T05:53:05Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
