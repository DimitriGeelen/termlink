---
id: T-1962
name: "termlink_help macro description documents post-T-1953 return fields"
description: >
  MCP client arc T-1962: the termlink_help #[tool(description=...)] macro string is the schema description MCP clients see at tool-discovery time. T-1953..T-1961 added parameters / verb_cognates / category_hint / deprecated to return envelopes, but the macro description never updated. LLMs reading the schema cannot know these fields exist. Update the description + add structural-invariant tests asserting each field name appears in the documented schema — drift-detection so future field additions can't ship without doc updates.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T22:09:06Z
last_update: 2026-06-03T22:11:14Z
date_finished: null
---

# T-1962: termlink_help macro description documents post-T-1953 return fields

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Macro string mentions `parameters` — `crates/termlink-mcp/src/tools.rs:12015` (in tool_detail return-shape sentence; also called out in the "drill-in" listing).
- [x] Macro string mentions `verb_cognates` — `tools.rs:12015` ("cross-domain tools sharing the trailing verb (omitted when noisy)").
- [x] Macro string mentions `category_hint` — `tools.rs:12015` ("OR `category_hint` (T-1958, when the passed value is actually a category name)").
- [x] Macro string mentions `deprecated` — `tools.rs:12015` (called out in both default return shape AND name_filter return shape).
- [x] Macro string mentions per-category `description` — `tools.rs:12015` ("the per-category `description` (T-1957) lets you route at category-discovery time").
- [x] New test `help_macro_description_documents_post_t1953_fields` — `tools.rs:35557-35596`, passes — sweeps the macro block extracted from source, asserts all 5 field names present.
- [x] Full lib suite: `test result: ok. 710 passed; 0 failed`.

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
cargo test --lib --package termlink-mcp help_macro_description_documents_post_t1953_fields 2>&1 | grep -q "test result: ok. 1 passed"
! cargo test --lib --package termlink-mcp 2>&1 | grep -E "FAILED"

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

### 2026-06-03T22:09:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1962-termlinkhelp-macro-description-documents.md
- **Context:** Initial task creation
