---
id: T-1946
name: "termlink_help reverse coverage invariant — every real tool must have a help entry"
description: >
  Add structural CI guard preventing future help-registry coverage gaps

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T20:14:56Z
last_update: 2026-06-03T20:14:56Z
date_finished: null
---

# T-1946: termlink_help reverse coverage invariant — every real tool must have a help entry

## Context

T-1945 closed coverage at 100% (252/252 real `#[tool(name=…)]` entries surfaced
in `help_categories()`). T-1941 installed a forward-direction guard
(`help_registry_has_no_phantom_entries`) — a help entry MUST resolve to a real
tool. The reverse direction — every real tool MUST have a help entry — is
currently held only by audit (`comm -23` diff at slice close). Without a
structural test, the next time a developer adds a new `#[tool(name=…)]` macro
they can ship without updating `help_categories()` and the coverage gap
regresses silently until the next audit.

This task adds the missing reverse-direction unit test
`help_registry_covers_all_real_tools`. Test pattern mirrors the existing
phantom guard: scan `tools.rs` source via `include_str!` + regex, walk
`help_categories()`, diff. Failure message names the missing tools so the fix
is obvious.

## Acceptance Criteria

### Agent
- [ ] `help_registry_covers_all_real_tools` test added in the `#[cfg(test)] mod tests` block alongside the phantom guard
- [ ] Test passes on current source (252/252 coverage)
- [ ] Failure mode produces an actionable message naming each missing tool
- [ ] `cargo test -p termlink-mcp --lib` passes (counts up from 678 → 679)

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
cargo test -p termlink-mcp --lib help_registry_covers_all_real_tools -- --nocapture
cargo test -p termlink-mcp --lib help_registry_has_no_phantom_entries -- --nocapture

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

### 2026-06-03T20:14:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1946-termlinkhelp-reverse-coverage-invariant-.md
- **Context:** Initial task creation
