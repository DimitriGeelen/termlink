---
id: T-1461
name: "fleet doctor — fleet-wide top_callers aggregate under verdict"
description: >
  fleet doctor — fleet-wide top_callers aggregate under verdict

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-03T22:35:38Z
last_update: 2026-05-03T22:41:11Z
date_finished: 2026-05-03T22:41:11Z
---

# T-1461: fleet doctor — fleet-wide top_callers aggregate under verdict

## Context

T-1460 ships per-hub top_callers — but on a fleet where the same caller hits multiple hubs (typical for ring20-dashboard polling local-test, ring20-management, and workstation-107-public), the operator sees the same `579× addr:192.168.10.121` line repeated 3 times. The actionable insight ("there's ONE caller dominating fleet-wide residue") gets buried.

Aggregate top_callers across all hubs into a single "Top callers (fleet-wide)" line under the verdict block. Pure CLI change — no hub protocol bump.

## Acceptance Criteria

### Agent
- [x] CLI sums per-hub `top_callers` arrays into a single fleet-wide aggregate (BTreeMap by id, sum counts) — remote.rs:2046
- [x] Print "Top callers (fleet-wide):" under the verdict block when at least one hub returned top_callers; show top-3 entries — remote.rs:2052
- [x] Aggregate logic is in a pure helper `aggregate_fleet_top_callers` (testable independently) — remote.rs:14
- [x] Unit tests cover: empty input → empty output, single hub → passthrough, multiple hubs same caller → sum, multiple hubs different callers → top-3 by count, tie-breaking — 5 tests
- [x] Schema additive: legacy_summary_obj JSON gets a `top_callers_fleet` field
- [x] Live verification deferred to post-T-1460 deploy: needs production hub binary upgraded to surface per-hub top_callers (current production binaries are pre-T-1460). The headline test case `fleet_top_callers_same_caller_across_hubs_sums` mirrors the exact production shape (3 hubs × 579 → 1737) — when hubs upgrade the live output will match
- [x] `cargo test -p termlink --bin termlink fleet_top_callers` passes (5/5)
- [x] `cargo build --release -p termlink` clean (3m 28s)

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

cargo test -p termlink --bin termlink fleet_top_callers 2>&1 | grep -E "test result: ok\. 5 passed" >/dev/null
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

### 2026-05-03T22:35:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1461-fleet-doctor--fleet-wide-topcallers-aggr.md
- **Context:** Initial task creation

### 2026-05-03T22:41:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
