---
id: T-1953
name: "termlink_help: tool_detail surfaces parameter schemas (derive-not-hardcode)"
description: >
  Add a third extractor (parallel to tool_descriptions()) that regex-scans tools.rs to build {tool_name: [{param_name, type, optional, doc}]}. Surface in tool_detail JSON response as parameters: [...]. Closes the call-the-tool-correctly gap — LLMs no longer have to guess param shapes or invoke-to-error to learn them. Auto-covers all 252 tools, structurally maintained, no curation. T-1952 follow-up.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T21:05:10Z
last_update: 2026-06-03T21:13:36Z
date_finished: 2026-06-03T21:17:30Z
---

# T-1953: termlink_help: tool_detail surfaces parameter schemas (derive-not-hardcode)

## Context

T-1952 closed the 3-step discovery loop: `list_categories` → `category` → `tool_detail`.
But `tool_detail` returns only descriptions — LLMs still have to guess parameter
shapes or invoke-to-error to discover them. This slice surfaces ground-truth
parameter info by regex-scanning tools.rs (same pattern as `tool_descriptions()`
at tools.rs:570), auto-covering all 252 tools with zero curation.

## Acceptance Criteria

### Agent
- [x] New `tool_params()` extractor function added to tools.rs — cached `OnceLock<HashMap<String, Vec<ParamInfo>>>`, regex-derived from `include_str!("./tools.rs")` (parallels `tool_descriptions()` at tools.rs:570). Implementation: `ParamInfo` struct + `parse_struct_fields()` helper + `tool_params()` at tools.rs:592-700 (commit `778625a4`).
- [x] `build_help_json` `tool_detail` branch includes `parameters: [{name, type, optional, doc?}]` in JSON response when the tool's param struct can be resolved. Tools with empty param structs return `parameters: []` (key present, array empty — distinguishes "no params" from "extraction failed"). Implementation: tools.rs:729-738.
- [x] New unit test: `tool_params_extracts_known_tool` — asserts that for `termlink_help`, the extractor returns the 4 expected fields (category/name_filter/list_categories/tool_detail), all marked optional. Implementation: tools.rs:34939-34970.
- [x] New unit test: `tool_detail_response_includes_parameters` — calls `build_help_json` with `tool_detail=Some("termlink_help")`, asserts response JSON contains a `parameters` array with ≥4 entries, each having name/type/optional. Implementation: tools.rs:34972-35010.
- [x] Bonus unit test: `tool_params_covers_majority_of_real_tools` — guards against Phase-1 regex regressions by asserting >50% tool coverage (caught the `[^{]` vs `[^)]` bug during dev — `Parameters(p)` has inner parens that the original `[^)]*` regex stopped at). Implementation: tools.rs:35012-35033.
- [x] `cargo check -p termlink-mcp --tests` passes (clean build, 20s).
- [x] `cargo test -p termlink-mcp --lib` passes — 688 total, 0 failed (685 prior + 3 new = exact +3 delta, no regressions).

## Verification

cargo check -p termlink-mcp --tests
cargo test -p termlink-mcp --lib --quiet 2>&1 | tail -5 | grep -E "test result.*ok"

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

### 2026-06-03T21:05:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1953-termlinkhelp-tooldetail-surfaces-paramet.md
- **Context:** Initial task creation
