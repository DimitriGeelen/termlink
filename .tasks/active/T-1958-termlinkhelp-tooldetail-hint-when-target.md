---
id: T-1958
name: "termlink_help tool_detail hint when target is a category name"
description: >
  MCP client arc T-1958: when an LLM passes a category name (e.g. 'channel', 'session', 'kv') as tool_detail, the error path emits did_you_mean tool-name suggestions by Levenshtein, none of which are likely matches because the input was a valid CATEGORY (not a typoed tool). Add a category_hint field to the error envelope when target is a known category — points the LLM at category=<value> or list_categories=true. Unknown non-category inputs keep the existing did_you_mean behavior unchanged.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T21:50:40Z
last_update: 2026-06-03T21:53:01Z
date_finished: null
---

# T-1958: termlink_help tool_detail hint when target is a category name

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] When `tool_detail=<value>` and `<value>` matches a category name from `help_categories()`, the error envelope includes a `category_hint` field with copy-pasteable corrective syntax (`category=<value>` or `list_categories=true`). — `crates/termlink-mcp/src/tools.rs:956-968` (category-match branch fires before did_you_mean path).
- [x] When `tool_detail=<value>` and `<value>` is NOT a category name, existing behavior unchanged — `did_you_mean` array preserved, no `category_hint` emitted. — same site: the original did_you_mean branch is the fall-through; verified by regression test `tool_detail_unknown_no_category_hint`.
- [x] The error message itself (the `error` field) acknowledges the category match — "'<value>' is a category, not a tool" — instead of the generic "Unknown tool" line, so the LLM sees the diagnosis even if it ignores the hint field. — `tools.rs:959` (string literal).
- [x] New test `tool_detail_category_name_emits_category_hint` (positive case) — `tools.rs:35360-35395`, passes (1/1) — verifies error language, category_hint presence + content, did_you_mean absent.
- [x] New test `tool_detail_unknown_no_category_hint` (negative case) — `tools.rs:35397-35415`, passes (1/1) — verifies category_hint absent + did_you_mean present for non-category unknowns.
- [x] Full lib test suite reports 700 passed (+2), 0 failed. — `test result: ok. 700 passed; 0 failed`.

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
cargo test --lib --package termlink-mcp tool_detail_category_name_emits_category_hint 2>&1 | grep -q "test result: ok. 1 passed"
cargo test --lib --package termlink-mcp tool_detail_unknown_no_category_hint 2>&1 | grep -q "test result: ok. 1 passed"
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

### 2026-06-03T21:50:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1958-termlinkhelp-tooldetail-hint-when-target.md
- **Context:** Initial task creation
