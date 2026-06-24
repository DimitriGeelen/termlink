---
id: T-1615
name: "fleet doctor summary undercounts WARN (hard-coded total_warn=0)"
description: >
  fleet doctor summary undercounts WARN (hard-coded total_warn=0)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-06T11:11:30Z
last_update: 2026-05-06T11:18:16Z
date_finished: 2026-05-06T11:18:16Z
---

# T-1615: fleet doctor summary undercounts WARN (hard-coded total_warn=0)

## Context

`fleet doctor` emits per-hub `[WARN] hub_version=0.9.0 — running binary predates T-1458` lines for stale-version hubs (line 2622 in `cmd_fleet_doctor`), but the summary footer says `0 warn` regardless of how many WARN lines fired. Root cause at line 2480: `let total_warn: u32 = 0;` declared immutable and never incremented. Operator trust issue — the summary contradicts the body of the report.

Live observation 2026-05-06: 5 hubs all on stale 0.9.0 binaries → 5 [WARN] lines emitted, summary says `5 ok, 0 warn, 0 fail`. The "ok" inflates while "warn" stays zero.

## Acceptance Criteria

### Agent
- [x] `total_warn` declared `mut` (line ~2480 in remote.rs)
- [x] Increment `total_warn` whenever a stale-version `[WARN]` is emitted (logical state, regardless of `!json` branch)
- [x] `target/release/termlink fleet doctor` now reports correct WARN count matching emitted [WARN] lines
- [x] JSON output's `summary.warn` field reflects the same correct count
- [x] No regression: PASS / FAIL counts unchanged on the same fleet

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

test -x target/release/termlink
grep -qF "let mut total_warn" crates/termlink-cli/src/commands/remote.rs
grep -qF "total_warn += 1;" crates/termlink-cli/src/commands/remote.rs

## Recommendation

**Recommendation:** GO (small, observable, real bug — operator trust gain).
**Rationale:** Summary contradicting body undermines the whole report. One-line fix + counter increment.
**Evidence:** Live `fleet doctor` emits 5 `[WARN]` lines but summary says `0 warn`.

## RCA

**Symptom:** `fleet doctor` summary footer reports `0 warn` even when N `[WARN]` lines fire in the body of the report.
**Root cause:** `total_warn: u32` declared immutable at line 2480 and never incremented anywhere. The variable was placeholder scaffolding for a counter that was never wired up.
**Why structurally allowed:** No test pins consistency between emitted `[WARN]` lines and `summary.warn` count. Rust didn't flag the unused-mut situation because the variable was already declared immutable; an immutable variable that is read but never mutated is valid code.
**Prevention:** A unit test could spin up a synthetic hub returning `version=0.9.0` and assert that `summary.warn >= 1`. Cheaper alternative: a clippy lint `clippy::needless_let_with_immutable_default_for_counter` if it existed; absent that, the live-fleet smoke regression (every operator-side `fleet doctor` invocation now exercises this) is the practical guard.

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

### 2026-05-06T11:11:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1615-fleet-doctor-summary-undercounts-warn-ha.md
- **Context:** Initial task creation

### 2026-05-06T11:18:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
