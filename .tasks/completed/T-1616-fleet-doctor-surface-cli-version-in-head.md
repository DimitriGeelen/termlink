---
id: T-1616
name: "fleet doctor: surface CLI version in header + skew context in WARN"
description: >
  fleet doctor: surface CLI version in header + skew context in WARN

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-06T11:21:19Z
last_update: 2026-05-06T11:27:36Z
date_finished: 2026-05-06T11:27:36Z
---

# T-1616: fleet doctor: surface CLI version in header + skew context in WARN

## Context

`fleet doctor` reports each hub's version inline (`(version: 0.9.0)`) and emits a stale-binary WARN, but never tells the operator what version their CLI is. Operator has to look in two places (`termlink info` for CLI, `fleet doctor` for hubs) to compare versions and gauge skew. Header-line addition makes the comparison immediate.

Live observation 2026-05-06: CLI is 0.9.2042 (d41fec98), all 5 hubs report 0.9.0 — that's exactly the kind of skew operators need to see at a glance.

## Acceptance Criteria

### Agent
- [x] `fleet doctor` header line includes CLI version: `Fleet doctor: N hub(s) configured (CLI 0.9.XXXX [hash])`
- [x] Stale-version WARN message includes CLI version for skew context: `hub_version=0.9.0, cli_version=0.9.XXXX — running hub binary predates ...`
- [x] Both changes use `env!("CARGO_PKG_VERSION")` and `option_env!("GIT_COMMIT")` (existing build.rs pattern, T-1458)
- [x] Build clean, no new warnings introduced
- [x] Dogfood: run `target/release/termlink fleet doctor`, observe header shows CLI version + WARN includes both versions

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
grep -aqF "(CLI " target/release/termlink
grep -aqF "cli_version=" target/release/termlink

## Recommendation

**Recommendation:** GO (small, high-clarity, no new dep).
**Rationale:** Header + WARN both gain skew context for free; operator no longer has to cross-reference `termlink info` and `fleet doctor` to gauge fleet drift.
**Evidence:** Live fleet shows 5 hubs on 0.9.0, CLI on 0.9.2042 — skew is the dominant fact, but currently shown in two separate places.

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

### 2026-05-06T11:21:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1616-fleet-doctor-surface-cli-version-in-head.md
- **Context:** Initial task creation

### 2026-05-06T11:27:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
