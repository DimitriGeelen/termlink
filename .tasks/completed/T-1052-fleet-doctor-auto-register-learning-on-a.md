---
id: T-1052
name: "fleet doctor auto-register learning on auth-mismatch"
description: >
  fleet doctor auto-register learning on auth-mismatch

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-14T10:15:08Z
last_update: '2026-08-18T18:58:43Z'
date_finished: 2026-04-15T13:35:44Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:43Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:43Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-1052: fleet doctor auto-register learning on auth-mismatch

## Context

First build task from T-1051 inception decomposition (Option D, R1 compliance).
When `termlink fleet doctor` detects an auth-mismatch or TOFU violation against a hub, it must auto-register a project learning that carries the hub address, the observed fingerprint, and a UTC timestamp — so a future agent can detect memory drift (the recorded fingerprint != the current one).

Hook point: `crates/termlink-cli/src/commands/remote.rs` — `cmd_fleet_doctor`, the `Ok(Err(e))` branch at line ~1414.

## Acceptance Criteria

### Agent
- [x] `cmd_fleet_doctor` auto-registers a learning in `.context/project/learnings.yaml` when `classify_fleet_error` matches "Secret mismatch" or "certificate changed"
- [x] Learning text contains: hub name, address, current TOFU fingerprint (or "unknown" if not yet pinned), and ISO-8601 UTC timestamp
- [x] Auto-registration is deduped via `.context/working/.fleet-learning-<hub>` marker: skip if the marker is <24h old AND the fingerprint matches
- [x] At least one unit/integration test exercises the dedupe path and the first-write path
- [x] `cargo build -p termlink` succeeds with zero new clippy warnings
- [x] `cargo test -p termlink --bin termlink -- fleet_learning` passes (5 tests)

## Verification

cargo build -p termlink 2>&1 | tail -5
cargo test -p termlink --bin termlink -- fleet_learning 2>&1 | grep -E "5 passed"

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

### 2026-04-14T10:15:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1052-fleet-doctor-auto-register-learning-on-a.md
- **Context:** Initial task creation

### 2026-04-15T13:35:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
