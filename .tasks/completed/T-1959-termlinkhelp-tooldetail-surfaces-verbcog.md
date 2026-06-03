---
id: T-1959
name: "termlink_help tool_detail surfaces verb_cognates across categories"
description: >
  MCP client arc T-1959: T-1956's related_tools only surfaces siblings sharing the same first-3-segment prefix (intra-domain). An LLM landing on termlink_agent_post never learns about termlink_channel_post / termlink_broadcast — verb-cognate tools in different domains. Add verb_cognates field: tools sharing the LAST segment but a DIFFERENT first segment, capped at 5. When the verb family exceeds 5 (common verbs like _status, _list signaling low discriminative value), the field is omitted to avoid noise. related_tools (intra-domain) preserved unchanged.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T21:54:35Z
last_update: 2026-06-03T21:57:27Z
date_finished: null
---

# T-1959: termlink_help tool_detail surfaces verb_cognates across categories

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] New `verb_cognates(target, categories, max_count)` function returns cross-category verb mates and respects the noise gate. — `crates/termlink-mcp/src/tools.rs:909-948` (compares `segs[1]` since `segs[0]` is always "termlink"; noise gate at `cognates.len() > max_count`).
- [x] `build_help_json` tool_detail mode emits `verb_cognates: [...]` when non-empty; field omitted entirely when count exceeds 5. — `tools.rs:1015-1018` (mutable JSON, conditional insert); max_count=5 passed at the call site.
- [x] `related_tools` (intra-domain) preserved unchanged. Both fields coexist. — `tools.rs:1014` (related_tools field still unconditional); regression-verified by `verb_cognates_omits_common_verb` test which asserts related_tools array still present even when cognates omitted.
- [x] New test `verb_cognates_finds_cross_category_post` — `tools.rs:35329-35353`, passes — `termlink_channel_post` surfaced, no `termlink_agent_*` intra-domain dupes leak.
- [x] New test `verb_cognates_omits_common_verb` — `tools.rs:35355-35375`, passes — `_status` family exceeds noise gate; field absent.
- [x] New test `verb_cognates_never_includes_self` — `tools.rs:35377-35399`, passes — sweeps EVERY tool and asserts self-exclusion.
- [x] Full lib test suite: `test result: ok. 703 passed; 0 failed`.

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
cargo test --lib --package termlink-mcp verb_cognates 2>&1 | grep -qE "test result: ok\. 3 passed"
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

### 2026-06-03T21:54:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1959-termlinkhelp-tooldetail-surfaces-verbcog.md
- **Context:** Initial task creation
