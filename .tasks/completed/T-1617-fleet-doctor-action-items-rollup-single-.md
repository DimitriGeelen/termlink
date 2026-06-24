---
id: T-1617
name: "fleet doctor: action items rollup (single-line summary of what to do)"
description: >
  fleet doctor: action items rollup (single-line summary of what to do)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-06T11:36:20Z
last_update: 2026-05-06T11:42:41Z
date_finished: 2026-05-06T11:42:41Z
---

# T-1617: fleet doctor: action items rollup (single-line summary of what to do)

## Context

`fleet doctor` currently emits per-hub `[WARN]` lines (one per stale-version hub) + a summary footer. With 5 stale hubs, the operator sees 5 identical WARN lines totaling ~1.2KB of repeated text. Add a single-line `Action items:` rollup at the bottom that aggregates by class (e.g., "Version skew: 5/5 hubs on 0.9.0 — restart with newer binary"). Per-hub WARN lines stay (operators may want per-hub context), but the rollup gives at-a-glance signal of what to actually DO.

Builds on T-1614/15/16 operator-fluent theme. Same dogfood target (5 stale-version hubs) gives immediate verification.

## Acceptance Criteria

### Agent
- [x] After `Versions in fleet:` line, emit `Action items:` block when any of: stale versions detected, FAIL hubs present, or AUTH-FAIL present
- [x] Stale-version rollup: "Version skew: N/M hubs on <version> — restart hub processes to pick up newer binary (CLI is on <cli_version>)"
- [x] FAIL/AUTH-FAIL rollups also emit when applicable (one line each)
- [x] If no action items, skip the section entirely (clean fleet stays clean output)
- [x] Build clean
- [x] Dogfood: run `target/release/termlink fleet doctor`, observe Action items block lists "Version skew: 5/5 ..."

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
grep -aqF "Action items:" target/release/termlink
grep -aqF "Version skew:" target/release/termlink

## Recommendation

**Recommendation:** GO (small, high-clarity rollup).
**Rationale:** Operator gets at-a-glance "what to do" without parsing N identical WARN lines. Per-hub detail preserved for context. Same dogfood target (5 stale 0.9.0 hubs) gives immediate verification.
**Evidence:** Live fleet has 5/5 hubs on stale 0.9.0; current output emits 5 identical WARN lines without summarizing the action.

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

### 2026-05-06T11:36:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1617-fleet-doctor-action-items-rollup-single-.md
- **Context:** Initial task creation

### 2026-05-06T11:42:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
