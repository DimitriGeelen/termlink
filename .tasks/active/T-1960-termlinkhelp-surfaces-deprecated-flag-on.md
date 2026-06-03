---
id: T-1960
name: "termlink_help surfaces deprecated flag on legacy tools"
description: >
  MCP client arc T-1960: legacy / retirement-WIP tools (inbox primitives, remote-inbox-*) carry 'legacy' / 'T-1166 retirement WIP' phrases in their short descriptions but LLMs see them in name_filter and tool_detail results indistinguishable from live tools. Add a derived deprecated boolean field: scan short description for 'legacy', 'retirement', 'deprecated', '(T-1166' (case-insensitive). Surface in both tool_detail and name_filter match rows so the LLM can route around deprecated paths. Pure derivation — no curation, drift-proof.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T22:00:42Z
last_update: 2026-06-03T22:00:42Z
date_finished: null
---

# T-1960: termlink_help surfaces deprecated flag on legacy tools

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [ ] New `is_deprecated(description: &str) -> bool` pure function classifies a short description as deprecated when (case-insensitive) it contains any of: "legacy", "retirement", "deprecated", "T-1166". Single source of truth for derivation.
- [ ] `build_help_json` tool_detail mode adds `deprecated: bool` field to the response so an LLM drilling into one tool sees the retirement status without parsing the description string.
- [ ] `build_help_json` name_filter mode adds `deprecated: bool` to each match row so search results carry the routing signal (LLM ranks live tools higher than retiring ones).
- [ ] New test `is_deprecated_flags_known_legacy_tools` — sweeps the real registry, asserts every tool whose description contains a marker phrase is flagged. At minimum: `termlink_inbox_status`, `termlink_inbox_list`, `termlink_inbox_clear`, `termlink_remote_inbox_status`, `termlink_remote_inbox_list`, `termlink_remote_inbox_clear`.
- [ ] New test `is_deprecated_excludes_live_tools` — asserts live tools (`termlink_agent_post`, `termlink_channel_post`, `termlink_help`, `termlink_doctor`) are NOT flagged.
- [ ] New test `tool_detail_response_includes_deprecated_field` — asserts the field is present (true OR false, both meaningful) so LLMs can rely on its presence rather than guessing absence semantics.
- [ ] New test `name_filter_matches_carry_deprecated_field` — asserts every match row includes the flag.
- [ ] Full lib test suite reports 707 passed (+4), 0 failed.

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
cargo test --lib --package termlink-mcp is_deprecated 2>&1 | grep -qE "test result: ok\. 2 passed"
cargo test --lib --package termlink-mcp tool_detail_response_includes_deprecated_field 2>&1 | grep -q "test result: ok. 1 passed"
cargo test --lib --package termlink-mcp name_filter_matches_carry_deprecated_field 2>&1 | grep -q "test result: ok. 1 passed"
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

### 2026-06-03T22:00:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1960-termlinkhelp-surfaces-deprecated-flag-on.md
- **Context:** Initial task creation
