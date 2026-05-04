---
id: T-1459
name: "fleet doctor cut-readiness — distinguish CUT-READY-DECAYING from WAIT"
description: >
  fleet doctor cut-readiness — distinguish CUT-READY-DECAYING from WAIT

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-03T22:13:57Z
last_update: 2026-05-03T22:22:37Z
date_finished: 2026-05-03T22:22:37Z
---

# T-1459: fleet doctor cut-readiness — distinguish CUT-READY-DECAYING from WAIT

## Context

`fleet doctor --legacy-usage` currently reports a binary verdict (`CUT-READY` / `WAIT` / `UNCERTAIN`). `WAIT` triggers on **any** legacy traffic in the audit window — including pure decay residue from heartbeats that stopped polling hours ago. The operator must read the per-hub `last call …(decay residue)` tag to interpret whether it's safe to cut.

Refinement: introduce `CUT-READY-DECAYING` — fired when total_legacy > 0 but no hub has had a legacy call within the last 5 minutes. Same threshold as the per-hub ACTIVE/decay-residue tag at remote.rs:1931. Operator can then act on the top-line directly.

## Acceptance Criteria

### Agent
- [x] Pure function `compute_cut_readiness_verdict` extracted from inline logic at remote.rs:~1896, takes (hubs_with_traffic, hubs_unsupported, hubs_no_audit, hubs_clean, now_ms), returns one of {CUT-READY, CUT-READY-DECAYING, WAIT, UNCERTAIN}
- [x] `ACTIVE_TRAFFIC_THRESHOLD_SECS = 300` constant introduced and shared between verdict + per-hub age tag (single source of truth)
- [x] CUT-READY-DECAYING printed with operator hint: residue is historical, audit window will clear naturally, force-cut OK if operator chooses
- [x] Doc comment on `--legacy-usage` flag (cli.rs:3056) updated to mention the new verdict
- [x] Unit tests cover all 4 verdicts: CUT-READY (all clean), CUT-READY-DECAYING (residue >5min), WAIT (residue <5min), UNCERTAIN (unsupported hubs) — 11 tests including boundary + zero-ts
- [x] `cargo test -p termlink --bin termlink cut_readiness` passes (11/11) — note: termlink has no lib target, AC adjusted to match crate shape
- [x] `cargo build --release -p termlink` clean (3m 30s)
- [x] Live verification: previously WAIT, now reports `Verdict: CUT-READY-DECAYING` on the fleet with the historical-residue hint — verified 2026-05-04

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

cargo test -p termlink --bin termlink cut_readiness 2>&1 | grep -E "test result: ok\. 11 passed" >/dev/null
! cargo check -p termlink 2>&1 | grep -E "^(warning:|error)" | grep -v "^warning:" | grep -q .

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

### 2026-05-03T22:13:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1459-fleet-doctor-cut-readiness--distinguish-.md
- **Context:** Initial task creation

### 2026-05-03T22:22:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
