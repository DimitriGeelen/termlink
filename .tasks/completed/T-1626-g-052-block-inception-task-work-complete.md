---
id: T-1626
name: "G-052: block inception task work-completed without decision"
description: >
  G-052: block inception task work-completed without decision

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [tests/test_g052_inception_decision_gate.sh]
related_tasks: []
created: 2026-05-06T18:04:08Z
last_update: 2026-05-06T18:17:49Z
date_finished: 2026-05-06T18:17:49Z
---

# T-1626: G-052: block inception task work-completed without decision

## Context

G-052 (high): an inception task (T-1448) was silently moved active→completed during an unrelated heartbeat-script commit. The commit-msg hook's inception gate is BLOCK-on-commit only; it doesn't intercept the lifecycle path that finalizes via `update-task.sh --status work-completed`. Net effect: operator's pending-decision queue can silently empty.

This task adds a structural gate to `update-task.sh` that blocks `work-completed` for `workflow_type: inception` tasks unless a `**Decision**: GO|NO-GO|DEFER` line exists in the task body (i.e. `fw inception decide` was run).

## Acceptance Criteria

### Agent
- [x] `update-task.sh` defines `check_inception_decision()` modelled on `check_rca_for_bugfix()`: only fires on `--status work-completed` for `workflow_type: inception`, looks for `^\*\*Decision\*\*:\s*(GO|NO-GO|DEFER)\b` in the task body, blocks with actionable message if missing
- [x] Gate is wired into the work-completed gate sequence in the main flow (alongside `check_rca_for_bugfix`, `check_evolution_log`)
- [x] `--skip-inception-decision` bypass flag accepted, logged via `log_gate_bypass` (mirrors `--skip-rca` pattern), and aggregated into deprecated `--force`
- [x] `--help` listing includes the new flag
- [x] Existing tests (if any in tests/) still pass; manual smoke: try `update-task.sh T-XXX --status work-completed` on an inception task without decision → exits non-zero with the helpful message; with the `**Decision**: GO` line present → succeeds

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
bash tests/test_g052_inception_decision_gate.sh

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

## Updates

### 2026-05-06T18:04:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1626-g-052-block-inception-task-work-complete.md
- **Context:** Initial task creation

### 2026-05-06T18:17:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
