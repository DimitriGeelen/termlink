---
id: T-1651
name: "remote profile add — tip when --bootstrap-from omitted (heal-readiness add-time nudge)"
description: >
  remote profile add — tip when --bootstrap-from omitted (heal-readiness add-time nudge)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [auth, fleet, ergonomic, discoverability]
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: [T-1650, T-1648, T-1649, T-1291]
created: 2026-05-16T22:23:56Z
last_update: 2026-05-16T22:31:50Z
date_finished: 2026-05-16T22:31:50Z
---

# T-1651: remote profile add — tip when --bootstrap-from omitted (heal-readiness add-time nudge)

## Context

T-1650 surfaced heal-readiness at *idle* in `remote profile list`. The cleaner ergonomic is "do the right thing the first time" — when an operator adds a profile WITHOUT `--bootstrap-from`, emit a one-line tip at add time. This catches the gap at the moment it's introduced, not later via list-output review or incident-time hint.

## Acceptance Criteria

### Agent
- [x] `remote profile add` emits a one-line tip when `--bootstrap-from` is omitted, recommending it for one-flag heal (T-1291)
- [x] Tip suppressed when `--bootstrap-from` is provided (no nag when configured)
- [x] Tip suppressed in JSON output mode (machine consumers don't need prose)
- [x] Tip wording cites T-1291 for traceability + names a concrete example (`ssh:<host>`)
- [x] Live smoke: `remote profile add` with and without `--bootstrap-from` shows the correct output
- [x] `cargo build --release -p termlink` succeeds

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

cargo build --release -p termlink 2>&1 | tail -3 | grep -E "Finished|error"

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

### 2026-05-16T22:23:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1651-remote-profile-add--tip-when---bootstrap.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-fdb3dc3e
- **Timestamp:** 2026-05-16T22:38:00Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-16T22:31:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
