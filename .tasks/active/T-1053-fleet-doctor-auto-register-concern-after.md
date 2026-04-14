---
id: T-1053
name: "fleet doctor auto-register concern after N sustained auth failures"
description: >
  fleet doctor auto-register concern after N sustained auth failures

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-14T18:33:50Z
last_update: 2026-04-14T18:33:50Z
date_finished: null
---

# T-1053: fleet doctor auto-register concern after N sustained auth failures

## Context

Second build task from T-1051 inception decomposition (Option D, G-019 compliance).
When a hub fails fleet-doctor with an auth-class error for 3+ consecutive runs AND the first failure is >24h old, auto-register a gap in `.context/project/concerns.yaml` so the sustained condition is visible in Watchtower and audits.

This complements T-1052: one observation creates a learning; a sustained pattern creates a concern.

Hook: `cmd_fleet_doctor` after `maybe_record_auth_mismatch_learning` — same dispatch point.
State store: `.context/working/.fleet-failure-state.yaml` keyed by hub name.

## Acceptance Criteria

### Agent
- [x] Per-hub failure state is persisted in `.context/working/.fleet-failure-state.json` (consecutive_failures, first_failure_at, last_failure_at, last_class, concern_registered). Format chosen: JSON (serde_json already a dep), not YAML, to avoid adding a new dep.
- [x] Passing fleet-doctor run resets a hub's counter to 0 and clears first_failure_at
- [x] Failing run increments counter, sets first_failure_at on 0→1 transition, updates last_failure_at
- [x] When `consecutive_failures >= 3` AND `(now - first_failure_at) > 24h` AND `concern_registered == false`: a gap is appended to `.context/project/concerns.yaml` with type=gap, severity=high, status=watching, trigger_fired=true
- [x] After registering a concern, `concern_registered` is flipped to true to prevent duplicates
- [x] 6 unit tests cover: first-failure recording, reset on success, threshold-not-met, threshold-met-write-concern, dedupe, parse_iso8601 round-trip + rejection
- [x] `cargo build -p termlink` clean, zero new clippy warnings
- [x] `cargo test -p termlink --bin termlink -- fleet_concern` passes (6 tests)

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

cargo build -p termlink 2>&1 | tail -5
cargo test -p termlink --bin termlink -- fleet_concern 2>&1 | grep -E "[0-9]+ passed"

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

### 2026-04-14T18:33:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1053-fleet-doctor-auto-register-concern-after.md
- **Context:** Initial task creation
